//! Build-time schema identities over a versioned, architecture-independent descriptor.
//! Names belong only to the strict identity. Storage, inlining, RT pool settings,
//! initializers and optional-table implementation details do not belong to either.
use crate::{payload_quant::PayloadQuantPlan, QslError, QslResult, QslSchema, QslType};

/// Bump when the canonical descriptor or any represented wire codec changes.
pub const SCHEMA_HASH_VERSION: u16 = 1;

/// xxHash is non-cryptographic, including its wider 128-bit variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SchemaHashAlgorithm {
    /// `!config USE_HASH = xxHash;` (XXH3-64, seed zero).
    #[default]
    Xxh3_64,
    /// `!config USE_HASH = xxHash128;` (XXH3-128, seed zero).
    Xxh3_128,
}
impl SchemaHashAlgorithm {
    pub const fn bits(self) -> u32 {
        match self {
            Self::Xxh3_64 => 64,
            Self::Xxh3_128 => 128,
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::Xxh3_64 => "xxHash",
            Self::Xxh3_128 => "xxHash128",
        }
    }
    pub const fn rust_variant(self) -> &'static str {
        match self {
            Self::Xxh3_64 => "Xxh3_64",
            Self::Xxh3_128 => "Xxh3_128",
        }
    }
    fn digest(self, bytes: &[u8]) -> u128 {
        match self {
            Self::Xxh3_64 => u128::from(xxhash_rust::xxh3::xxh3_64(bytes)),
            Self::Xxh3_128 => xxhash_rust::xxh3::xxh3_128(bytes),
        }
    }
}
impl std::str::FromStr for SchemaHashAlgorithm {
    type Err = QslError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "xxHash" | "XXH3_64" => Ok(Self::Xxh3_64),
            "xxHash128" | "XXH3_128" => Ok(Self::Xxh3_128),
            _ => Err(QslError::InvalidSchema),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaHashKind {
    Strict,
    Light,
}

/// A digest must be compared together with algorithm, version and completeness.
/// For XXH3-64 the upper 64 bits of each value are zero. Neither digest is added
/// to a serialized record; sending or negotiating it remains explicit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchemaHashes {
    pub version: u16,
    pub algorithm: SchemaHashAlgorithm,
    pub strict: u128,
    pub light: u128,
    /// False for an opaque QslCustom: only its Rust path and declared size are
    /// known. Its external implementation must be agreed separately.
    pub complete: bool,
}
impl SchemaHashes {
    pub fn strict_hex(self) -> String {
        self.hex(self.strict)
    }
    pub fn light_hex(self) -> String {
        self.hex(self.light)
    }
    fn hex(self, value: u128) -> String {
        format!(
            "{value:0width$x}",
            width = self.algorithm.bits() as usize / 4
        )
    }
}

/// Generated implementations contain literal constants, never runtime hashing.
pub trait SchemaIdentity {
    const SCHEMA_HASHES: SchemaHashes;
}

impl QslSchema {
    /// Analyze an already expanded/validated schema. Resolve DEQ constants and
    /// Rust macros before calling this function, as the build pipeline does.
    pub fn hashes(&self, algorithm: SchemaHashAlgorithm) -> QslResult<SchemaHashes> {
        let (strict, complete) = descriptor(self, SchemaHashKind::Strict)?;
        let (light, _) = descriptor(self, SchemaHashKind::Light)?;
        Ok(SchemaHashes {
            version: SCHEMA_HASH_VERSION,
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
    let quant = PayloadQuantPlan::analyze(schema)?;
    let mut d = Descriptor {
        bytes: b"QSL-SCHEMA\0".to_vec(),
        complete: true,
    };
    d.bytes
        .extend_from_slice(&SCHEMA_HASH_VERSION.to_le_bytes());
    d.byte(u8::from(kind == SchemaHashKind::Strict));
    if kind == SchemaHashKind::Strict {
        d.text(&schema.name)?;
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
        if mode == 0 && schema.optional_count() > 1 && field.optional {
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
