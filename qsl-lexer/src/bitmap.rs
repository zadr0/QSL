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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries_and_spare_bits() {
        for n in 0..=128 {
            let p = BitmapPlan::new(n).unwrap();
            assert_eq!(
                p.width,
                match n {
                    0 => 0,
                    1..=8 => 8,
                    9..=16 => 16,
                    17..=32 => 32,
                    33..=64 => 64,
                    _ => 128,
                }
            );
            assert!(p.validate(0).is_ok());
            for marker in 1..=p.width {
                let bit = 1u128 << (p.width - marker);
                assert_eq!(p.validate(bit).is_ok(), marker <= n);
            }
        }
        assert!(BitmapPlan::new(129).is_err());
    }
    #[test]
    fn traversal_matches_reverse_markers() {
        for n in [1, 8, 9, 16, 17, 32, 33, 64, 65, 128] {
            let p = BitmapPlan::new(n).unwrap();
            let mut mask = (1..=n)
                .step_by(3)
                .fold(0, |m, i| m | p.marker_bit(i).unwrap());
            let mut markers = Vec::new();
            while mask != 0 {
                markers.push(p.width - mask.trailing_zeros() as usize);
                mask &= mask - 1;
            }
            assert_eq!(
                markers,
                (1..=n)
                    .step_by(3)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
            );
        }
    }
}
