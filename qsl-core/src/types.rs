use crate::io::{QslError, QslResult};

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
