use crate::io::{QslError, QslResult};
use std::collections::HashSet;

pub const FLAG_COMPACT: usize = 0b0000_0001;
pub const FLAG_SAFETY: usize = 0b0000_0010;
pub const FLAG_ALIGNMENT: usize = 0b0000_0100;
pub const FLAG_OPTIONAL_BITMAP: usize = 0b0001_0000;
pub const FLAG_OPT_TABLE: usize = 0b0010_0000;
pub const FLAG_PREDICT_SEQ: usize = 0b0100_0000;
/// Internal lifetime marker for per-field byte/view borrows.
pub const FLAG_FIELD_BORROW: usize = 0b1000_0000;
pub const FLAG_LIFETIME: usize = 0b0000_1000;
pub const FLAG_STATE_BITS: usize = 1 << 8;
pub const FLAG_INLINE: usize = 1 << 9;
pub const FLAG_INLINE_ALWAYS: usize = 1 << 10;
pub const FLAG_INLINE_NEVER: usize = 1 << 11;
pub const STATE_BITS_QUANTUM_SHIFT: usize = 12;
pub const STATE_BITS_QUANTUM_MASK: usize = 7 << STATE_BITS_QUANTUM_SHIFT;
/// Internal Rust-import marker, not a QSL schema attribute.
pub const FLAG_MACRO: usize = 1 << 15;
/// Explicit compatibility mode for the old count+u32-offset directory.
pub const FLAG_OPT_TABLE_LEGACY: usize = 1 << 16;
pub const FLAG_PAYLOAD_QUANT: usize = 1 << 17;
/// Canonical QPS: exact operand packing, independent of optional prediction.
pub const FLAG_QPS: usize = FLAG_PAYLOAD_QUANT;
pub const PAYLOAD_QUANT_SHIFT: usize = 18;
pub const PAYLOAD_QUANT_MASK: usize = 3 << PAYLOAD_QUANT_SHIFT;

/// Compile-time-sized custom codec used by generated, unchecked QSL I/O.
///
/// All methods are inlined into generated serializers. No `Result` or
/// buffer-length bookkeeping is required at runtime. Implementors must obey
/// the exact wire-size contract under both normal and compact encodings.
pub trait QslCustom: Sized {
    /// Size of a normally encoded custom value in bytes.
    const SIZE: usize;

    /// Size of a compact custom value in bytes; defaults to the normal size.
    const COMP_SIZE: usize = Self::SIZE;

    /// Validate the wire representation before a safe generated decoder calls
    /// the trusted codec. Custom codecs must explicitly opt into safe decoding.
    fn qsl_validate(_bytes: &[u8], _compact: bool) -> QslResult<()> {
        Err(QslError::InvalidSchema)
    }

    /// Validate a custom value before any checked generated encoder writes it.
    /// The default refuses opaque codecs rather than authorizing unsafe I/O.
    fn qsl_validate_values(&self) -> QslResult<()> {
        Err(QslError::InvalidSchema)
    }

    /// Constant-size normal codec. Use a raw pointer store only.
    /// # Safety
    /// `to` must point to `Self::SIZE` exclusively writable bytes.
    unsafe fn qsl_write_unchecked(&self, to: *mut u8);

    /// Read an owned value without checking its input buffer.
    /// # Safety
    /// `from` must point to `Self::SIZE` readable, schema-valid bytes.
    unsafe fn qsl_read_unchecked(from: *const u8) -> Self;

    /// Compile-time normal size, retained for direct-codegen compatibility.
    #[inline(always)]
    fn qsl_size(&self) -> usize {
        Self::SIZE
    }

    /// Compile-time compact size, retained for direct-codegen compatibility.
    #[inline(always)]
    fn qsl_comp_size(&self) -> usize {
        Self::COMP_SIZE
    }

    /// Compact raw-pointer encoder (defaults to the normal encoder).
    /// # Safety
    /// `to` must have `Self::COMP_SIZE` valid writable bytes.
    unsafe fn qsl_comp_write_unchecked(&self, to: *mut u8) {
        self.qsl_write_unchecked(to);
    }

