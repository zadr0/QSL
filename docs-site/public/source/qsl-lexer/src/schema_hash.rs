//! Build-time descriptor generation; fixed identity values are portable.
use crate::{payload_quant::PayloadQuantPlan, QslError, QslResult, QslSchema, QslType};
pub use qsl_core::schema_hash::*;
impl QslSchema {
    /// Analyze an already expanded/validated schema. Resolve DEQ constants and
    /// Rust macros before calling this function, as the build pipeline does.
    pub fn hashes(&self, algorithm: SchemaHashAlgorithm) -> QslResult<SchemaHashes> {
        let (strict, complete) = descriptor(self, SchemaHashKind::Strict)?;
        let (light, _) = descriptor(self, SchemaHashKind::Light)?;
        Ok(SchemaHashes {
            version: if self.options.auto_wire.is_some() {
                2
            } else {
                SCHEMA_HASH_VERSION
            },
            algorithm,
            strict: algorithm.digest(&strict),
            light: algorithm.digest(&light),
            complete,
        })
    }

    /// Canonical bytes for exact comparison, reproducibility and hash audits.
    /// No Debug output, HashMap iteration, native usize bytes or source text is
    /// hashed. Integer words and length prefixes are explicitly little-endian.
    pub fn hash_descriptor(&self, kind: SchemaHashKind) -> QslResult<Vec<u8>> {
        descriptor(self, kind).map(|(bytes, _)| bytes)
    }
}

