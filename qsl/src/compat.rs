//! Adapter for the original QSL parser model.
//!
//! The parser's public enum and schema shape must not be silently replaced
//! with interpreter-only range variants. `bits` is the precomputed rank width.

use crate::io::{QslError, QslResult};
use crate::types::{QslField, QslSchema, QslType};

#[derive(Default, Debug, Clone)]
pub enum ParserQslType {
    #[default]
    None,
    Byte,
    FixedString,
    String(usize),
    Number16(bool, usize),
    Number32(bool, usize),
    Number64(bool, usize),
    Flags,
    Array(Box<Self>, usize),
    Padding,
}

#[derive(Default, Debug, Clone)]
pub struct ParserQslField {
    pub optional: bool,
    pub bits: usize,
    pub data: ParserQslType,
    pub name: String,
    pub tag: usize,
}

#[derive(Default, Debug, Clone)]
pub struct ParserQslSchema {
    pub flags: usize,
    pub fields: Vec<ParserQslField>,
    pub name: String,
    pub max_tag: usize,
}

impl ParserQslSchema {
    pub fn new(flags: usize) -> Self {
        Self {
            flags,
            ..Self::default()
        }
    }
    pub fn optional_count(&self) -> usize {
        self.fields.iter().filter(|f| f.optional).count()
    }
    pub fn validate(&self) -> QslResult<()> {
        QslSchema::try_from(self).map(|_| ())
    }
}

impl ParserQslType {
    /// The two's-complement form of a signed minimum is transported in usize,
    /// preserving the original `NumberXX(bool, usize)` enum layout.
    fn normalize(&self, rank_bits: usize) -> QslResult<QslType> {
        Ok(match self {
            Self::None => QslType::None,
            Self::Byte => QslType::Byte,
            Self::FixedString => {
                if rank_bits & 7 != 0 {
                    return Err(QslError::InvalidSchema);
                }
                QslType::FixedString(rank_bits / 8)
            }
            Self::String(min) => QslType::String(*min),
            Self::Number16(signed, min) => QslType::Integer {
                bits: 16,
                signed: *signed,
                min: if *signed {
                    (*min as isize) as i128
                } else {
                    *min as i128
                },
            },
            Self::Number32(signed, min) => QslType::Integer {
                bits: 32,
                signed: *signed,
                min: if *signed {
                    (*min as isize) as i128
                } else {
                    *min as i128
                },
            },
            Self::Number64(signed, min) => QslType::Integer {
                bits: 64,
                signed: *signed,
                min: if *signed {
                    (*min as isize) as i128
                } else {
                    *min as i128
                },
            },
            Self::Flags => QslType::Flags(rank_bits),
            Self::Array(inner, min) => QslType::Array(Box::new(inner.normalize(rank_bits)?), *min),
            Self::Padding => QslType::Padding(rank_bits / 8),
        })
    }
}

impl TryFrom<&ParserQslSchema> for QslSchema {
    type Error = QslError;

    fn try_from(source: &ParserQslSchema) -> QslResult<Self> {
        let mut fields = Vec::with_capacity(source.fields.len());
        for (index, f) in source.fields.iter().enumerate() {
            let tag = if f.tag == 0 { index + 1 } else { f.tag };
            fields.push(QslField {
                optional: f.optional,
                bits: f.bits,
                data: f.data.normalize(f.bits)?,
                name: f.name.clone(),
                tag,
                default: None,
            });
        }
        let maximum = fields.iter().map(|f| f.tag).max().unwrap_or(0);
        let max_tag = source.max_tag.max(maximum);
        let schema = QslSchema {
            flags: source.flags,
            fields,
            name: source.name.clone(),
            max_tag,
        };
        // Original parser uses bit 3 for lifetime. Internal flags must match.
        schema.validate()?;
        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retains_old_number_shape_and_negative_minimum() {
        let ty = ParserQslType::Number16(true, 0usize.wrapping_sub(5));
        assert!(matches!(
            ty.normalize(4).unwrap(),
            QslType::Integer { min: -5, .. }
        ));
    }
}