    /// Compact raw-pointer decoder (defaults to the normal decoder).
    /// # Safety
    /// `from` must have `Self::COMP_SIZE` valid readable bytes.
    unsafe fn qsl_comp_read_unchecked(from: *const u8) -> Self {
        Self::qsl_read_unchecked(from)
    }
}

mod primitive_integer_seal {
    pub trait Sealed {}
    macro_rules! integer { ($($t:ty),*) => { $(impl Sealed for $t {})* }; }
    integer!(u8, i8, u16, i16, u32, i32, u64, i64);
}
/// Supported scalar representations for the opt-in primitive macros.
#[doc(hidden)]
pub trait PrimitiveInteger: primitive_integer_seal::Sealed {}
macro_rules! primitive_integer { ($($t:ty),*) => { $(impl PrimitiveInteger for $t {})* }; }
primitive_integer!(u8, i8, u16, i16, u32, i32, u64, i64);

/// Exact transparent integer representation promised by QslPrimitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveRepr {
    pub bits: u8,
    pub signed: bool,
    pub min: i128,
    pub max: i128,
}
impl PrimitiveRepr {
    pub fn data(self) -> QslType {
        QslType::RangedInteger {
            bits: self.bits,
            signed: self.signed,
            min: self.min,
            max: self.max,
        }
    }
    pub fn validate(self) -> QslResult<()> {
        self.data().validate(false)
    }
}
/// Explicit optimization contract for transparent scalar wrappers.
/// # Safety
/// The type has exactly the declared integer's size, alignment and native bit
/// representation, has no padding or destructor, and every declared domain value
/// is valid. Bit conversion is exact, deterministic and allocation-free. Its
/// normal QslCustom codec is the same little-endian scalar representation.
pub unsafe trait QslPrimitive: QslCustom {
    const REPRESENTATION: PrimitiveRepr;
    fn qsl_to_bits(&self) -> u64;
    /// Construct through ordinary typed Rust code; this must be a total, safe conversion.
    /// Schema-domain validation is performed separately before checked materialization.
    fn qsl_from_bits(bits: u64) -> Self;
}

#[derive(Default, Debug, Clone)]
pub enum QslType {
    #[default]
    None,
    Byte,
    Integer {
        bits: u8,
        signed: bool,
        min: i128,
    },
    /// `bits` denotes the Rust storage width, not the compact rank width.
    /// COMPACT stores `value - min` using ceil(log2(max-min+1)) bits.
    RangedInteger {
        bits: u8,
        signed: bool,
        min: i128,
        max: i128,
    },
    Flags(usize),
    /// Exact UTF-8 byte count. Generates [u8; N], with no runtime length.
    FixedString(usize),
    /// Minimum character count; the wire stores the UTF-8 byte length.
    String(usize),
    /// Explicit UTF-8 byte-length range; compact length stores `len-min`.
    RangedString {
        min: usize,
        max: usize,
    },
    /// Generates `::qsl::InlineString<MAX>`; inclusive bounds count UTF-8 bytes.
    /// Enabled by `|decompress=true|`; MAX must not exceed 65535.
    CompressedString {
        min: usize,
        max: usize,
    },
    /// Generates &str borrowed from validated UTF-8; enabled by binary_opt=view.
    BorrowedString {
        min: usize,
        max: Option<usize>,
    },
    /// Owned, bounded UTF-8 storage with a raw, schema-width length header.
    InlineRawString {
        min: usize,
        max: usize,
    },
    /// A compressed fixed byte count, stored in Rust as [u8; N].
    FixedCompressedString(usize),
    Array(Box<QslType>, usize),
    RangedArray {
        element: Box<QslType>,
        min: usize,
        max: usize,
    },
    FixedArray(Box<QslType>, usize),
    Padding(usize),
    Custom {
        rust_type: String,
        fixed_wire_size: Option<usize>,
    },
}

