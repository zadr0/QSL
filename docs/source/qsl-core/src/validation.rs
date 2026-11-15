//! Checked wire validation and single-pass materialization.
//! Compressed output is bounded independently of the untrusted stored length.
//! Heap restoration is fallible; trusted pointer APIs remain separate.
use crate::{QslError, QslResult};
#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

pub const DEFAULT_MAX_DATA_SIZE: usize = 65535;

pub fn check_size(size: usize, maximum: usize) -> QslResult<()> {
    if size > maximum || size > isize::MAX as usize {
        return Err(QslError::SizeLimitExceeded);
    }
    Ok(())
}

/// Aggregate decoded/native storage bytes, independent of compressed frame size.
/// Fixed heap containers reserve their full object size, including spare capacity.
/// This bounds data, not allocator metadata or the process working set.
pub struct ValueBudget {
    remaining: usize,
}
impl ValueBudget {
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    pub fn new(maximum: usize) -> Self {
        Self { remaining: maximum }
    }
    pub fn charge(&mut self, bytes: usize) -> QslResult<()> {
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or(QslError::SizeLimitExceeded)?;
        Ok(())
    }
}

/// A fully checked run. Parsing never writes to output or allocates.
pub enum CompressedRun<'a> {
    Literal(&'a [u8]),
    Match { count: usize, distance: usize },
}
impl CompressedRun<'_> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Literal(bytes) => bytes.len(),
            Self::Match { count, .. } => *count,
        }
    }
}

/// The wire header bounds packed input only. Every run must independently fit
/// the actual destination bound before any reserve, backreference or write.
pub fn compressed_run<'a>(
    packed: &'a [u8],
    cursor: &mut usize,
    produced: usize,
    maximum: usize,
    width: usize,
) -> QslResult<Option<CompressedRun<'a>>> {
    if width != 1 && width != 2 {
        return Err(QslError::InvalidWidth);
    }
    if *cursor == packed.len() {
        return Ok(None);
    }
    let control = *packed.get(*cursor).ok_or(QslError::UnexpectedEof)?;
    let payload = cursor.checked_add(1).ok_or(QslError::LengthOverflow)?;
    let count = if control < 128 {
        control as usize + 1
    } else {
        (control & 127) as usize + 4
    };
    let end = produced
        .checked_add(count)
        .ok_or(QslError::LengthOverflow)?;
    if end > maximum {
        return Err(QslError::InvalidValue);
    }
    let (next, run) = if control < 128 {
        let next = payload.checked_add(count).ok_or(QslError::LengthOverflow)?;
        let bytes = packed.get(payload..next).ok_or(QslError::UnexpectedEof)?;
        (next, CompressedRun::Literal(bytes))
    } else {
        let next = payload.checked_add(width).ok_or(QslError::LengthOverflow)?;
        let bytes = packed.get(payload..next).ok_or(QslError::UnexpectedEof)?;
        let distance = bytes[0] as usize
            + if width == 2 {
                (bytes[1] as usize) << 8
            } else {
                0
            };
        if distance == 0 || distance > produced {
            return Err(QslError::InvalidValue);
        }
        (next, CompressedRun::Match { count, distance })
    };
    *cursor = next;
    Ok(Some(run))
}

