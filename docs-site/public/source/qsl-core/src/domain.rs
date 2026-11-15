//! Exact build-time integer domains. No floating-point width calculation.
use crate::{QslError, QslResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cardinality {
    Finite(u128),
    Full128,
}

/// Bounds are native-width bit patterns; signed order is two's complement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntegerDomain {
    physical_bits: u16,
    signed: bool,
    minimum: u128,
    maximum: u128,
    step: u128,
}

pub const fn low_mask(bits: u16) -> u128 {
    match bits {
        0 => 0,
        1..=127 => (1u128 << bits) - 1,
        _ => u128::MAX,
    }
}
impl IntegerDomain {
    pub const fn physical_bits(self) -> u16 {
        self.physical_bits
    }
    pub const fn signedness(self) -> bool {
        self.signed
    }
    pub const fn minimum_bits(self) -> u128 {
        self.minimum
    }
    pub const fn maximum_bits(self) -> u128 {
        self.maximum
    }
    pub const fn step(self) -> u128 {
        self.step
    }
    /// Encode an exact rank into MSB-first bytes; only the returned prefix is written.
    pub fn encode_bytes(self, value: u128, out: &mut [u8]) -> QslResult<usize> {
        let rank = self.encode_bits(value)?;
        let bits = self.encoded_bits() as usize;
        let n = bits.div_ceil(8);
        let out = out.get_mut(..n).ok_or(QslError::BufferTooSmall)?;
        // Width and rank are proven by the validated domain; the exact prefix is writable.
        unsafe {
            let mut writer = crate::dynamic::DynBitWriter::from_raw(out.as_mut_ptr());
            writer.write_unsigned_unchecked(rank, bits);
        }
        Ok(n)
    }
    /// Decode one exact MSB-first rank, rejecting spare codes and nonzero tail padding.
    pub fn decode_bytes(self, input: &[u8]) -> QslResult<u128> {
        let bits = self.encoded_bits() as usize;
        let n = bits.div_ceil(8);
        if input.len() != n {
            return Err(QslError::InvalidValue);
        }
        if !bits.is_multiple_of(8) && input[n - 1] & ((1u8 << (8 - bits % 8)) - 1) != 0 {
            return Err(QslError::InvalidPadding);
        }
        let mut rank = 0u128;
        for i in 0..bits {
            rank = (rank << 1) | u128::from((input[i / 8] >> (7 - i % 8)) & 1);
        }
        self.decode_bits(rank)
    }

    pub fn unsigned(bits: u16, minimum: u128, maximum: u128) -> QslResult<Self> {
        Self::progression(bits, false, minimum, maximum, 1)
    }
    pub fn signed(bits: u16, minimum: i128, maximum: i128) -> QslResult<Self> {
        if !matches!(bits, 8 | 16 | 32 | 64 | 128) {
            return Err(QslError::InvalidWidth);
        }
        if bits < 128 {
            let limit = 1i128 << (bits - 1);
            if minimum < -limit || maximum >= limit {
                return Err(QslError::InvalidValue);
            }
        }
        Self::progression(
            bits,
            true,
            (minimum as u128) & low_mask(bits),
            (maximum as u128) & low_mask(bits),
            1,
        )
    }
    /// Build an inclusive progression; both endpoints must be reachable by the positive step.
    pub fn progression(
        bits: u16,
        signed: bool,
        minimum: u128,
        maximum: u128,
        step: u128,
    ) -> QslResult<Self> {
        if !matches!(bits, 8 | 16 | 32 | 64 | 128) {
            return Err(QslError::InvalidWidth);
        }
        let mask = low_mask(bits);
        if minimum & !mask != 0 || maximum & !mask != 0 || step == 0 {
            return Err(QslError::InvalidValue);
        }
        let sign = if signed { 1u128 << (bits - 1) } else { 0 };
        if (minimum ^ sign) > (maximum ^ sign) {
            return Err(QslError::InvalidSchema);
        }
        let span = maximum.wrapping_sub(minimum) & mask;
        if !span.is_multiple_of(step) {
            return Err(QslError::InvalidSchema);
        }
        Ok(Self {
            physical_bits: bits,
            signed,
            minimum,
            maximum,
            step,
        })
    }
    pub fn maximum_rank(self) -> u128 {
        (self.maximum.wrapping_sub(self.minimum) & low_mask(self.physical_bits)) / self.step
    }
    pub fn cardinality(self) -> Cardinality {
        self.maximum_rank()
            .checked_add(1)
            .map_or(Cardinality::Full128, Cardinality::Finite)
    }
    pub fn encoded_bits(self) -> u16 {
        (128 - self.maximum_rank().leading_zeros()) as u16
    }
    pub fn encode_bits(self, value: u128) -> QslResult<u128> {
        let mask = low_mask(self.physical_bits);
        if value & !mask != 0 {
            return Err(QslError::InvalidValue);
        }
        let sign = if self.signed {
            1u128 << (self.physical_bits - 1)
        } else {
            0
        };
        let ordered = value ^ sign;
        if ordered < (self.minimum ^ sign) || ordered > (self.maximum ^ sign) {
            return Err(QslError::InvalidValue);
        }
        let delta = value.wrapping_sub(self.minimum) & mask;
        if !delta.is_multiple_of(self.step) {
            return Err(QslError::InvalidValue);
        }
        Ok(delta / self.step)
    }
    pub fn encode_signed(self, value: i128) -> QslResult<u128> {
        if !self.signed {
            return Err(QslError::InvalidSchema);
        }
        if self.physical_bits < 128 {
            let limit = 1i128 << (self.physical_bits - 1);
            if value < -limit || value >= limit {
                return Err(QslError::InvalidValue);
            }
        }
        self.encode_bits((value as u128) & low_mask(self.physical_bits))
    }
    pub fn decode_bits(self, rank: u128) -> QslResult<u128> {
        if rank > self.maximum_rank() {
            return Err(QslError::InvalidValue);
        }
        // rank*step <= span by the preceding bound; wrapping addition represents signed bits.
        let delta = rank.checked_mul(self.step).ok_or(QslError::InvalidValue)?;
        Ok(self.minimum.wrapping_add(delta) & low_mask(self.physical_bits))
    }
    pub fn decode_signed(self, rank: u128) -> QslResult<i128> {
        if !self.signed {
            return Err(QslError::InvalidSchema);
        }
        let bits = self.decode_bits(rank)?;
        Ok(if self.physical_bits == 128 {
            bits as i128
        } else {
            let shift = 128 - self.physical_bits;
            ((bits << shift) as i128) >> shift
        })
    }
}