impl QslType {
    pub fn rank_bits(min: i128, max: i128) -> QslResult<usize> {
        if max < min {
            return Err(QslError::InvalidSchema);
        }
        let delta = u64::try_from(max.checked_sub(min).ok_or(QslError::InvalidSchema)?)
            .map_err(|_| QslError::InvalidWidth)?;
        // ceil(log2(MAX-MIN+1)) == bit_length(MAX-MIN).
        Ok((u64::BITS - delta.leading_zeros()) as usize)
    }

    pub fn rank_width(&self, fallback: usize) -> QslResult<usize> {
        match self {
            Self::RangedInteger { min, max, .. } => Self::rank_bits(*min, *max),
            Self::RangedString { min, max } | Self::RangedArray { min, max, .. } => {
                Self::rank_bits(*min as i128, *max as i128)
            }
            _ => Ok(fallback),
        }
    }

    pub fn rank_base(&self) -> i128 {
        match self {
            Self::Integer { min, .. } | Self::RangedInteger { min, .. } => *min,
            Self::String(n) | Self::FixedString(n) => *n as i128,
            Self::RangedString { min, .. } | Self::BorrowedString { min, .. } => *min as i128,
            Self::Array(_, min) | Self::RangedArray { min, .. } => *min as i128,
            _ => 0,
        }
    }
    pub fn number16(signed: bool, min: usize) -> Self {
        Self::Integer {
            bits: 16,
            signed,
            min: min as i128,
        }
    }
    pub fn number32(signed: bool, min: usize) -> Self {
        Self::Integer {
            bits: 32,
            signed,
            min: min as i128,
        }
    }
    pub fn number64(signed: bool, min: usize) -> Self {
        Self::Integer {
            bits: 64,
            signed,
            min: min as i128,
        }
    }
    pub fn primitive(&self) -> QslResult<Option<&'static str>> {
        let name = match self {
            Self::None | Self::Padding(_) => "()",
            Self::Byte => "u8",
            Self::Integer { bits, signed, .. } | Self::RangedInteger { bits, signed, .. } => {
                match (*bits, *signed) {
                    (8, false) => "u8",
                    (8, true) => "i8",
                    (16, false) => "u16",
                    (16, true) => "i16",
                    (32, false) => "u32",
                    (32, true) => "i32",
                    (64, false) => "u64",
                    (64, true) => "i64",
                    _ => return Err(QslError::InvalidSchema),
                }
            }
            Self::Flags(n) => match *n {
                1..=8 => "u8",
                9..=16 => "u16",
                17..=32 => "u32",
                33..=64 => "u64",
                _ => return Err(QslError::InvalidSchema),
            },
            _ => return Ok(None),
        };
        Ok(Some(name))
    }
    pub fn rust_type(&self, borrowed: bool) -> QslResult<String> {
        if let Some(name) = self.primitive()? {
            return Ok(name.into());
        }
        match self {
            Self::FixedString(n) | Self::FixedCompressedString(n) => Ok(format!("[u8; {n}]")),

            Self::BorrowedString { .. } => Ok("&'a str".into()),

            Self::InlineRawString { max, .. } => Ok(format!("::qsl::InlineString<{max}>")),
            Self::CompressedString { max, .. } => Ok(format!("::qsl::InlineString<{max}>")),
            Self::String(_) | Self::RangedString { .. } => {
                Ok(if borrowed { "&'a str" } else { "String" }.into())
            }
            Self::Array(elem, _)
            | Self::FixedArray(elem, _)
            | Self::RangedArray { element: elem, .. } => {
                if borrowed {
                    if !matches!(elem.as_ref(), Self::Byte) {
                        return Err(QslError::InvalidSchema);
                    }
                    Ok("&'a [u8]".into())
                } else {
                    Ok(format!("Vec<{}>", elem.rust_type(false)?))
                }
            }
            Self::Custom { rust_type, .. } if !borrowed && !rust_type.is_empty() => {
                Ok(rust_type.clone())
            }
            _ => Err(QslError::InvalidSchema),
        }
    }
    pub fn fixed_wire_size(&self) -> QslResult<Option<usize>> {
        Ok(match self {
            Self::None => Some(0),
            Self::Padding(n) => Some(*n),
            Self::Byte => Some(1),
            Self::Integer { bits, .. } | Self::RangedInteger { bits, .. } => {
                self.primitive()?;
                Some(usize::from(*bits) / 8)
            }
            Self::Flags(_) => {
                let name = self.primitive()?.ok_or(QslError::InvalidSchema)?;
                Some(match name {
                    "u8" => 1,
                    "u16" => 2,
                    "u32" => 4,
                    _ => 8,
                })
            }
            Self::FixedString(n) => Some(*n),
            Self::FixedCompressedString(n) if *n < crate::STRING_COMPRESSION_MIN_BYTES => Some(*n),
            Self::FixedArray(elem, n) => elem
                .fixed_wire_size()?
                .map(|v| v.checked_mul(*n).ok_or(QslError::LengthOverflow))
                .transpose()?,
            Self::Custom {
                fixed_wire_size, ..
            } => *fixed_wire_size,
            Self::String(_)
            | Self::BorrowedString { .. }
            | Self::InlineRawString { .. }
            | Self::FixedCompressedString(_)
            | Self::RangedString { .. }
            | Self::CompressedString { .. }
            | Self::Array(_, _)
            | Self::RangedArray { .. } => None,
        })
    }
    pub fn default_compact_bits(&self) -> QslResult<usize> {
        Ok(match self {
            Self::Byte => 8,
            Self::Integer { bits, .. } => {
                self.primitive()?;
                *bits as usize
            }
            Self::RangedInteger { min, max, .. } => Self::rank_bits(*min, *max)?,
            Self::RangedString { min, max } | Self::RangedArray { min, max, .. } => {
                Self::rank_bits(*min as i128, *max as i128)?
            }
            Self::Flags(n) => {
                self.primitive()?;
                *n
            }
            _ => 32,
        })
    }
    pub fn validate(&self, borrowed: bool) -> QslResult<()> {
        self.rust_type(borrowed)?;
        if let Self::Array(inner, _)
        | Self::FixedArray(inner, _)
        | Self::RangedArray { element: inner, .. } = self
        {
            inner.validate(borrowed)?;
        }
        if let Self::RangedInteger {
            bits,
            signed,
            min,
            max,
        } = self
        {
            Self::rank_bits(*min, *max)?;
            let width = *bits as u32;
            let (low, high) = if *signed {
                (-(1i128 << (width - 1)), (1i128 << (width - 1)) - 1)
            } else {
                (0, (1i128 << width) - 1)
            };
            if *min < low || *max > high {
                return Err(QslError::InvalidSchema);
            }
        }
        if let Self::RangedString { min, max } | Self::RangedArray { min, max, .. } = self {
            if min > max || *max > u32::MAX as usize {
                return Err(QslError::InvalidSchema);
            }
        }
        if let Self::CompressedString { min, max } | Self::InlineRawString { min, max } = self {
            if min > max || *max > u16::MAX as usize {
                return Err(QslError::InvalidSchema);
            }
        }
        if let Self::BorrowedString { min, max } = self {
            if max.is_some_and(|max| max < *min || max > u32::MAX as usize) {
                return Err(QslError::InvalidSchema);
            }
        }
        if let Self::FixedCompressedString(n) = self {
            if *n > u16::MAX as usize {
                return Err(QslError::InvalidSchema);
            }
        }
        self.fixed_wire_size()?;
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct QslField {
    pub storage: crate::modernization::StorageKind,
    pub optional: bool,
    /// In COMPACT, the rank width (0..=64) of the field or its length prefix.
    pub bits: usize,
    pub data: QslType,
    pub name: String,
    /// 1-based tag. Required fields never serialize this value.
    pub tag: usize,
    /// Compile-time numeric initializer from the `|default=...|` attribute.
    /// The tag is compiler-assigned; it is not a user attribute.
    pub default: Option<i128>,
}

#[derive(Default, Debug, Clone)]
pub struct QslSchema {
    pub options: crate::modernization::SchemaOptions,
    pub flags: usize,
    pub fields: Vec<QslField>,
    pub name: String,
    pub max_tag: usize,
}

impl QslSchema {
    pub fn compact(&self) -> bool {
        self.flags & FLAG_COMPACT != 0
    }
    pub fn bitmap(&self) -> bool {
        self.flags & FLAG_OPTIONAL_BITMAP != 0
    }
    pub fn opt_table(&self) -> bool {
        self.flags & FLAG_OPT_TABLE != 0
    }
    /// QPS is lossless operand compression; predict_seq owns length prediction.
    pub fn qps(&self) -> bool {
        self.flags & FLAG_QPS != 0
    }
    pub fn quantum(&self) -> usize {
        self.payload_quantum()
    }
    pub fn quant_requested(&self) -> bool {
        self.flags & FLAG_PAYLOAD_QUANT != 0
    }
    pub fn payload_quantum(&self) -> usize {
        2 << ((self.flags & PAYLOAD_QUANT_MASK) >> PAYLOAD_QUANT_SHIFT)
    }
    pub fn legacy_opt_table(&self) -> bool {
        self.flags & FLAG_OPT_TABLE_LEGACY != 0 && self.optional_count() > 1
    }
    pub fn predict_seq(&self) -> bool {
        !self.bitmap()
            && !self.state_bits()
            && (self.flags & FLAG_PREDICT_SEQ != 0
                || (self.flags & FLAG_OPT_TABLE_LEGACY != 0 && self.optional_count() == 1))
    }
    pub fn state_bits(&self) -> bool {
        self.flags & FLAG_STATE_BITS != 0
    }
    pub fn state_bits_quantum(&self) -> usize {
        1usize << ((self.flags & STATE_BITS_QUANTUM_MASK) >> STATE_BITS_QUANTUM_SHIFT)
    }
    pub fn predict_table(&self) -> bool {
        self.predict_seq() && !self.opt_table() && self.optional_count() > 1
    }
    /// The historic length graph is selected explicitly by both mechanisms.
    pub fn predicted_opt_table(&self) -> bool {
        self.opt_table() && self.predict_seq()
    }
    pub fn sequential(&self) -> bool {
        !self.bitmap()
            && self.flags & (FLAG_OPT_TABLE_LEGACY | FLAG_PREDICT_SEQ | FLAG_STATE_BITS) != 0
    }
    pub fn lifetime(&self) -> bool {
        self.flags & (FLAG_LIFETIME | FLAG_FIELD_BORROW) != 0
    }
    pub fn borrow_strings(&self) -> bool {
        self.flags & FLAG_LIFETIME != 0
    }
    pub fn optional_count(&self) -> usize {
        self.fields.iter().filter(|f| f.optional).count()
    }
    /// Example: max_tag=8 => exactly 3 bits, storing tag-1.
    pub fn tag_bits(&self) -> usize {
        if self.max_tag < 2 {
            0
        } else {
            (usize::BITS - (self.max_tag - 1).leading_zeros()) as usize
        }
    }
    /// Number of bits needed to represent 0..=optional_count.
    pub fn count_bits(&self) -> usize {
        let count = self.optional_count();
        if count == 0 {
            0
        } else {
            (usize::BITS - count.leading_zeros()) as usize
        }
    }
    /// Bitmap v2 counts optional fields only; an empty mask has no header.
    pub fn bitmap_width(&self) -> usize {
        crate::bitmap::BitmapPlan::new(self.optional_count()).map_or(0, |p| p.width)
    }
    pub fn bitmap_bytes(&self) -> usize {
        self.bitmap_width() / 8
    }
    /// Compatibility sizing helper; use bitmap_bytes for exact wire length.
    pub fn bitmap_words(&self) -> usize {
        self.bitmap_width().div_ceil(32)
    }
    pub fn optional_tag_base(&self) -> usize {
        self.fields
            .iter()
            .find(|f| f.optional)
            .map_or(0, |f| f.tag - 1)
    }
    pub fn validate(&self) -> QslResult<()> {
        crate::modernization::validate_options(self).map_err(|_| QslError::InvalidSchema)?;
        if self.name.is_empty() || !valid_ident(&self.name) {
            return Err(QslError::InvalidSchema);
        }
        if self.flags
            & !(FLAG_COMPACT
                | FLAG_SAFETY
                | FLAG_ALIGNMENT
                | FLAG_OPTIONAL_BITMAP
                | FLAG_LIFETIME
                | FLAG_OPT_TABLE
                | FLAG_OPT_TABLE_LEGACY
                | FLAG_PREDICT_SEQ
                | FLAG_FIELD_BORROW
                | FLAG_STATE_BITS
                | FLAG_INLINE
                | FLAG_INLINE_ALWAYS
                | FLAG_INLINE_NEVER
                | FLAG_MACRO
                | STATE_BITS_QUANTUM_MASK
                | FLAG_PAYLOAD_QUANT
                | PAYLOAD_QUANT_MASK)
            != 0
        {
            return Err(QslError::InvalidSchema);
        }
        if self.optional_count() != 0 && self.max_tag == 0 {
            return Err(QslError::InvalidSchema);
        }
        if self.flags & FLAG_OPT_TABLE_LEGACY != 0
            && self.flags & (FLAG_OPT_TABLE | FLAG_PREDICT_SEQ | FLAG_STATE_BITS) != 0
        {
            return Err(QslError::InvalidSchema);
        }
        if !self.quant_requested() && self.flags & PAYLOAD_QUANT_MASK != 0 {
            return Err(QslError::InvalidSchema);
        }
        if self.max_tag > if self.bitmap() { 65535 } else { 64 }
            || self.optional_count() > if self.bitmap() { 128 } else { 64 }
        {
            return Err(QslError::InvalidSchema);
        }
        if (self.flags & (FLAG_INLINE | FLAG_INLINE_ALWAYS | FLAG_INLINE_NEVER)).count_ones() > 1
            || self.state_bits_quantum() > 16
            || (!self.state_bits() && self.state_bits_quantum() != 1)
        {
            return Err(QslError::InvalidSchema);
        }
        {
            let mut optional_seen = false;
            for field in &self.fields {
                optional_seen |= field.optional;
                if optional_seen && !field.optional {
                    return Err(QslError::InvalidSchema);
                }
            }
        }
        if self.bitmap() && self.flags & (FLAG_OPT_TABLE_LEGACY | FLAG_STATE_BITS) != 0 {
            return Err(QslError::InvalidSchema);
        }
        if self.sequential() {
            if self.bitmap() || self.compact() || (self.optional_count() == 0 && !self.opt_table())
            {
                return Err(QslError::InvalidSchema);
            }
            let mut optional_seen = false;
            for field in &self.fields {
                optional_seen |= field.optional;
                if optional_seen && !field.optional {
                    return Err(QslError::InvalidSchema);
                }
                // EOF cannot represent a present zero-byte optional value.
                if self.predict_seq()
                    && !self.opt_table()
                    && !self.predict_table()
                    && field.optional
                    && field.data.fixed_wire_size()? == Some(0)
                {
                    return Err(QslError::InvalidSchema);
                }
            }
        }
        let mut names = HashSet::new();
        let mut optional_tags = HashSet::new();
        let mut has_borrowed_data = false;
        for (index, field) in self.fields.iter().enumerate() {
            let native = self.options.transmutate
                || self
                    .options
                    .regions
                    .iter()
                    .any(|r| r.start <= index && index < r.end);
            if !valid_ident(&field.name) || !names.insert(field.name.as_str()) {
                return Err(QslError::InvalidSchema);
            }
            if field.tag == 0 || field.tag > self.max_tag {
                return Err(QslError::InvalidSchema);
            }
            if field.optional && !optional_tags.insert(field.tag) {
                return Err(QslError::InvalidSchema);
            }
            if field.bits > 64 && !matches!(&field.data, QslType::FixedString(_)) {
                return Err(QslError::InvalidWidth);
            }
            let native_custom = matches!(&field.data,QslType::Custom{rust_type,..} if self.options.native_types.contains_key(rust_type));
            field
                .data
                .validate(self.borrow_strings() && !native && !native_custom)?;
            field.data.rank_width(field.bits)?;
            if let Some(value) = field.default {
                let in_range = match &field.data {
                    QslType::Byte => (0..=255).contains(&value),
                    QslType::Integer { bits, signed, .. }
                    | QslType::RangedInteger { bits, signed, .. } => {
                        let low = if *signed { -(1i128 << (*bits - 1)) } else { 0 };
                        let high = if *signed {
                            (1i128 << (*bits - 1)) - 1
                        } else {
                            (1i128 << *bits) - 1
                        };
                        let (min, max) =
                            if let QslType::RangedInteger { min, max, .. } = &field.data {
                                (*min, *max)
                            } else {
                                (low, high)
                            };
                        (min..=max).contains(&value)
                    }
                    QslType::Flags(bits) => {
                        value >= 0 && ((*bits == 64) || ((value as u128) < (1u128 << *bits)))
                    }
                    _ => false,
                };
                if !in_range {
                    return Err(QslError::InvalidSchema);
                }
            }
            let needs_lifetime = matches!(field.data, QslType::BorrowedString { .. })
                || (self.borrow_strings()
                    && (matches!(
                        field.data,
                        QslType::String(_)
                            | QslType::RangedString { .. }
                            | QslType::FixedString(_)
                            | QslType::InlineRawString { .. }
                    ) || matches!(&field.data, QslType::Array(inner, _) | QslType::FixedArray(inner, _)
                    | QslType::RangedArray { element: inner, .. } if matches!(inner.as_ref(), QslType::Byte))));
            if needs_lifetime && !self.lifetime() {
                return Err(QslError::InvalidSchema);
            }
            has_borrowed_data |= needs_lifetime
                || (self.borrow_strings()
                    && matches!(&field.data,QslType::Custom{rust_type,..} if self.options.native_types.get(rust_type).is_some_and(|l|l.is_region)));
        }
        // Otherwise `Foo<'a>` would have an unused lifetime parameter.
        if self.lifetime() && !has_borrowed_data && !self.options.transmutate {
            return Err(QslError::InvalidSchema);
        }
        if self.quant_requested() {
            crate::payload_quant::PayloadQuantPlan::analyze(self)?;
        }
        if self.state_bits() || self.predict_table() {
            crate::static_index::PredictPlan::analyze(self)?;
        }
        Ok(())
    }
}

fn valid_ident(s: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn",
    ];
    let mut c = s.chars();
    matches!(c.next(), Some(x) if x == '_' || x.is_ascii_alphabetic())
        && c.all(|x| x == '_' || x.is_ascii_alphanumeric())
        && !KEYWORDS.contains(&s)
}

/// Attach already-validated build-time type facts before the usual schema checks.
#[doc(hidden)]
pub fn apply_primitive_bindings(
    schema: &mut QslSchema,
    bindings: &std::collections::BTreeMap<String, PrimitiveRepr>,
) -> QslResult<()> {
    for (name, p) in bindings {
        p.validate()?;
        let layout = crate::modernization::native_type_layout(&p.data(), &Default::default())
            .ok_or(QslError::InvalidSchema)?;
        schema.options.native_types.insert(name.clone(), layout);
    }
    fn bind(t: &mut QslType, bindings: &std::collections::BTreeMap<String, PrimitiveRepr>) {
        match t {
            QslType::Custom {
                rust_type,
                fixed_wire_size,
            } => {
                if let Some(p) = bindings.get(rust_type) {
                    *fixed_wire_size = Some(p.bits as usize / 8);
                }
            }
            QslType::Array(e, _)
            | QslType::FixedArray(e, _)
            | QslType::RangedArray { element: e, .. } => bind(e, bindings),
            _ => {}
        }
    }
    for f in &mut schema.fields {
        bind(&mut f.data, bindings);
    }
    Ok(())
}
