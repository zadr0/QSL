//! Build-time sign/magnitude arithmetic; no float and no narrowing through i128.
use crate::lexer::{Lexer, ParseError};
use std::collections::HashMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WideNumber {
    negative: bool,
    magnitude: u128,
}
impl WideNumber {
    pub fn positive(magnitude: u128) -> Self {
        Self {
            negative: false,
            magnitude,
        }
    }
    pub fn from_i128(n: i128) -> Self {
        Self {
            negative: n < 0,
            magnitude: n.unsigned_abs(),
        }
    }
    pub fn signed(self) -> Option<i128> {
        if self.negative {
            if self.magnitude == 1u128 << 127 {
                Some(i128::MIN)
            } else {
                i128::try_from(self.magnitude).ok()?.checked_neg()
            }
        } else {
            i128::try_from(self.magnitude).ok()
        }
    }
    fn neg(mut self) -> Self {
        if self.magnitude != 0 {
            self.negative = !self.negative;
        }
        self
    }
    fn add(self, other: Self) -> Option<Self> {
        if self.negative == other.negative {
            Some(Self {
                negative: self.negative,
                magnitude: self.magnitude.checked_add(other.magnitude)?,
            })
        } else if self.magnitude >= other.magnitude {
            Some(Self {
                negative: self.negative && self.magnitude != other.magnitude,
                magnitude: self.magnitude - other.magnitude,
            })
        } else {
            Some(Self {
                negative: other.negative,
                magnitude: other.magnitude - self.magnitude,
            })
        }
    }
    pub fn unsigned(self) -> Option<u128> {
        if self.negative {
            None
        } else {
            Some(self.magnitude)
        }
    }
    pub fn native(self, bits: u16, signed: bool) -> Option<u128> {
        let mask = crate::domain::low_mask(bits);
        if signed {
            let limit = 1u128 << (bits - 1);
            if self.negative {
                (self.magnitude <= limit).then(|| 0u128.wrapping_sub(self.magnitude) & mask)
            } else {
                (self.magnitude < limit).then_some(self.magnitude)
            }
        } else {
            (!self.negative && self.magnitude <= mask).then_some(self.magnitude)
        }
    }
}
impl<'a> Lexer<'a> {
    fn wide_atom(&mut self, defs: &HashMap<String, i128>) -> Result<WideNumber, ParseError> {
        if self.eat("-")? {
            return Ok(self.wide_atom(defs)?.neg());
        }
        if self.eat("+")? {
            return self.wide_atom(defs);
        }
        if self.eat("(")? {
            let n = self.wide_expr(defs)?;
            self.expect(")")?;
            return Ok(n);
        }
        if self.eat("!")? {
            let name = self.ident()?;
            return self
                .wide_constants
                .get(name)
                .copied()
                .or_else(|| defs.get(name).copied().map(WideNumber::from_i128))
                .ok_or_else(|| self.error(format!("unknown constant `{name}`")));
        }
        self.skip_trivia()?;
        if self.bytes().get(self.pos).is_some_and(u8::is_ascii_digit) {
            return self.number_u128().map(WideNumber::positive);
        }
        let name = self.ident()?;
        if name == "sizeof" {
            self.expect("(")?;
            let size = match self.ident()? {
                "u8" | "i8" => 1,
                "u16" | "i16" => 2,
                "u32" | "i32" => 4,
                "u64" | "i64" => 8,
                "u128" | "i128" => 16,
                _ => return Err(self.error("unsupported sizeof type")),
            };
            self.expect(")")?;
            return Ok(WideNumber::positive(size));
        }
        self.wide_constants
            .get(name)
            .copied()
            .or_else(|| defs.get(name).copied().map(WideNumber::from_i128))
            .ok_or_else(|| self.error(format!("unknown constant `{name}`")))
    }
    fn wide_term(&mut self, defs: &HashMap<String, i128>) -> Result<WideNumber, ParseError> {
        let mut lhs = self.wide_atom(defs)?;
        loop {
            let op = if self.eat("*")? {
                0
            } else if self.eat("/")? {
                1
            } else if self.eat("%")? {
                2
            } else {
                return Ok(lhs);
            };
            let rhs = self.wide_atom(defs)?;
            let magnitude = match op {
                0 => lhs.magnitude.checked_mul(rhs.magnitude),
                1 => lhs.magnitude.checked_div(rhs.magnitude),
                _ => lhs.magnitude.checked_rem(rhs.magnitude),
            }
            .ok_or_else(|| self.error("128-bit expression overflow or division by zero"))?;
            lhs = WideNumber {
                negative: magnitude != 0
                    && if op == 2 {
                        lhs.negative
                    } else {
                        lhs.negative != rhs.negative
                    },
                magnitude,
            };
        }
    }
    pub(crate) fn wide_expr(
        &mut self,
        defs: &HashMap<String, i128>,
    ) -> Result<WideNumber, ParseError> {
        let mut lhs = self.wide_term(defs)?;
        loop {
            let add = if self.eat("+")? {
                true
            } else if self.eat("-")? {
                false
            } else {
                return Ok(lhs);
            };
            let rhs = self.wide_term(defs)?;
            lhs = lhs
                .add(if add { rhs } else { rhs.neg() })
                .ok_or_else(|| self.error("128-bit expression overflow"))?;
        }
    }
}

impl From<i128> for WideNumber {
    fn from(value: i128) -> Self {
        Self::from_i128(value)
    }
}
impl PartialEq<i128> for WideNumber {
    fn eq(&self, value: &i128) -> bool {
        self.signed() == Some(*value)
    }
}
impl std::fmt::Display for WideNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.negative {
            write!(f, "-{}", self.magnitude)
        } else {
            self.magnitude.fmt(f)
        }
    }
}