pub struct ValidationCursor<'a> {
    input: &'a [u8],
    bit: usize,
    budget: ValueBudget,
}
impl<'a> ValidationCursor<'a> {
    pub fn input_bytes(&self) -> &'a [u8] {
        self.input
    }
    pub fn remaining_budget(&self) -> usize {
        self.budget.remaining
    }
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            bit: 0,
            budget: ValueBudget::new(DEFAULT_MAX_DATA_SIZE),
        }
    }
    /// Reject an oversized frame before examining fields or restoring values.
    pub fn with_limit(input: &'a [u8], maximum: usize) -> QslResult<Self> {
        Self::with_budget(input, maximum, maximum)
    }
    pub fn with_budget(
        input: &'a [u8],
        maximum_frame: usize,
        decoded_budget: usize,
    ) -> QslResult<Self> {
        check_size(input.len(), maximum_frame)?;
        Ok(Self {
            input,
            bit: 0,
            budget: ValueBudget::new(decoded_budget),
        })
    }
    pub fn charge(&mut self, bytes: usize) -> QslResult<()> {
        self.budget.charge(bytes)
    }
    pub fn check_values(&self, bytes: usize, bits: usize) -> QslResult<()> {
        if bytes > self.budget.remaining {
            return Err(QslError::SizeLimitExceeded);
        }
        if self
            .bit
            .checked_add(bits)
            .is_none_or(|n| n > self.input.len().saturating_mul(8))
        {
            return Err(QslError::UnexpectedEof);
        }
        Ok(())
    }
    /// Bound native container storage/work without consuming or aligning wire bits.
    pub fn check_native_allocation(&self, bytes: usize) -> QslResult<()> {
        if bytes > self.budget.remaining {
            Err(QslError::SizeLimitExceeded)
        } else {
            Ok(())
        }
    }
    pub fn check_allocation(&mut self, bytes: usize) -> QslResult<()> {
        self.align()?;
        if bytes > self.budget.remaining {
            return Err(QslError::SizeLimitExceeded);
        }
        if bytes > self.input.len() - self.position() {
            return Err(QslError::UnexpectedEof);
        }
        Ok(())
    }
    pub fn text(&mut self, n: usize, min: usize, max: usize) -> QslResult<&'a str> {
        if n < min || n > max {
            return Err(QslError::InvalidValue);
        }
        core::str::from_utf8(self.payload(n)?).map_err(|_| QslError::InvalidUtf8)
    }

    pub fn payload(&mut self, bytes: usize) -> QslResult<&'a [u8]> {
        self.charge(bytes)?;
        self.bytes(bytes)
    }
    /// Exact LSB-first wide rank; checked chunks never read outside the frame.
    pub fn bits128(&mut self, width: usize) -> QslResult<u128> {
        if width > 128 {
            return Err(QslError::InvalidWidth);
        }
        let low = self.bits(width.min(64))? as u128;
        if width > 64 {
            Ok(low | ((self.bits(width - 64)? as u128) << 64))
        } else {
            Ok(low)
        }
    }
    /// Read a native little-endian integer from an exact byte window.
    pub fn uint128(&mut self, n: usize) -> QslResult<u128> {
        if n > 16 {
            return Err(QslError::InvalidWidth);
        }
        let mut bytes = [0u8; 16];
        bytes[..n].copy_from_slice(self.bytes(n)?);
        Ok(u128::from_le_bytes(bytes))
    }
    pub fn bits(&mut self, width: usize) -> QslResult<u64> {
        if width > 64 {
            return Err(QslError::InvalidWidth);
        }
        let end = self
            .bit
            .checked_add(width)
            .ok_or(QslError::LengthOverflow)?;
        if end > self.input.len().saturating_mul(8) {
            return Err(QslError::UnexpectedEof);
        }
        let start = self.bit >> 3;
        let shift = self.bit & 7;
        let count = (shift + width + 7) >> 3;
        let mut word = [0u8; 16];
        word[..count].copy_from_slice(&self.input[start..start + count]);
        let value = (u128::from_le_bytes(word) >> shift) as u64;
        let mask = ((1u128 << width) - 1) as u64;
        let value = value & mask;
        self.bit = end;
        Ok(value)
    }
    pub fn align(&mut self) -> QslResult<()> {
        let pad = (8 - (self.bit & 7)) & 7;
        if self.bits(pad)? != 0 {
            return Err(QslError::InvalidPadding);
        }
        Ok(())
    }
    pub fn position(&self) -> usize {
        self.bit >> 3
    }
    pub fn bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
        self.align()?;
        let start = self.position();
        let end = start.checked_add(n).ok_or(QslError::LengthOverflow)?;
        let bytes = self.input.get(start..end).ok_or(QslError::UnexpectedEof)?;
        self.bit = end.checked_mul(8).ok_or(QslError::LengthOverflow)?;
        Ok(bytes)
    }
    pub fn uint(&mut self, n: usize) -> QslResult<u64> {
        if n > 8 {
            return Err(QslError::InvalidWidth);
        }
        let mut value = [0u8; 8];
        value[..n].copy_from_slice(self.bytes(n)?);
        Ok(u64::from_le_bytes(value))
    }
    pub fn utf8(&mut self, n: usize, min: usize, max: usize) -> QslResult<()> {
        if n < min || n > max {
            return Err(QslError::InvalidValue);
        }
        core::str::from_utf8(self.payload(n)?).map_err(|_| QslError::InvalidUtf8)?;
        Ok(())
    }
    pub fn finish(mut self) -> QslResult<()> {
        self.align()?;
        if self.position() != self.input.len() {
            return Err(QslError::InvalidValue);
        }
        Ok(())
    }
    /// Restore an owned String using checked QSL-LZv1 / fixed UTF-8 v2 expansion.
    /// Each run is validated and budgeted before reserving its output storage.
    #[cfg(feature = "alloc")]
    pub fn decode_compressed_owned<const MAX: usize>(
        &mut self,
        min: usize,
        raw_only: bool,
        fixed: bool,
    ) -> QslResult<String> {
        let width = if MAX <= 255 { 1 } else { 2 };
        let (stored, compressed, prefix) = if fixed {
            self.align()?;
            if MAX >= crate::STRING_COMPRESSION_MIN_BYTES
                && self.input_bytes().get(self.position()) == Some(&255)
            {
                self.bytes(1)?;
                (self.uint(width)? as usize, true, 1 + width)
            } else {
                (MAX, false, 0)
            }
        } else {
            let header = if MAX <= 127 {
                1
            } else if MAX <= 32767 {
                2
            } else {
                4
            };
            let word = self.uint(header)? as u32;
            let flag = 1u32 << (header * 8 - 1);
            ((word & (flag - 1)) as usize, word & flag != 0, header)
        };
        if !compressed {
            let text = self.text(stored, if fixed { MAX } else { min }, MAX)?;
            let mut out = String::new();
            out.try_reserve_exact(text.len())
                .map_err(|_| QslError::AllocationFailed)?;
            out.push_str(text);
            return Ok(out);
        }
        if raw_only || stored > MAX {
            return Err(QslError::InvalidValue);
        }
        let packed = self.bytes(stored)?;
        let mut output = Vec::<u8>::new();
        let mut cursor = 0;
        while let Some(run) = compressed_run(packed, &mut cursor, output.len(), MAX, width)? {
            let count = run.len();
            self.charge(count)?;
            output
                .try_reserve(count)
                .map_err(|_| QslError::AllocationFailed)?;
            match run {
                CompressedRun::Literal(bytes) => output.extend_from_slice(bytes),
                CompressedRun::Match { count, distance } => {
                    for _ in 0..count {
                        let b = output[output.len() - distance];
                        output.push(b);
                    }
                }
            }
        }
        if output.len() < min || (fixed && output.len() != MAX) || prefix + stored >= output.len() {
            return Err(QslError::InvalidValue);
        }
        String::from_utf8(output).map_err(|_| QslError::InvalidUtf8)
    }
    /// Restore into heap InlineString without a MAX-byte stack temporary.
    /// Each run is checked before writing; invalid UTF-8 is never published.
    #[cfg(feature = "alloc")]
    pub fn decode_compressed_boxed<const MAX: usize, L: crate::storage::LengthWord>(
        &mut self,
        min: usize,
        raw_only: bool,
        fixed: bool,
    ) -> QslResult<alloc::boxed::Box<crate::InlineString<MAX, L>>> {
        let width = if MAX <= 255 { 1 } else { 2 };
        let (stored, compressed, prefix) = if fixed {
            self.align()?;
            if MAX >= crate::STRING_COMPRESSION_MIN_BYTES
                && self.input.get(self.position()) == Some(&255)
            {
                self.bytes(1)?;
                (self.uint(width)? as usize, true, 1 + width)
            } else {
                (MAX, false, 0)
            }
        } else {
            let header = if MAX <= 127 {
                1
            } else if MAX <= 32767 {
                2
            } else {
                4
            };
            let word = self.uint(header)? as u32;
            let flag = 1u32 << (header * 8 - 1);
            ((word & (flag - 1)) as usize, word & flag != 0, header)
        };
        // Reserve the entire fixed heap object once, before alloc. Every decoded
        // prefix is bounded by MAX below, so it fits this already charged region.
        self.charge(core::mem::size_of::<crate::InlineString<MAX, L>>())?;
        if !compressed {
            if stored < if fixed { MAX } else { min } || stored > MAX {
                return Err(QslError::InvalidValue);
            }
            let text =
                core::str::from_utf8(self.bytes(stored)?).map_err(|_| QslError::InvalidUtf8)?;
            return crate::InlineString::try_boxed_from_str(text);
        }
        if raw_only || stored > MAX {
            return Err(QslError::InvalidValue);
        }
        let packed = self.bytes(stored)?;
        let mut cursor = 0;
        // Reject an invalid first token before allocating the fixed-capacity box.
        // Subsequent runs are still parsed exactly once, before each expansion.
        let mut next_run = compressed_run(packed, &mut cursor, 0, MAX, width)?;
        if next_run.is_none() {
            return Err(QslError::InvalidValue);
        }
        let mut output = crate::InlineString::<MAX, L>::try_boxed_new()?;
        let bytes = output.spare_bytes();
        let mut length = 0;
        while let Some(run) = next_run {
            match run {
                CompressedRun::Literal(data) => {
                    for &b in data {
                        bytes[length].write(b);
                        length += 1;
                    }
                }
                CompressedRun::Match { count, distance } => {
                    for _ in 0..count {
                        // compressed_run proves 0 < distance <= initialized prefix length.
                        let b = unsafe { bytes[length - distance].assume_init() };
                        bytes[length].write(b);
                        length += 1;
                    }
                }
            }
            next_run = compressed_run(packed, &mut cursor, length, MAX, width)?;
        }
        if length < min || (fixed && length != MAX) || prefix + stored >= length {
            return Err(QslError::InvalidValue);
        }
        let initialized = unsafe { core::slice::from_raw_parts(bytes.as_ptr().cast(), length) };
        core::str::from_utf8(initialized).map_err(|_| QslError::InvalidUtf8)?;
        unsafe {
            output.publish_len(length);
        }
        Ok(output)
    }
    pub fn compressed<const MAX: usize>(
        &mut self,
        min: usize,
        raw_only: bool,
        fixed: bool,
    ) -> QslResult<()> {
        self.decode_compressed::<MAX>(min, raw_only, fixed)
            .map(|_| ())
    }

    /// Decode once into initialized-prefix storage, validating before exposing UTF-8.
    pub fn decode_compressed<const MAX: usize>(
        &mut self,
        min: usize,
        raw_only: bool,
        fixed: bool,
    ) -> QslResult<crate::InlineString<MAX>> {
        self.decode_compressed_with::<MAX, u32>(min, raw_only, fixed)
    }
    /// Same checked codec with a build-time physical length word; wire stays identical.
    pub fn decode_compressed_with<const MAX: usize, L: crate::storage::LengthWord>(
        &mut self,
        min: usize,
        raw_only: bool,
        fixed: bool,
    ) -> QslResult<crate::InlineString<MAX, L>> {
        let width = if MAX <= 255 { 1 } else { 2 };
        let (stored, compressed, prefix) = if fixed {
            self.align()?;
            if MAX >= crate::STRING_COMPRESSION_MIN_BYTES
                && self.input.get(self.position()) == Some(&255)
            {
                self.bytes(1)?;
                (self.uint(width)? as usize, true, 1 + width)
            } else {
                (MAX, false, 0)
            }
        } else {
            let header = if MAX <= 127 {
                1
            } else if MAX <= 32767 {
                2
            } else {
                4
            };
            let word = self.uint(header)? as u32;
            let flag = 1u32 << (header * 8 - 1);
            ((word & (flag - 1)) as usize, word & flag != 0, header)
        };
        if !compressed {
            let text = self.text(stored, if fixed { MAX } else { min }, MAX)?;
            return Ok(unsafe { crate::InlineString::from_str_unchecked(text) });
        }
        if raw_only || stored > MAX {
            return Err(QslError::InvalidValue);
        }
        let packed = self.bytes(stored)?;
        let mut output = crate::InlineArray::<u8, MAX>::new();
        let mut cursor = 0;
        while let Some(run) =
            compressed_run(packed, &mut cursor, output.len(), output.capacity(), width)?
        {
            self.charge(run.len())?;
            match run {
                CompressedRun::Literal(bytes) => {
                    for &b in bytes {
                        output.push(b)?;
                    }
                }
                CompressedRun::Match { count, distance } => {
                    for _ in 0..count {
                        let b = output[output.len() - distance];
                        output.push(b)?;
                    }
                }
            }
        }
        if output.len() < min || (fixed && output.len() != MAX) || prefix + stored >= output.len() {
            return Err(QslError::InvalidValue);
        }
        let text = core::str::from_utf8(&output).map_err(|_| QslError::InvalidUtf8)?;
        Ok(unsafe { crate::InlineString::from_str_unchecked(text) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use std::{prelude::v1::*, vec};
    #[test]
    fn native_bit_windows_match_a_bit_oracle_at_every_offset_and_width() {
        let bytes = [0x37, 0xa5, 0xff, 0, 0x13, 0x80, 0xab, 0xcd, 0xf9];
        for start in 0..8 {
            for width in 0..=64 {
                let mut c = ValidationCursor::with_limit(&bytes, 65535).unwrap();
                c.bits(start).unwrap();
                let mut expected = 0u64;
                for bit in 0..width {
                    expected |=
                        u64::from((bytes[(start + bit) >> 3] >> ((start + bit) & 7)) & 1) << bit;
                }
                assert_eq!(c.bits(width).unwrap(), expected);
                assert_eq!(c.bit, start + width);
            }
        }
        let mut c = ValidationCursor::with_limit(&[], 1).unwrap();
        assert_eq!(c.bits(0), Ok(0));
        assert_eq!(c.bits(1), Err(QslError::UnexpectedEof));
        assert!(matches!(
            ValidationCursor::with_limit(&bytes, 8),
            Err(QslError::SizeLimitExceeded)
        ));
    }
    #[test]
    fn budgets_fail_on_sum_and_declared_lengths_before_data_access() {
        let mut b = ValueBudget::new(10);
        b.charge(7).unwrap();
        assert_eq!(b.charge(4), Err(QslError::SizeLimitExceeded));
        let mut c = ValidationCursor::with_limit(&[], 65535).unwrap();
        assert_eq!(
            c.utf8(u32::MAX as usize, 0, usize::MAX),
            Err(QslError::SizeLimitExceeded)
        );
    }
    #[test]
    fn expanded_runs_check_destination_and_arithmetic_before_advancing() {
        let mut cursor = 0;
        assert!(matches!(
            compressed_run(&[0, b'A'], &mut cursor, usize::MAX, usize::MAX, 1),
            Err(QslError::LengthOverflow)
        ));
        assert_eq!(cursor, 0);
        assert!(matches!(
            compressed_run(&[0, b'A'], &mut cursor, 100, 100, 1),
            Err(QslError::InvalidValue)
        ));
        assert_eq!(cursor, 0);
        assert!(matches!(
            compressed_run(&[0xFF, 1], &mut cursor, 1, 100, 1),
            Err(QslError::InvalidValue)
        ));
        assert_eq!(cursor, 0);
    }

    #[test]
    fn lz_budget_charges_expansion_instead_of_the_forged_stored_length() {
        // Four compressed bytes reconstruct exactly 100 A bytes.
        let frame = [0x84, 0, b'A', 0xDF, 1];
        let mut exact = ValidationCursor::with_budget(&frame, frame.len(), 100).unwrap();
        assert_eq!(
            exact
                .decode_compressed::<100>(0, false, false)
                .unwrap()
                .len(),
            100
        );
        exact.finish().unwrap();
        for budget in [0, 1, 4, 99] {
            let mut inline = ValidationCursor::with_budget(&frame, frame.len(), budget).unwrap();
            assert!(matches!(
                inline.decode_compressed::<100>(0, false, false),
                Err(QslError::SizeLimitExceeded)
            ));
        }
        // Encoded metadata fits, but the last token would reconstruct 132 B.
        let forged = [0x84, 0, b'A', 0xFF, 1];
        let mut c = ValidationCursor::new(&forged);
        assert!(matches!(
            c.decode_compressed::<100>(0, false, false),
            Err(QslError::InvalidValue)
        ));
    }

    #[test]
    fn invalid_input_has_no_pointer_access_or_panic() {
        for n in 0..256 {
            let bytes = vec![n as u8; n];
            let _ = ValidationCursor::new(&bytes).compressed::<255>(0, false, false);
        }
        assert!(ValidationCursor::new(&[0x84, 0, b'A', 0xdf, 0])
            .compressed::<100>(0, false, false)
            .is_err());
        assert!(ValidationCursor::new(&[0x84, 0, b'A', 0xdf, 1])
            .compressed::<100>(0, false, false)
            .is_ok());
    }
}

#[cfg(all(test, feature = "alloc"))]
mod owned_tests {
    use super::*;
    #[test]
    fn owned_lz_budget_and_forged_length_are_checked_before_growth() {
        let frame = [0x84, 0, b'A', 0xDF, 1];
        let mut c = ValidationCursor::with_budget(&frame, frame.len(), 100).unwrap();
        assert_eq!(
            c.decode_compressed_owned::<100>(0, false, false).unwrap(),
            "A".repeat(100)
        );
        c.finish().unwrap();
        for budget in [0, 1, 4, 99] {
            let mut c = ValidationCursor::with_budget(&frame, frame.len(), budget).unwrap();
            assert_eq!(
                c.decode_compressed_owned::<100>(0, false, false),
                Err(QslError::SizeLimitExceeded)
            );
        }
        let mut c = ValidationCursor::new(&[0x84, 0, b'A', 0xFF, 1]);
        assert_eq!(
            c.decode_compressed_owned::<100>(0, false, false),
            Err(QslError::InvalidValue)
        );
    }
    #[test]
    fn heap_inline_capacity_is_checked_before_allocating_even_for_short_values() {
        let input = [1, b'a'];
        let capacity = core::mem::size_of::<crate::InlineString<100>>();
        let mut cursor = ValidationCursor::with_budget(&input, 1024, capacity - 1).unwrap();
        assert!(matches!(
            cursor.decode_compressed_boxed::<100, u32>(0, false, false),
            Err(QslError::SizeLimitExceeded)
        ));
        let mut cursor = ValidationCursor::with_budget(&input, 1024, capacity).unwrap();
        assert_eq!(
            cursor
                .decode_compressed_boxed::<100, u32>(0, false, false)
                .unwrap()
                .as_str(),
            "a"
        );
        let mut cursor = ValidationCursor::with_budget(&input, 1024, 1).unwrap();
        assert_eq!(
            cursor
                .decode_compressed_owned::<100>(0, false, false)
                .unwrap(),
            "a"
        );
    }
    #[test]
    fn fixed_heap_budget_is_aggregate_and_reserved_once_for_raw_and_lz() {
        let capacity = core::mem::size_of::<crate::InlineString<100>>();
        let raw = [1, b'a', 1, b'b'];
        let mut cursor = ValidationCursor::with_budget(&raw, 1024, 2 * capacity - 1).unwrap();
        assert_eq!(
            cursor
                .decode_compressed_boxed::<100, u32>(0, false, false)
                .unwrap()
                .as_str(),
            "a"
        );
        assert_eq!(cursor.remaining_budget(), capacity - 1);
        assert!(matches!(
            cursor.decode_compressed_boxed::<100, u32>(0, false, false),
            Err(QslError::SizeLimitExceeded)
        ));
        let mut cursor = ValidationCursor::with_budget(&raw, 1024, 2 * capacity).unwrap();
        for expected in ["a", "b"] {
            assert_eq!(
                cursor
                    .decode_compressed_boxed::<100, u32>(0, false, false)
                    .unwrap()
                    .as_str(),
                expected
            );
        }
        assert_eq!(cursor.remaining_budget(), 0);
        cursor.finish().unwrap();
        let packed = [0x84, 0, b'A', 0xdf, 1];
        let mut cursor = ValidationCursor::with_budget(&packed, 1024, capacity).unwrap();
        assert_eq!(
            cursor
                .decode_compressed_boxed::<100, u32>(0, false, false)
                .unwrap()
                .len(),
            100
        );
        assert_eq!(cursor.remaining_budget(), 0);
        cursor.finish().unwrap();
    }
}
