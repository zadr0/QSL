//! Safe, allocation-free wire validation. The trusted pointer decoder is called
//! only after the complete frame has passed these schema-specific checks.
use crate::{QslError, QslResult};

pub const DEFAULT_MAX_DATA_SIZE: usize = 65535;

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
        check_size(input.len(), maximum)?;
        Ok(Self {
            input,
            bit: 0,
            budget: ValueBudget::new(maximum),
        })
    }
    pub fn charge(&mut self, bytes: usize) -> QslResult<()> {
        self.budget.charge(bytes)
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
        if width == 0 {
            return Ok(0);
        }
        let start = self.bit >> 3;
        let shift = self.bit & 7;
        let count = (shift + width + 7) >> 3;
        let mut word = [0u8; 16];
        word[..count].copy_from_slice(&self.input[start..start + count]);
        let value = (u128::from_le_bytes(word) >> shift) as u64;
        let mask = u64::MAX >> (64 - width);
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
        let width = if MAX <= 255 { 1 } else { 2 };
        let (stored, compressed, prefix) = if fixed {
            self.align()?;
            if MAX > 0 && self.input.get(self.position()) == Some(&255) {
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
            return self.utf8(stored, if fixed { MAX } else { min }, MAX);
        }
        if raw_only || stored > MAX {
            return Err(QslError::InvalidValue);
        }
        let packed = self.bytes(stored)?;
        let mut output = [0u8; MAX];
        let mut produced = 0usize;
        let mut cursor = 0usize;
        while cursor < packed.len() {
            let control = packed[cursor];
            cursor += 1;
            let count = if control < 128 {
                control as usize + 1
            } else {
                (control & 127) as usize + 4
            };
            let end = produced
                .checked_add(count)
                .ok_or(QslError::LengthOverflow)?;
            self.charge(count)?;
            if end > MAX {
                return Err(QslError::InvalidValue);
            }
            if control < 128 {
                let tail = cursor.checked_add(count).ok_or(QslError::LengthOverflow)?;
                output[produced..end]
                    .copy_from_slice(packed.get(cursor..tail).ok_or(QslError::UnexpectedEof)?);
                cursor = tail;
            } else {
                let bytes = packed
                    .get(cursor..cursor + width)
                    .ok_or(QslError::UnexpectedEof)?;
                let distance = bytes[0] as usize
                    + if width == 2 {
                        (bytes[1] as usize) << 8
                    } else {
                        0
                    };
                cursor += width;
                if distance == 0 || distance > produced {
                    return Err(QslError::InvalidValue);
                }
                for i in produced..end {
                    output[i] = output[i - distance];
                }
            }
            produced = end;
        }
        if produced < min || (fixed && produced != MAX) || prefix + stored >= produced {
            return Err(QslError::InvalidValue);
        }
        std::str::from_utf8(&output[..produced]).map_err(|_| QslError::InvalidUtf8)?;
        Ok(())
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
