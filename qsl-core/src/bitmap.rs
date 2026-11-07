//! Bitmap v2: optional-only markers, smallest native word, descending marker traversal.
use crate::{QslError, QslResult};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitmapPlan {
    pub fields: usize,
    pub width: usize,
}
impl BitmapPlan {
    pub fn new(fields: usize) -> QslResult<Self> {
        let width = match fields {
            0 => 0,
            1..=8 => 8,
            9..=16 => 16,
            17..=32 => 32,
            33..=64 => 64,
            65..=128 => 128,
            _ => return Err(QslError::InvalidSchema),
        };
        Ok(Self { fields, width })
    }
    pub fn bytes(self) -> usize {
        self.width / 8
    }
    pub fn rust_type(self) -> &'static str {
        match self.width {
            0 | 8 => "u8",
            16 => "u16",
            32 => "u32",
            64 => "u64",
            128 => "u128",
            _ => unreachable!(),
        }
    }
    pub fn marker_bit(self, marker: usize) -> QslResult<u128> {
        if marker == 0 || marker > self.fields {
            return Err(QslError::InvalidValue);
        }
        Ok(1u128 << (self.width - marker))
    }
    pub fn validate(self, mask: u128) -> QslResult<()> {
        if mask == 0 {
            return Ok(());
        }
        if self.width == 0 || (self.width < 128 && mask >> self.width != 0) {
            return Err(QslError::InvalidValue);
        }
        let maximum = self.width - (mask.trailing_zeros() as usize);
        if maximum > self.fields {
            return Err(QslError::InvalidValue);
        }
        Ok(())
    }
}
