//! Versioned dyn v1 framing. Index addresses are zero-based MSB-first bits.
//! Validation borrows caller scratch for duplicate tags; it allocates nothing.
use crate::{QslError as E, QslResult as R};
pub const DYN_HEADER_BYTES: usize = 12;
const MAGIC: [u8; 4] = *b"QDY1";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DynMode {
    Line = 0,
    StrictIndex = 1,
    DynamicIndex = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DynWidth {
    W16 = 16,
    W32 = 32,
    W64 = 64,
}
impl DynWidth {
    pub const fn bytes(self) -> usize {
        self as usize / 8
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynHeader {
    pub mode: DynMode,
    pub width: DynWidth,
    pub revision: u32,
}
impl DynHeader {
    pub const fn tag_limit(self) -> usize {
        match (self.mode, self.width) {
            (DynMode::Line, DynWidth::W16)
            | (DynMode::StrictIndex, DynWidth::W32)
            | (DynMode::DynamicIndex, DynWidth::W64) => u16::MAX as usize,
            (DynMode::DynamicIndex, DynWidth::W32) => u8::MAX as usize,
            (DynMode::Line, DynWidth::W64) => usize::MAX,
            _ => u32::MAX as usize,
        }
    }

    pub fn validate(self) -> R<()> {
        if self.mode != DynMode::Line && self.width == DynWidth::W16 {
            return Err(E::InvalidWidth);
        }
        Ok(())
    }
    pub fn read(frame: &[u8], limit: usize) -> R<Self> {
        if frame.len() > limit {
            return Err(E::SizeLimitExceeded);
        }
        let h = frame.get(..DYN_HEADER_BYTES).ok_or(E::UnexpectedEof)?;
        if h[..4] != MAGIC || h[6..8] != [0, 0] {
            return Err(E::InvalidValue);
        }
        let mode = match h[4] {
            0 => DynMode::Line,
            1 => DynMode::StrictIndex,
            2 => DynMode::DynamicIndex,
            _ => return Err(E::InvalidValue),
        };
        let width = match h[5] {
            16 => DynWidth::W16,
            32 => DynWidth::W32,
            64 => DynWidth::W64,
            _ => return Err(E::InvalidWidth),
        };
        let out = Self {
            mode,
            width,
            revision: u32::from_le_bytes(h[8..12].try_into().unwrap()),
        };
        out.validate()?;
        Ok(out)
    }
    pub fn write(self, out: &mut [u8]) -> R<()> {
        self.validate()?;
        let h = out.get_mut(..DYN_HEADER_BYTES).ok_or(E::BufferTooSmall)?;
        h[..4].copy_from_slice(&MAGIC);
        h[4] = self.mode as u8;
        h[5] = self.width as u8;
        h[6..8].fill(0);
        h[8..12].copy_from_slice(&self.revision.to_le_bytes());
        Ok(())
    }
}
fn word(bytes: &[u8]) -> u64 {
    let mut x = [0u8; 8];
    x[..bytes.len()].copy_from_slice(bytes);
    u64::from_le_bytes(x)
}
fn write_word(out: &mut [u8], value: u64) -> R<()> {
    if out.len() < 8 && value >= (1u64 << (out.len() * 8)) {
        return Err(E::LengthOverflow);
    }
    out.copy_from_slice(&value.to_le_bytes()[..out.len()]);
    Ok(())
}
fn unique(tags: &mut [u32]) -> R<()> {
    tags.sort_unstable();
    if tags.windows(2).any(|w| w[0] == w[1]) {
        Err(E::InvalidValue)
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynEntry {
    pub offset_bits: u32,
    pub tag: u32,
    pub size_bits: u32,
}

/// A validated field extent. `size_bits` is only the header width for a dynamic value.
#[derive(Clone, Copy, Debug)]
pub struct DynField<'a> {
    payload: &'a [u8],
    pub entry: DynEntry,
    end_bits: usize,
    strict: bool,
}
impl<'a> DynField<'a> {
    pub fn byte_extent(&self) -> R<&'a [u8]> {
        if !self.entry.offset_bits.is_multiple_of(8) || !self.end_bits.is_multiple_of(8) {
            return Err(E::InvalidWidth);
        }
        Ok(&self.payload[self.entry.offset_bits as usize / 8..self.end_bits / 8])
    }
    pub fn has_size(&self) -> bool {
        !self.strict
    }

    pub fn extent_bits(&self) -> usize {
        self.end_bits - self.entry.offset_bits as usize
    }
    /// Read an integer rank from at most 128 bits, independent of host endian.
    pub fn unsigned(&self, width: usize) -> R<u128> {
        if width > 128
            || width > self.extent_bits()
            || (!self.strict && width != self.entry.size_bits as usize)
        {
            return Err(E::InvalidWidth);
        }
        let mut x = 0u128;
        let start = self.entry.offset_bits as usize;
        for i in start..start + width {
            x = (x << 1) | u128::from((self.payload[i / 8] >> (7 - i % 8)) & 1);
        }
        Ok(x)
    }

    /// # Safety
    /// width <= 128 and the entire requested initialized bit span belongs to this field.
    pub unsafe fn unsigned_unchecked(&self, width: usize) -> u128 {
        let mut x = 0u128;
        let start = self.entry.offset_bits as usize;
        for i in start..start + width {
            x = (x << 1) | u128::from((*self.payload.get_unchecked(i / 8) >> (7 - i % 8)) & 1);
        }
        x
    }
    /// A known compact scalar may be followed only by zero byte-alignment padding.
    pub fn validate_scalar_tail(&self, width: usize) -> R<()> {
        if width > self.extent_bits() || self.extent_bits() - width > 7 {
            return Err(E::InvalidValue);
        }
        for i in self.entry.offset_bits as usize + width..self.end_bits {
            if (self.payload[i / 8] >> (7 - i % 8)) & 1 != 0 {
                return Err(E::InvalidPadding);
            }
        }
        Ok(())
    }
    fn dynamic_payload(&self, header_bytes: usize) -> R<(usize, &'a [u8])> {
        if !matches!(header_bytes, 1 | 2 | 4)
            || !self.entry.offset_bits.is_multiple_of(8)
            || (!self.strict && self.entry.size_bits as usize != header_bytes * 8)
        {
            return Err(E::InvalidWidth);
        }
        let start = self.entry.offset_bits as usize / 8;
        let header_end = start.checked_add(header_bytes).ok_or(E::LengthOverflow)?;
        if header_end * 8 > self.end_bits {
            return Err(E::UnexpectedEof);
        }
        let count = word(&self.payload[start..header_end]) as usize;
        Ok((count, &self.payload[header_end..self.end_bits / 8]))
    }
    /// Length is the count of UTF-8 bytes, not Unicode scalar values.
    pub fn utf8(&self, header_bytes: usize, min: usize, max: usize) -> R<&'a str> {
        let (n, tail) = self.dynamic_payload(header_bytes)?;
        if n < min || n > max {
            return Err(E::SizeLimitExceeded);
        }
        core::str::from_utf8(tail.get(..n).ok_or(E::UnexpectedEof)?).map_err(|_| E::InvalidUtf8)
    }
    /// Return encoded array bytes; the count is elements, never a byte count.
    pub fn array(
        &self,
        header_bytes: usize,
        element_bits: usize,
        min: usize,
        max: usize,
    ) -> R<(usize, &'a [u8])> {
        let (n, tail) = self.dynamic_payload(header_bytes)?;
        if n < min || n > max {
            return Err(E::SizeLimitExceeded);
        }
        let bits = n.checked_mul(element_bits).ok_or(E::LengthOverflow)?;
        let bytes = bits.checked_add(7).ok_or(E::LengthOverflow)? / 8;
        Ok((n, tail.get(..bytes).ok_or(E::UnexpectedEof)?))
    }
}

/// Zero-copy index view; constructing it checks all table boundaries and tags once.
#[derive(Clone, Copy, Debug)]
pub struct DynIndexView<'a> {
    pub header: DynHeader,
    payload: &'a [u8],
    table: &'a [u8],
}
impl<'a> DynIndexView<'a> {
    /// # Safety
    /// Frame has the stated header and a structurally valid, unique, in-bounds index.
    /// Every field read through this view must separately satisfy its schema codec.
    pub unsafe fn from_validated_unchecked(frame: &'a [u8], header: DynHeader) -> Self {
        let trailer = frame.len() - 4;
        let back = u32::from_le(core::ptr::read_unaligned(
            frame.as_ptr().add(trailer).cast::<u32>(),
        )) as usize;
        let start = trailer - back;
        Self {
            header,
            payload: frame.get_unchecked(DYN_HEADER_BYTES..start),
            table: frame.get_unchecked(start..trailer),
        }
    }

    /// Scratch holds one u32 per entry and is reusable immediately after validation.
    pub fn parse(frame: &'a [u8], limit: usize, tags: &mut [u32]) -> R<Self> {
        let header = DynHeader::read(frame, limit)?;
        if header.mode == DynMode::Line {
            return Err(E::InvalidSchema);
        }
        let trailer = frame
            .len()
            .checked_sub(4)
            .filter(|&n| n >= DYN_HEADER_BYTES)
            .ok_or(E::UnexpectedEof)?;
        let back = u32::from_le_bytes(frame[trailer..].try_into().unwrap()) as usize;
        let table_start = trailer
            .checked_sub(back)
            .filter(|&n| n >= DYN_HEADER_BYTES)
            .ok_or(E::InvalidValue)?;
        let table = &frame[table_start..trailer];
        let entry_bytes = header.width.bytes();
        if !table.len().is_multiple_of(entry_bytes) {
            return Err(E::InvalidValue);
        }
        let out = Self {
            header,
            payload: &frame[DYN_HEADER_BYTES..table_start],
            table,
        };
        let payload_bits = out.payload.len().checked_mul(8).ok_or(E::LengthOverflow)?;
        let n = out.len();
        let tags = tags.get_mut(..n).ok_or(E::BufferTooSmall)?;
        for (i, tag) in tags.iter_mut().enumerate() {
            let entry = out.entry(i);
            *tag = entry.tag;
            if entry.tag == 0 {
                return Err(E::InvalidValue);
            }
            let start = entry.offset_bits as usize;
            let end = if i + 1 < n {
                out.entry(i + 1).offset_bits as usize
            } else {
                payload_bits
            };
            if start > end
                || end > payload_bits
                || (header.mode == DynMode::DynamicIndex && entry.size_bits as usize > end - start)
            {
                return Err(E::InvalidValue);
            }
        }
        unique(tags)?;
        Ok(out)
    }
    pub fn len(&self) -> usize {
        self.table.len() / self.header.width.bytes()
    }
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
    fn entry(&self, i: usize) -> DynEntry {
        let n = self.header.width.bytes();
        let b = &self.table[i * n..(i + 1) * n];
        let half = n / 2;
        let offset_bits = word(&b[..half]) as u32;
        let (tag, size_bits) = if self.header.mode == DynMode::StrictIndex {
            (word(&b[half..]) as u32, 0)
        } else {
            (
                word(&b[half..half + half / 2]) as u32,
                word(&b[half + half / 2..]) as u32,
            )
        };
        DynEntry {
            offset_bits,
            tag,
            size_bits,
        }
    }
    pub fn at(&self, i: usize) -> Option<DynField<'a>> {
        if i >= self.len() {
            return None;
        }
        let end_bits = if i + 1 < self.len() {
            self.entry(i + 1).offset_bits as usize
        } else {
            self.payload.len() * 8
        };
        Some(DynField {
            payload: self.payload,
            entry: self.entry(i),
            end_bits,
            strict: self.header.mode == DynMode::StrictIndex,
        })
    }
    pub fn find(&self, tag: u32) -> Option<DynField<'a>> {
        (0..self.len()).find_map(|i| self.at(i).filter(|f| f.entry.tag == tag))
    }
}

