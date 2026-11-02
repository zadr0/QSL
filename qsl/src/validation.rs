//! Safe, allocation-free wire validation. The trusted pointer decoder is called
//! only after the complete frame has passed these schema-specific checks.
use crate::{QslError, QslResult};

pub struct ValidationCursor<'a> {
    input: &'a [u8],
    bit: usize,
}
impl<'a> ValidationCursor<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, bit: 0 }
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
        let mut value = 0u64;
        for i in 0..width {
            value |= (((self.input[(self.bit + i) >> 3] >> ((self.bit + i) & 7)) & 1) as u64) << i;
        }
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
        std::str::from_utf8(self.bytes(n)?).map_err(|_| QslError::InvalidUtf8)?;
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
