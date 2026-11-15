//! Allocation-free contracts for transparent integer wrappers.
use crate::{QslCustom, QslError, QslResult};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveRepr {
    pub bits: u8,
    pub signed: bool,
    pub min: i128,
    pub max: i128,
}
impl PrimitiveRepr {
    pub fn validate(self) -> QslResult<()> {
        if !matches!(self.bits, 8 | 16 | 32 | 64) || self.min > self.max {
            return Err(QslError::InvalidSchema);
        }
        let (lo, hi) = if self.signed {
            (-(1i128 << (self.bits - 1)), (1i128 << (self.bits - 1)) - 1)
        } else {
            (0, (1i128 << self.bits) - 1)
        };
        if self.min < lo || self.max > hi {
            return Err(QslError::InvalidSchema);
        }
        Ok(())
    }
}
/// Transparent integer wrapper whose native and wire representations are explicit.
/// # Safety
/// Self has the declared integer size/alignment, no padding or destructor, and
/// every declared domain value is a valid Self. Conversions are exact, total,
/// deterministic and allocation-free. The raw codec uses little-endian integers.
pub unsafe trait QslPrimitive: QslCustom {
    const REPRESENTATION: PrimitiveRepr;
    fn qsl_to_bits(&self) -> u64;
    fn qsl_from_bits(bits: u64) -> Self;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_widths_bounds_and_ranges_are_errors() {
        for bits in 0..=u8::MAX {
            let p = PrimitiveRepr {
                bits,
                signed: false,
                min: 0,
                max: 0,
            };
            assert_eq!(p.validate().is_ok(), matches!(bits, 8 | 16 | 32 | 64));
        }
        for p in [
            PrimitiveRepr {
                bits: 8,
                signed: false,
                min: -1,
                max: 5,
            },
            PrimitiveRepr {
                bits: 8,
                signed: true,
                min: -129,
                max: 0,
            },
            PrimitiveRepr {
                bits: 8,
                signed: true,
                min: 0,
                max: 128,
            },
            PrimitiveRepr {
                bits: 64,
                signed: false,
                min: 3,
                max: 2,
            },
        ] {
            assert!(p.validate().is_err());
        }
    }
}