/// Encode a caller-prepared bit payload and index into an existing output buffer.
/// Values are validated before narrowing; an error may leave partially written output.
pub fn write_indexed(
    header: DynHeader,
    payload: &[u8],
    entries: &[DynEntry],
    out: &mut [u8],
    limit: usize,
    tags: &mut [u32],
) -> R<usize> {
    header.validate()?;
    if header.mode == DynMode::Line {
        return Err(E::InvalidSchema);
    }
    let table_bytes = entries
        .len()
        .checked_mul(header.width.bytes())
        .ok_or(E::LengthOverflow)?;
    let n = DYN_HEADER_BYTES
        .checked_add(payload.len())
        .and_then(|n| n.checked_add(table_bytes))
        .and_then(|n| n.checked_add(4))
        .ok_or(E::LengthOverflow)?;
    if n > limit {
        return Err(E::SizeLimitExceeded);
    }
    let out = out.get_mut(..n).ok_or(E::BufferTooSmall)?;
    header.write(out)?;
    out[DYN_HEADER_BYTES..DYN_HEADER_BYTES + payload.len()].copy_from_slice(payload);
    let start = DYN_HEADER_BYTES + payload.len();
    let width = header.width.bytes();
    for (i, e) in entries.iter().enumerate() {
        let row = &mut out[start + i * width..start + (i + 1) * width];
        let half = width / 2;
        write_word(&mut row[..half], e.offset_bits as u64)?;
        if header.mode == DynMode::StrictIndex {
            write_word(&mut row[half..], e.tag as u64)?;
        } else {
            write_word(&mut row[half..half + half / 2], e.tag as u64)?;
            write_word(&mut row[half + half / 2..], e.size_bits as u64)?;
        }
    }
    let back = u32::try_from(table_bytes).map_err(|_| E::LengthOverflow)?;
    out[n - 4..].copy_from_slice(&back.to_le_bytes());
    DynIndexView::parse(out, limit, tags)?;
    Ok(n)
}