struct Descriptor {
    bytes: Vec<u8>,
    complete: bool,
}
impl Descriptor {
    fn byte(&mut self, value: u8) {
        self.bytes.push(value);
    }
    fn size(&mut self, value: usize) -> QslResult<()> {
        self.bytes.extend_from_slice(
            &u64::try_from(value)
                .map_err(|_| QslError::LengthOverflow)?
                .to_le_bytes(),
        );
        Ok(())
    }
    fn number(&mut self, value: i128) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn text(&mut self, value: &str) -> QslResult<()> {
        self.size(value.len())?;
        self.bytes.extend_from_slice(value.as_bytes());
        Ok(())
    }
    fn bound(&mut self, value: Option<usize>) -> QslResult<()> {
        self.byte(u8::from(value.is_some()));
        if let Some(value) = value {
            self.size(value)?;
        }
        Ok(())
    }
    fn utf8(&mut self, min: usize, max: Option<usize>, codec: u8) -> QslResult<()> {
        self.byte(3);
        self.byte(codec);
        self.size(min)?;
        self.bound(max)
    }
    fn integer(&mut self, bits: u8, signed: bool, min: i128, max: i128, rank_base: i128) {
        self.byte(1);
        self.byte(bits);
        self.byte(u8::from(signed));
        self.number(min);
        self.number(max);
        self.number(rank_base);
    }
    fn ty(&mut self, ty: &QslType, rank_encoded: bool, depth: usize) -> QslResult<()> {
        if depth > 64 {
            return Err(QslError::InvalidSchema);
        }
        match ty {
            QslType::DomainInteger(d) => {
                use crate::domain::NumericDomain;
                if let NumericDomain::Interval(i) = d {
                    if i.physical_bits() <= 64
                        && i.step() == 1
                        && !(rank_encoded
                            && depth > 0
                            && i.physical_bits() == 8
                            && !i.signedness()
                            && i.minimum_bits() == 0
                            && i.maximum_bits() == 255)
                    {
                        let bits = i.physical_bits();
                        let actual = |n: u128| {
                            if i.signedness() {
                                ((n << (128 - bits)) as i128) >> (128 - bits)
                            } else {
                                n as i128
                            }
                        };
                        let min = actual(i.minimum_bits());
                        let max = actual(i.maximum_bits());
                        self.integer(
                            bits as u8,
                            i.signedness(),
                            min,
                            max,
                            if rank_encoded { min } else { 0 },
                        );
                        return Ok(());
                    }
                }
                self.byte(9);
                self.byte(d.physical_bits() as u8);
                self.byte(u8::from(d.signedness()));
                match d {
                    NumericDomain::Interval(i) => {
                        self.byte(0);
                        for n in [i.minimum_bits(), i.maximum_bits(), i.step()] {
                            self.number(n as i128);
                        }
                    }
                    NumericDomain::Set(set) => {
                        self.byte(1);
                        self.size(set.values().len())?;
                        for &n in set.values() {
                            self.number(n as i128);
                        }
                    }
                }
            }
            QslType::None => self.byte(0),
            QslType::Byte => self.integer(8, false, 0, 255, 0),
            QslType::Integer { bits, signed, min } => {
                let low = if *signed { -(1i128 << (*bits - 1)) } else { 0 };
                let high = if *signed {
                    (1i128 << (*bits - 1)) - 1
                } else {
                    (1i128 << *bits) - 1
                };
                self.integer(
                    *bits,
                    *signed,
                    low,
                    high,
                    if rank_encoded { *min } else { 0 },
                );
            }
            QslType::RangedInteger {
                bits,
                signed,
                min,
                max,
            } => {
                self.integer(
                    *bits,
                    *signed,
                    *min,
                    *max,
                    if rank_encoded { *min } else { 0 },
                );
            }
            QslType::Flags(bits) => {
                let width = if *bits <= 8 {
                    8
                } else if *bits <= 16 {
                    16
                } else if *bits <= 32 {
                    32
                } else {
                    64
                };
                self.integer(width, false, 0, (1i128 << bits) - 1, 0);
            }
            QslType::FixedString(n) => {
                self.byte(2);
                self.size(*n)?;
            }
            QslType::FixedCompressedString(n) if *n < crate::STRING_COMPRESSION_MIN_BYTES => {
                self.byte(2);
                self.size(*n)?;
            }
            QslType::FixedCompressedString(n) => {
                self.byte(4);
                self.size(*n)?;
            }
            QslType::String(min) => self.utf8(*min, None, 0)?,
            QslType::RangedString { min, max } => self.utf8(*min, Some(*max), 0)?,
            QslType::BorrowedString { min, max } => self.utf8(*min, *max, 0)?,
            // InlineRawString uses a different length header; it is not merely
            // an ownership switch. Keep it distinct from String and borrowed UTF-8.
            QslType::InlineRawString { min, max } => self.utf8(*min, Some(*max), 1)?,
            QslType::CompressedString { min, max } => self.utf8(*min, Some(*max), 2)?,
            QslType::Array(element, min) => {
                self.byte(5);
                self.size(*min)?;
                self.bound(None)?;
                self.ty(element, rank_encoded, depth + 1)?;
            }
            QslType::RangedArray { element, min, max } => {
                self.byte(5);
                self.size(*min)?;
                self.bound(Some(*max))?;
                self.ty(element, rank_encoded, depth + 1)?;
            }
            QslType::FixedArray(element, n) => {
                self.byte(6);
                self.size(*n)?;
                self.ty(element, rank_encoded, depth + 1)?;
            }
            QslType::BitPadding(n) => {
                self.byte(10);
                self.size(*n)?;
            }
            QslType::Padding(n) => {
                self.byte(7);
                self.size(*n)?;
            }
            QslType::Custom {
                rust_type,
                fixed_wire_size,
            } => {
                self.byte(8);
                self.text(rust_type)?;
                self.bound(*fixed_wire_size)?;
                self.complete = false;
            }
        }
        Ok(())
    }
}
fn descriptor(schema: &QslSchema, kind: SchemaHashKind) -> QslResult<(Vec<u8>, bool)> {
    schema.validate()?;
    let wire = crate::analysis::WirePlan::analyze(schema)?.lower(schema)?;
    let schema = &wire;
    let quant = PayloadQuantPlan::analyze(schema)?;
    let mut d = Descriptor {
        bytes: b"QSL-SCHEMA\0".to_vec(),
        complete: true,
    };
    let version = if schema.options.auto_wire.is_some() {
        2u16
    } else {
        SCHEMA_HASH_VERSION
    };
    d.bytes.extend_from_slice(&version.to_le_bytes());
    if let Some(wire_version) = schema.options.auto_wire {
        d.bytes.extend_from_slice(&wire_version.to_le_bytes());
    }
    d.byte(u8::from(kind == SchemaHashKind::Strict));
    if kind == SchemaHashKind::Strict {
        d.text(&schema.name)?;
    }
    if let Some(header) = schema.options.dynamic {
        d.byte(0xd1);
        d.byte(header.mode as u8);
        d.byte(header.width as u8);
        d.bytes.extend_from_slice(&header.revision.to_le_bytes());
    }
    d.byte(u8::from(schema.compact()));
    d.byte(u8::from(
        schema.compact() && schema.flags & crate::FLAG_ALIGNMENT != 0,
    ));
    // Effective format, not a raw bit-mask of source attributes. predict_seq
    // controls length inference; opt_table alone only dispatches readers. Q_table quantum
    // changes the static lookup representation, never the wire bytes.
    let hybrid = crate::hybrid::HybridPlan::analyze(schema)?;
    let mode = if hybrid.is_some() {
        7
    } else if quant.active {
        6
    } else if schema.predicted_opt_table() {
        3
    } else if schema.state_bits() {
        4
    } else if schema.predict_seq() {
        5
    } else if schema.legacy_opt_table() {
        2
    } else if schema.bitmap() {
        1
    } else {
        0
    };
    d.byte(mode);
    if schema.bitmap() {
        d.byte(2);
        d.size(schema.bitmap_width())?;
    }
    if let Some(h) = &hybrid {
        d.size(h.prefix.fields.len())?;
        d.size(h.prefix_bytes)?;
        d.size(h.length_header)?;
        d.size(schema.quantum())?;
    }
    d.byte(u8::from(schema.qps()));
    if schema.qps() {
        d.size(schema.quantum())?;
        d.byte(3);
    } // QPB3 envelope
    d.byte(u8::from(quant.active));
    if quant.active {
        d.size(quant.header_bytes)?;
    }
    if mode == 0 && schema.optional_count() > 1 {
        d.size(schema.tag_bits())?;
    }
    d.size(schema.fields.len())?;
    for field in &schema.fields {
        if kind == SchemaHashKind::Strict {
            d.text(&field.name)?;
        }
        d.byte(u8::from(field.optional));
        if schema.options.dynamic.is_some()
            || (mode == 0 && schema.optional_count() > 1 && field.optional)
        {
            d.size(field.tag)?;
        }
        if schema.compact() {
            d.size(field.data.rank_width(field.bits)?)?;
        }
        d.ty(
            &field.data,
            schema.compact()
                || quant.active
                || hybrid
                    .as_ref()
                    .is_some_and(|h| h.prefix.fields.iter().any(|f| f.tag == field.tag)),
            0,
        )?;
    }
    Ok((d.bytes, d.complete))
}
