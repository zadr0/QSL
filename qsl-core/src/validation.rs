//! Checked wire validation and single-pass materialization.
//! Compressed output is bounded independently of the untrusted stored length.
//! Heap restoration is fallible; trusted pointer APIs remain separate.
use crate::{QslError, QslResult};

pub use crate::config::DEFAULT_MAX_DATA_SIZE;

pub fn check_size(size: usize, maximum: usize) -> QslResult<()> {
    if size > maximum || size > isize::MAX as usize {
        return Err(QslError::SizeLimitExceeded);
    }
    Ok(())
}

/// Aggregate decoded value bytes, independent of compressed frame size.
/// This bounds data, not allocator metadata or the process working set.
pub struct ValueBudget {
    remaining: usize,
}
impl ValueBudget {
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
enum CompressedRun<'a> {
    Literal(&'a [u8]),
    Match { count: usize, distance: usize },
}
impl CompressedRun<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Literal(bytes) => bytes.len(),
            Self::Match { count, .. } => *count,
        }
    }
}

/// The wire header bounds packed input only. Every run must independently fit
/// the actual destination bound before any reserve, backreference or write.
fn compressed_run<'a>(
    packed: &'a [u8],
    cursor: &mut usize,
    produced: usize,
    maximum: usize,
    width: usize,
) -> QslResult<Option<CompressedRun<'a>>> {
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
        std::str::from_utf8(self.payload(n)?).map_err(|_| QslError::InvalidUtf8)
    }

    pub fn payload(&mut self, bytes: usize) -> QslResult<&'a [u8]> {
        self.charge(bytes)?;
        self.bytes(bytes)
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
        std::str::from_utf8(self.payload(n)?).map_err(|_| QslError::InvalidUtf8)?;
        Ok(())
    }
    pub fn finish(mut self) -> QslResult<()> {
        self.align()?;
        if self.position() != self.input.len() {
            return Err(QslError::InvalidValue);
        }
        Ok(())
    }
    /// Validate QSL-LZv1 / fixed UTF-8 v2 by safe token expansion into bounded
    /// stack storage. No speculative pointer reads and no heap allocation.
    pub fn compressed<const MAX: usize>(
        &mut self,
        min: usize,
        raw_only: bool,
        fixed: bool,
    ) -> QslResult<()> {
        self.decode_compressed::<MAX>(min, raw_only, fixed)
            .map(|_| ())
    }
    /// Heap fallback for a string above STACK_LIMIT. Growth is fallible and
    /// charged before every decoded run; the resulting Vec becomes String.
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
    /// Decode once into initialized-prefix storage, validating before exposing UTF-8.
    pub fn decode_compressed<const MAX: usize>(
        &mut self,
        min: usize,
        raw_only: bool,
        fixed: bool,
    ) -> QslResult<crate::InlineString<MAX>> {
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
        let text = std::str::from_utf8(&output).map_err(|_| QslError::InvalidUtf8)?;
        Ok(unsafe { crate::InlineString::from_str_unchecked(text) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            let mut heap = ValidationCursor::with_budget(&frame, frame.len(), budget).unwrap();
            assert!(matches!(
                heap.decode_compressed_owned::<100>(0, false, false),
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