#[derive(Clone, Copy, Debug)]
pub struct DynLineField<'a> {
    pub tag: u64,
    pub bytes: &'a [u8],
}
#[derive(Clone, Copy, Debug)]
pub struct DynLineView<'a> {
    pub header: DynHeader,
    body: &'a [u8],
    count: usize,
}
impl<'a> DynLineView<'a> {
    /// # Safety
    /// Frame has the stated header and consists of complete unique nonzero-tag line records.
    pub unsafe fn from_validated_unchecked(frame: &'a [u8], header: DynHeader) -> Self {
        let body = frame.get_unchecked(DYN_HEADER_BYTES..);
        let width = header.width.bytes();
        let mut pos = 0;
        let mut count = 0;
        while pos < body.len() {
            let n = word(body.get_unchecked(pos + width..pos + 2 * width)) as usize;
            pos += 2 * width + n;
            count += 1;
        }
        Self {
            header,
            body,
            count,
        }
    }

    pub fn parse(frame: &'a [u8], limit: usize, tags: &mut [u64]) -> R<Self> {
        let header = DynHeader::read(frame, limit)?;
        if header.mode != DynMode::Line {
            return Err(E::InvalidSchema);
        }
        let width = header.width.bytes();
        let mut pos = DYN_HEADER_BYTES;
        let mut count = 0;
        while pos < frame.len() {
            let h = frame.get(pos..pos + 2 * width).ok_or(E::UnexpectedEof)?;
            let tag = word(&h[..width]);
            let n = usize::try_from(word(&h[width..])).map_err(|_| E::LengthOverflow)?;
            if tag == 0 {
                return Err(E::InvalidValue);
            }
            *tags.get_mut(count).ok_or(E::BufferTooSmall)? = tag;
            pos = pos
                .checked_add(2 * width)
                .and_then(|p| p.checked_add(n))
                .filter(|&p| p <= frame.len())
                .ok_or(E::UnexpectedEof)?;
            count += 1;
        }
        tags[..count].sort_unstable();
        if tags[..count].windows(2).any(|w| w[0] == w[1]) {
            return Err(E::InvalidValue);
        }
        Ok(Self {
            header,
            body: &frame[DYN_HEADER_BYTES..],
            count,
        })
    }
    pub fn len(&self) -> usize {
        self.count
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn fields(&self) -> impl Iterator<Item = DynLineField<'a>> {
        let width = self.header.width.bytes();
        let mut tail = self.body;
        core::iter::from_fn(move || {
            if tail.is_empty() {
                return None;
            }
            let tag = word(&tail[..width]);
            let n = word(&tail[width..2 * width]) as usize;
            let bytes = &tail[2 * width..2 * width + n];
            tail = &tail[2 * width + n..];
            Some(DynLineField { tag, bytes })
        })
    }
}
pub fn write_line(
    header: DynHeader,
    fields: &[DynLineField<'_>],
    out: &mut [u8],
    limit: usize,
    tags: &mut [u64],
) -> R<usize> {
    header.validate()?;
    if header.mode != DynMode::Line {
        return Err(E::InvalidSchema);
    }
    let width = header.width.bytes();
    let n = fields.iter().try_fold(DYN_HEADER_BYTES, |n, f| {
        n.checked_add(2 * width)
            .and_then(|n| n.checked_add(f.bytes.len()))
            .ok_or(E::LengthOverflow)
    })?;
    if n > limit {
        return Err(E::SizeLimitExceeded);
    }
    let out = out.get_mut(..n).ok_or(E::BufferTooSmall)?;
    header.write(out)?;
    let mut pos = DYN_HEADER_BYTES;
    for field in fields {
        write_word(&mut out[pos..pos + width], field.tag)?;
        write_word(
            &mut out[pos + width..pos + 2 * width],
            field.bytes.len() as u64,
        )?;
        pos += 2 * width;
        out[pos..pos + field.bytes.len()].copy_from_slice(field.bytes);
        pos += field.bytes.len();
    }
    DynLineView::parse(out, limit, tags)?;
    Ok(n)
}

/// Exact MSB-first reference kernel for dyn compact scalar fields.
/// It initializes each touched byte and never reads or writes a rounded-up word.
pub struct DynBitWriter {
    base: *mut u8,
    bit: usize,
}
impl DynBitWriter {
    /// # Safety
    /// base addresses the full exclusive writable encoded payload capacity.
    pub unsafe fn from_raw(base: *mut u8) -> Self {
        Self { base, bit: 0 }
    }
    pub fn bit_position(&self) -> usize {
        self.bit
    }
    pub fn align(&mut self) {
        self.bit = (self.bit + 7) & !7;
    }
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.base.wrapping_add(self.bit / 8)
    }
    /// # Safety
    /// n initialized bytes were just written at the byte-aligned cursor.
    pub unsafe fn claim_initialized_bytes(&mut self, n: usize) {
        self.bit += n * 8;
    }
    /// # Safety
    /// width<=128, value fits width, and ceil((bit+width)/8) bytes fit the output.
    pub unsafe fn write_unsigned_unchecked(&mut self, value: u128, width: usize) {
        let mut left = width;
        while left != 0 {
            let shift = self.bit % 8;
            let count = (8 - shift).min(left);
            left -= count;
            let mask = ((1u16 << count) - 1) as u128;
            let part = (((value >> left) & mask) as u8) << (8 - shift - count);
            let ptr = self.base.add(self.bit / 8);
            if shift == 0 {
                ptr.write(part);
            } else {
                ptr.write(ptr.read() | part);
            }
            self.bit += count;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use std::{prelude::v1::*, vec};
    #[test]
    fn all_scalar_widths_cross_exact_windows_without_touching_canaries() {
        let mut seed = 0x1234_5678_9abc_def0u128;
        for prefix in 0usize..8 {
            for width in 0usize..=128 {
                for _ in 0..8 {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    let value = seed & crate::domain::low_mask(width as u16);
                    let mut output = [0xa5u8; 24];
                    let mut writer = unsafe { DynBitWriter::from_raw(output.as_mut_ptr().add(1)) };
                    unsafe {
                        writer.write_unsigned_unchecked(
                            crate::domain::low_mask(prefix as u16),
                            prefix,
                        );
                        writer.write_unsigned_unchecked(value, width);
                    }
                    assert_eq!(writer.bit_position(), prefix + width);
                    writer.align();
                    let size = (prefix + width).div_ceil(8);
                    let mut expected = vec![0u8; size];
                    for bit in 0..prefix + width {
                        let one = if bit < prefix {
                            true
                        } else {
                            (value >> (width - 1 - (bit - prefix))) & 1 != 0
                        };
                        if one {
                            expected[bit / 8] |= 1 << (7 - bit % 8);
                        }
                    }
                    assert_eq!(&output[1..1 + size], expected);
                    assert_eq!(output[0], 0xa5);
                    assert!(output[1 + size..].iter().all(|&x| x == 0xa5));
                }
            }
        }
    }
    fn header(mode: DynMode, width: DynWidth) -> DynHeader {
        DynHeader {
            mode,
            width,
            revision: 1,
        }
    }
    #[test]
    fn index32_golden_msb_utf8_and_unknown_skip() {
        let payload = [0, 0, 0, 0, 0xa0, 3, 0xd0, 0xaf, b'a'];
        let entries = [
            DynEntry {
                offset_bits: 0,
                tag: 1,
                size_bits: 32,
            },
            DynEntry {
                offset_bits: 32,
                tag: 2,
                size_bits: 4,
            },
            DynEntry {
                offset_bits: 40,
                tag: 9,
                size_bits: 8,
            },
        ];
        let mut out = [0u8; 64];
        let mut scratch = [0u32; 3];
        let n = write_indexed(
            header(DynMode::DynamicIndex, DynWidth::W32),
            &payload,
            &entries,
            &mut out,
            64,
            &mut scratch,
        )
        .unwrap();
        assert_eq!(&out[..12], b"QDY1\x02\x20\0\0\x01\0\0\0");
        assert_eq!(
            &out[21..n],
            &[0, 0, 1, 32, 32, 0, 2, 4, 40, 0, 9, 8, 12, 0, 0, 0]
        );
        let view = DynIndexView::parse(&out[..n], 64, &mut scratch).unwrap();
        assert_eq!(view.find(2).unwrap().unsigned(4).unwrap(), 10);
        assert_eq!(view.find(9).unwrap().utf8(1, 0, 10).unwrap(), "Яa");
        assert!(view.find(7).is_none());
    }
    #[test]
    fn strict_and_dynamic_widths_and_zero_bit_fields() {
        for mode in [DynMode::StrictIndex, DynMode::DynamicIndex] {
            for width in [DynWidth::W32, DynWidth::W64] {
                let mut out = [0u8; 64];
                let mut scratch = [0u32; 3];
                let entries = [
                    DynEntry {
                        offset_bits: 0,
                        tag: 3,
                        size_bits: 0,
                    },
                    DynEntry {
                        offset_bits: 0,
                        tag: 1,
                        size_bits: 4,
                    },
                    DynEntry {
                        offset_bits: 4,
                        tag: 2,
                        size_bits: 4,
                    },
                ];
                let n = write_indexed(
                    header(mode, width),
                    &[0xa7],
                    &entries,
                    &mut out,
                    64,
                    &mut scratch,
                )
                .unwrap();
                let view = DynIndexView::parse(&out[..n], 64, &mut scratch).unwrap();
                assert_eq!(view.find(3).unwrap().unsigned(0).unwrap(), 0);
                assert_eq!(view.find(1).unwrap().unsigned(4).unwrap(), 10);
                assert_eq!(view.find(2).unwrap().unsigned(4).unwrap(), 7);
            }
        }
    }
    #[test]
    fn rejects_malformed_indexes_and_dynamic_overflows() {
        let mut out = [0u8; 64];
        let mut tags = [0u32; 3];
        let h = header(DynMode::DynamicIndex, DynWidth::W32);
        for entries in [
            vec![DynEntry {
                offset_bits: 0,
                tag: 0,
                size_bits: 8,
            }],
            vec![DynEntry {
                offset_bits: 0,
                tag: 1,
                size_bits: 9,
            }],
            vec![DynEntry {
                offset_bits: 9,
                tag: 1,
                size_bits: 0,
            }],
            vec![
                DynEntry {
                    offset_bits: 0,
                    tag: 1,
                    size_bits: 0,
                },
                DynEntry {
                    offset_bits: 0,
                    tag: 1,
                    size_bits: 8,
                },
            ],
            vec![
                DynEntry {
                    offset_bits: 4,
                    tag: 2,
                    size_bits: 1,
                },
                DynEntry {
                    offset_bits: 0,
                    tag: 3,
                    size_bits: 1,
                },
            ],
        ] {
            assert!(write_indexed(h, &[0], &entries, &mut out, 64, &mut tags).is_err());
        }
        let n = write_indexed(
            h,
            &[255, 0],
            &[DynEntry {
                offset_bits: 0,
                tag: 1,
                size_bits: 8,
            }],
            &mut out,
            64,
            &mut tags,
        )
        .unwrap();
        let v = DynIndexView::parse(&out[..n], 64, &mut tags).unwrap();
        assert_eq!(v.find(1).unwrap().utf8(1, 0, 65535), Err(E::UnexpectedEof));
        assert!(v.find(1).unwrap().array(1, usize::MAX, 0, 65535).is_err());
        assert!(DynIndexView::parse(&out[..n], n - 1, &mut tags).is_err());
        out[n - 4..n].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(DynIndexView::parse(&out[..n], 64, &mut tags).is_err());
        assert!(header(DynMode::DynamicIndex, DynWidth::W16)
            .validate()
            .is_err());
    }
    #[test]
    fn line_golden_and_every_truncation() {
        for width in [DynWidth::W16, DynWidth::W32, DynWidth::W64] {
            let mut out = [0u8; 128];
            let mut tags = [0u64; 3];
            let h = header(DynMode::Line, width);
            let n = write_line(
                h,
                &[
                    DynLineField {
                        tag: 5,
                        bytes: &[3, 0xd0, 0xaf, b'a'],
                    },
                    DynLineField {
                        tag: 200,
                        bytes: &[7],
                    },
                ],
                &mut out,
                128,
                &mut tags,
            )
            .unwrap();
            let view = DynLineView::parse(&out[..n], 128, &mut tags).unwrap();
            assert_eq!(view.fields().map(|x| x.tag).collect::<Vec<_>>(), [5, 200]);
            assert_eq!(view.fields().next().unwrap().bytes, &[3, 0xd0, 0xaf, b'a']);
            for end in 0..n {
                let result = DynLineView::parse(&out[..end], 128, &mut tags);
                if result.is_ok() {
                    assert!(end == 12 || end == 12 + 2 * width.bytes() + 4);
                }
            }
        }
    }
    #[test]
    fn arbitrary_frames_never_panic() {
        let mut state = 71u64;
        let mut data = [0u8; 128];
        let mut tags = [0u32; 32];
        let mut line_tags = [0u64; 32];
        for n in 0..128 {
            for _ in 0..64 {
                for x in &mut data[..n] {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    *x = state as u8;
                }
                let _ = DynIndexView::parse(&data[..n], 128, &mut tags);
                let _ = DynLineView::parse(&data[..n], 128, &mut line_tags);
            }
        }
    }
}
