//! Build-time owned value sets and shared allocation-free intervals.
use crate::{QslError, QslResult};
pub use qsl_core::domain::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerSet {
    physical_bits: u16,
    signed: bool,
    values: Vec<u128>,
}
impl IntegerSet {
    pub const fn physical_bits(&self) -> u16 {
        self.physical_bits
    }
    pub const fn signedness(&self) -> bool {
        self.signed
    }
    pub fn signed(bits: u16, values: impl IntoIterator<Item = i128>) -> QslResult<Self> {
        let mut native = Vec::new();
        for value in values {
            IntegerDomain::signed(bits, value, value)?;
            native.push(value as u128 & low_mask(bits));
        }
        Self::new(bits, true, native)
    }
    pub fn new(bits: u16, signed: bool, mut values: Vec<u128>) -> QslResult<Self> {
        if !matches!(bits, 8 | 16 | 32 | 64 | 128) {
            return Err(QslError::InvalidWidth);
        }
        if values.is_empty() || values.iter().any(|v| v & !low_mask(bits) != 0) {
            return Err(QslError::InvalidValue);
        }
        let sign = if signed { 1u128 << (bits - 1) } else { 0 };
        values.sort_by_key(|v| v ^ sign);
        if values.windows(2).any(|w| w[0] == w[1]) {
            return Err(QslError::InvalidSchema);
        }
        Ok(Self {
            physical_bits: bits,
            signed,
            values,
        })
    }
    pub fn values(&self) -> &[u128] {
        &self.values
    }
    pub fn encoded_bits(&self) -> u16 {
        (usize::BITS - (self.values.len() - 1).leading_zeros()) as u16
    }
    pub fn encode_bits(&self, value: u128) -> QslResult<usize> {
        if value & !low_mask(self.physical_bits) != 0 {
            return Err(QslError::InvalidValue);
        }
        let sign = if self.signed {
            1u128 << (self.physical_bits - 1)
        } else {
            0
        };
        self.values
            .binary_search_by_key(&(value ^ sign), |v| v ^ sign)
            .map_err(|_| QslError::InvalidValue)
    }
    pub fn decode_bits(&self, rank: usize) -> QslResult<u128> {
        self.values.get(rank).copied().ok_or(QslError::InvalidValue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_width_domains_and_extremes() {
        let u = IntegerDomain::unsigned(128, 0, u128::MAX).unwrap();
        assert_eq!(u.cardinality(), Cardinality::Full128);
        assert_eq!(u.encoded_bits(), 128);
        for x in [0, 1, u64::MAX as u128, u128::MAX - 1, u128::MAX] {
            assert_eq!(u.decode_bits(u.encode_bits(x).unwrap()).unwrap(), x)
        }
        let s = IntegerDomain::signed(128, i128::MIN, i128::MAX).unwrap();
        assert_eq!(s.cardinality(), Cardinality::Full128);
        for x in [i128::MIN, -1, 0, 1, i128::MAX] {
            assert_eq!(s.decode_signed(s.encode_signed(x).unwrap()).unwrap(), x)
        }
        assert_eq!(s.encode_signed(i128::MIN).unwrap(), 0);
        assert_eq!(s.encode_signed(i128::MAX).unwrap(), u128::MAX);
    }
    #[test]
    fn real_byte_codec_covers_full_128_bits_and_exact_output() {
        for domain in [
            IntegerDomain::unsigned(128, 0, u128::MAX).unwrap(),
            IntegerDomain::signed(128, i128::MIN, i128::MAX).unwrap(),
            IntegerDomain::unsigned(128, 7, 7 + (1u128 << 77) - 1).unwrap(),
        ] {
            for rank in [0, domain.maximum_rank() / 2, domain.maximum_rank()] {
                let value = domain.decode_bits(rank).unwrap();
                let mut output = [0xa5; 17];
                let n = domain.encode_bytes(value, &mut output).unwrap();
                assert_eq!(domain.decode_bytes(&output[..n]).unwrap(), value);
                assert_eq!(output[n], 0xa5);
            }
        }
    }
    #[test]
    fn signed_zero_crossing_is_exact_exhaustively() {
        for minimum in -128i128..=127 {
            for maximum in minimum..=127 {
                let d = IntegerDomain::signed(8, minimum, maximum).unwrap();
                for value in minimum..=maximum {
                    let r = d.encode_signed(value).unwrap();
                    assert_eq!(r, (value - minimum) as u128);
                    assert_eq!(d.decode_signed(r).unwrap(), value)
                }
                assert!(d.decode_bits(d.maximum_rank() + 1).is_err());
            }
        }
    }
    #[test]
    fn progression_and_sets_are_not_intervals() {
        let d = IntegerDomain::progression(8, true, (-3i8) as u8 as u128, 3, 2).unwrap();
        for (rank, value) in [-3, -1, 1, 3].into_iter().enumerate() {
            assert_eq!(d.encode_signed(value).unwrap(), rank as u128);
            assert_eq!(d.decode_signed(rank as u128).unwrap(), value)
        }
        assert!(d.encode_signed(0).is_err());
        assert!(IntegerDomain::progression(8, false, 1, 8, 2).is_err());
        let aligned = IntegerDomain::progression(8, false, 1, 7, 2).unwrap();
        assert_eq!(aligned.maximum_rank(), 3);
        assert_eq!(aligned.decode_bits(3).unwrap(), 7);
        assert!(aligned.encode_bits(8).is_err());
        let set = IntegerSet::new(8, true, vec![1, 255]).unwrap();
        assert_eq!(set.encoded_bits(), 1);
        assert!(set.encode_bits(0).is_err());
        assert_eq!(set.encode_bits(255).unwrap(), 0);
        assert_eq!(IntegerDomain::signed(8, -1, 1).unwrap().encoded_bits(), 2);
    }
    #[test]
    fn every_width_boundary_and_constant() {
        for width in [
            0u16, 1, 2, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 77, 127, 128,
        ] {
            let d = IntegerDomain::unsigned(128, 0, low_mask(width)).unwrap();
            assert_eq!(d.encoded_bits(), width);
            for value in [0, low_mask(width) / 2, low_mask(width)] {
                assert_eq!(d.decode_bits(d.encode_bits(value).unwrap()).unwrap(), value)
            }
        }
        let d = IntegerDomain::unsigned(64, 7, 7).unwrap();
        assert_eq!(d.encoded_bits(), 0);
        assert_eq!(d.decode_bits(0).unwrap(), 7);
        assert!(IntegerDomain::signed(8, -129, 0).is_err());
        assert!(IntegerDomain::unsigned(8, 0, 256).is_err());
    }
}

/// Exact schema operand domain. All metadata is owned by the build-time schema.
/// Generated readers use constants; this enum does not allocate on the hot path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumericDomain {
    Interval(IntegerDomain),
    Set(IntegerSet),
}
impl From<IntegerDomain> for NumericDomain {
    fn from(v: IntegerDomain) -> Self {
        Self::Interval(v)
    }
}
impl From<IntegerSet> for NumericDomain {
    fn from(v: IntegerSet) -> Self {
        Self::Set(v)
    }
}
impl NumericDomain {
    pub fn physical_bits(&self) -> u16 {
        match self {
            Self::Interval(d) => d.physical_bits(),
            Self::Set(d) => d.physical_bits(),
        }
    }
    pub fn signedness(&self) -> bool {
        match self {
            Self::Interval(d) => d.signedness(),
            Self::Set(d) => d.signedness(),
        }
    }
    pub fn encoded_bits(&self) -> u16 {
        match self {
            Self::Interval(d) => d.encoded_bits(),
            Self::Set(d) => d.encoded_bits(),
        }
    }
    pub fn minimum_bits(&self) -> u128 {
        match self {
            Self::Interval(d) => d.minimum_bits(),
            Self::Set(d) => d.values()[0],
        }
    }
    pub fn maximum_rank(&self) -> u128 {
        match self {
            Self::Interval(d) => d.maximum_rank(),
            Self::Set(d) => (d.values().len() - 1) as u128,
        }
    }
    pub fn encode_bits(&self, value: u128) -> QslResult<u128> {
        match self {
            Self::Interval(d) => d.encode_bits(value),
            Self::Set(d) => d.encode_bits(value).map(|n| n as u128),
        }
    }
    pub fn decode_bits(&self, rank: u128) -> QslResult<u128> {
        match self {
            Self::Interval(d) => d.decode_bits(rank),
            Self::Set(d) => {
                d.decode_bits(usize::try_from(rank).map_err(|_| QslError::InvalidValue)?)
            }
        }
    }
    pub fn primitive(&self) -> &'static str {
        match (self.physical_bits(), self.signedness()) {
            (8, false) => "u8",
            (8, true) => "i8",
            (16, false) => "u16",
            (16, true) => "i16",
            (32, false) => "u32",
            (32, true) => "i32",
            (64, false) => "u64",
            (64, true) => "i64",
            (128, false) => "u128",
            (128, true) => "i128",
            _ => unreachable!("validated domain width"),
        }
    }
}

#[cfg(test)]
mod schema_width_tests {
    use super::*;
    #[test]
    fn public_schema_metadata_cannot_truncate_an_exact_domain() {
        let domain: NumericDomain = IntegerDomain::unsigned(128, 0, u128::MAX).unwrap().into();
        let mut schema = crate::QslSchema {
            name: "P".into(),
            fields: vec![crate::QslField {
                name: "value".into(),
                tag: 1,
                bits: 64,
                data: crate::QslType::DomainInteger(domain),
                ..Default::default()
            }],
            max_tag: 1,
            ..Default::default()
        };
        assert_eq!(schema.validate(), Err(QslError::InvalidWidth));
        schema.fields[0].bits = 128;
        schema.validate().unwrap();
        schema.fields[0].data = crate::QslType::FixedString(256);
        schema.fields[0].bits = 256;
        schema.validate().unwrap();
    }
}
