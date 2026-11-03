//! Exact payload operands and typed padded regions. Q_value and Q_table are
//! independent: quotient and remainder are both retained, never rounded away.
use crate::{QslError, QslResult, QslSchema, QslType, SerOut, Serializable};
use std::marker::PhantomData;

/// An optimizer boundary, not a CPU fence. Keep the selected scalar address
/// opaque so LLVM cannot turn a safe load selection back into conditional I/O.
#[doc(hidden)]
#[inline(always)]
#[allow(clippy::pointers_in_nomem_asm_block)] // Pointer value only; the assembly never dereferences it.
pub fn selected_address<T>(mut value: *const T) -> *const T {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::asm!("/* {0} */",inout(reg) value,options(nomem,nostack,preserves_flags));
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        value = std::hint::black_box(value);
    }
    value
}
/// Keep computation unconditional even when Option's unused payload would let
/// LLVM sink it into a present-only block. The x86-64 instruction is a comment.
#[doc(hidden)]
#[inline(always)]
pub fn scalar_operand(mut value: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::asm!("/* {0} */",inout(reg) value,options(nomem,nostack,preserves_flags));
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        value = std::hint::black_box(value);
    }
    value
}

#[derive(Debug, Clone)]
pub struct QuantField {
    pub index: usize,
    pub min: i128,
    pub max: i128,
    pub bits: usize,
    pub native_bytes: usize,
    pub rust_type: String,
    pub optional: bool,
    pub word_bytes: usize,
    pub max_start: usize,
}
#[derive(Debug, Clone)]
pub struct PayloadQuantPlan {
    pub quantum: usize,
    pub padding_bytes: usize,
    pub fields: Vec<QuantField>,
    pub header_bytes: usize,
    pub max_bytes: usize,
    pub min_bytes: usize,
    pub raw_max_bytes: Option<usize>,
    pub active: bool,
    pub reason: Option<&'static str>,
}
impl PayloadQuantPlan {
    pub fn analyze(s: &QslSchema) -> QslResult<Self> {
        let q = s.payload_quantum();
        let n = s.optional_count();
        let mut p = Self {
            quantum: q,
            padding_bytes: 1,
            fields: Vec::new(),
            header_bytes: if s.state_bits() { 4 } else { (n + 7) >> 3 },
            max_bytes: 0,
            min_bytes: 0,
            raw_max_bytes: None,
            active: false,
            reason: Some("payload quantization disabled"),
        };
        if !s.quant_requested() {
            return Ok(p);
        }
        if n > 64 {
            return Err(QslError::InvalidSchema);
        }
        if s.compact()
            || s.bitmap()
            || s.flags & crate::FLAG_ALIGNMENT != 0
            || s.flags & crate::FLAG_OPT_TABLE_LEGACY != 0
        {
            return Err(QslError::InvalidSchema);
        }
        let mut full_bits = 0usize;
        let mut required_bits = 0usize;
        let mut native = 0usize;
        let mut residues = 1u8;
        for (index, f) in s.fields.iter().enumerate() {
            f.data.validate(s.borrow_strings())?;
            let (min, max) = match f.data {
                QslType::Byte => (0, 255),
                QslType::RangedInteger { min, max, .. } => (min, max),
                QslType::Integer {
                    bits,
                    signed,
                    min: 0,
                } => {
                    if signed {
                        (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
                    } else {
                        (0, (1i128 << bits) - 1)
                    }
                }
                QslType::Flags(bits) => (0, (1i128 << bits) - 1),
                _ => {
                    p.reason = Some("non-numeric payload: preserve the explicit original codec");
                    return Ok(p);
                }
            };
            let bits = QslType::rank_bits(min, max)?;
            let bytes = f.data.fixed_wire_size()?.ok_or(QslError::InvalidSchema)?;
            full_bits += bits;
            if !f.optional {
                required_bits += bits;
            }
            native += bytes;
            let max_start = 7 - residues.leading_zeros() as usize;
            let word_bytes = [1, 2, 4, 8, 16]
                .into_iter()
                .find(|&w| w <= q && w * 8 >= bits + max_start)
                .unwrap_or(q);
            p.padding_bytes = p.padding_bytes.max(word_bytes);
            p.fields.push(QuantField {
                index,
                min,
                max,
                bits,
                native_bytes: bytes,
                rust_type: f.data.primitive()?.ok_or(QslError::InvalidSchema)?.into(),
                optional: f.optional,
                word_bytes,
                max_start,
            });
            let next = residues.rotate_left((bits & 7) as u32);
            residues = if f.optional { residues | next } else { next };
        }
        let raw_header = if s.state_bits() {
            4
        } else if s.opt_table() {
            crate::static_optional::StaticOptionalPlan::analyze(s)?.header_bytes()
        } else if s.predict_seq() || n == 0 {
            0
        } else if n == 1 {
            1
        } else {
            (s.count_bits() + n * s.tag_bits() + 7) >> 3
        };
        p.raw_max_bytes = Some(raw_header + native);
        p.max_bytes = p.header_bytes + ((full_bits + 7) >> 3);
        p.min_bytes = p.header_bytes + ((required_bits + 7) >> 3);
        if p.max_bytes
            .checked_add(p.padding_bytes)
            .is_none_or(|size| size >= raw_header + native)
        {
            p.reason =
                Some("no physical buffer saving after exact operands and padding; raw fallback");
            return Ok(p);
        }
        if s.predict_seq() || s.state_bits() && s.state_bits_quantum() != 1 {
            return Err(QslError::InvalidSchema);
        }
        p.active = true;
        p.reason = None;
        Ok(p)
    }
}

/// # Safety
/// The implementation writes only initialized scalar fields, inverts its exact
/// operand codec, respects PADDING_BYTES, and checks the full numeric domain.
pub unsafe trait QuantizedRecord: Serializable {
    const MAX_DATA_SIZE: usize = crate::validation::DEFAULT_MAX_DATA_SIZE;
    const QUANTUM: usize;
    const PADDING_BYTES: usize;
    const MAX_QUANTIZED_BYTES: usize;
    fn quant_validate_values(&self) -> QslResult<()>;
    /// # Safety
    /// out has serialized_size()+PADDING_BYTES initialized, exclusive bytes. Values
    /// are schema-valid and out is disjoint from self. Padding is not on wire.
    unsafe fn quant_write_prepared(&self, out: *mut u8) -> usize;
    /// # Safety
    /// bytes has len+PADDING_BYTES readable initialized bytes; its first len bytes
    /// are schema-valid. out is aligned, uninitialized/disjoint storage for Self.
    unsafe fn quant_read_padded(bytes: *const u8, len: usize, out: *mut Self);
}
pub trait CheckedQuantizedRecord: QuantizedRecord {
    fn quant_validate(input: &[u8]) -> QslResult<()>;
}

pub struct QuantizedFrame<'a, T: QuantizedRecord> {
    storage: &'a [u8],
    len: usize,
    marker: PhantomData<fn() -> T>,
}
impl<'a, T: CheckedQuantizedRecord> QuantizedFrame<'a, T> {
    pub fn try_new(storage: &'a [u8], len: usize) -> QslResult<Self> {
        let end = len
            .checked_add(T::PADDING_BYTES)
            .ok_or(QslError::LengthOverflow)?;
        let storage = storage.get(..end).ok_or(QslError::BufferTooSmall)?;
        T::quant_validate(&storage[..len])?;
        Ok(Self {
            storage,
            len,
            marker: PhantomData,
        })
    }
}
impl<T: QuantizedRecord> QuantizedFrame<'_, T> {
    /// # Safety
    /// The first len bytes are a schema-valid frame and the following
    /// T::PADDING_BYTES bytes are initialized and readable.
    pub unsafe fn from_validated(storage: &[u8], len: usize) -> QuantizedFrame<'_, T> {
        QuantizedFrame {
            storage,
            len,
            marker: PhantomData,
        }
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.storage[..self.len]
    }
    pub fn decode_into<'out>(&self, out: &'out mut std::mem::MaybeUninit<T>) -> &'out mut T {
        unsafe {
            T::quant_read_padded(self.storage.as_ptr(), self.len, out.as_mut_ptr());
            out.assume_init_mut()
        }
    }
    pub fn decode_boxed(&self) -> Box<T> {
        let mut out = Box::<T>::new_uninit();
        unsafe {
            T::quant_read_padded(self.storage.as_ptr(), self.len, out.as_mut_ptr());
            out.assume_init()
        }
    }
}

pub struct QuantizedOut<T: QuantizedRecord> {
    storage: SerOut,
    len: usize,
    marker: PhantomData<fn() -> T>,
}
impl<T: QuantizedRecord> QuantizedOut<T> {
    pub fn encode(value: &T) -> QslResult<Self> {
        value.quant_validate_values()?;
        let expected = value.serialized_size();
        crate::validation::check_size(expected, T::MAX_DATA_SIZE)?;
        let cap = expected
            .checked_add(T::PADDING_BYTES)
            .ok_or(QslError::LengthOverflow)?;
        let mut storage = SerOut::new(cap);
        let len = unsafe { value.quant_write_prepared(storage.as_mut_ptr()) };
        debug_assert_eq!(len, expected);
        Ok(Self {
            storage,
            len,
            marker: PhantomData,
        })
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.storage[..self.len]
    }
    pub fn frame(&self) -> QuantizedFrame<'_, T> {
        QuantizedFrame {
            storage: &self.storage[..self.len + T::PADDING_BYTES],
            len: self.len,
            marker: PhantomData,
        }
    }
}
impl<T: QuantizedRecord> std::ops::Deref for QuantizedOut<T> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl<T: QuantizedRecord> AsRef<[u8]> for QuantizedOut<T> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

pub struct QuantizedRegions<T: QuantizedRecord, const CAP: usize, const N: usize> {
    pool: crate::rt::StaticRegions<CAP, N>,
    marker: PhantomData<fn() -> T>,
}
impl<T: QuantizedRecord, const CAP: usize, const N: usize> Default for QuantizedRegions<T, CAP, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: QuantizedRecord, const CAP: usize, const N: usize> QuantizedRegions<T, CAP, N> {
    pub const fn new() -> Self {
        Self {
            pool: crate::rt::StaticRegions::new_zeroed(),
            marker: PhantomData,
        }
    }
    pub fn try_encode(&'static self, value: &T) -> QslResult<QuantizedLease<T, CAP, N>> {
        value.quant_validate_values()?;
        crate::validation::check_size(value.serialized_size(), T::MAX_DATA_SIZE)?;
        if value
            .serialized_size()
            .checked_add(T::PADDING_BYTES)
            .is_none_or(|n| n > CAP)
        {
            return Err(QslError::BufferTooSmall);
        }
        let mut storage = self.pool.try_acquire()?;
        let len = unsafe { value.quant_write_prepared(storage.as_mut_ptr()) };
        unsafe {
            storage.set_len(len + T::PADDING_BYTES);
        }
        Ok(QuantizedLease {
            storage,
            len,
            marker: PhantomData,
        })
    }
}
pub struct QuantizedLease<T: QuantizedRecord, const CAP: usize, const N: usize> {
    storage: crate::rt::RegionLease<CAP, N>,
    len: usize,
    marker: PhantomData<fn() -> T>,
}
impl<T: QuantizedRecord, const CAP: usize, const N: usize> QuantizedLease<T, CAP, N> {
    pub fn as_slice(&self) -> &[u8] {
        &self.storage[..self.len]
    }
    pub fn frame(&self) -> QuantizedFrame<'_, T> {
        QuantizedFrame {
            storage: &self.storage,
            len: self.len,
            marker: PhantomData,
        }
    }
}
impl<T: QuantizedRecord, const CAP: usize, const N: usize> std::ops::Deref
    for QuantizedLease<T, CAP, N>
{
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl<T: QuantizedRecord, const CAP: usize, const N: usize> AsRef<[u8]>
    for QuantizedLease<T, CAP, N>
{
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

pub trait QuantizedDestination<T: QuantizedRecord> {
    /// # Safety
    /// Input has len+T::PADDING_BYTES readable, initialized bytes. Raw destinations
    /// are aligned/uninitialized; references are initialized scalar-only records.
    unsafe fn decode_padded(self, bytes: *const u8, len: usize);
}
impl<T: QuantizedRecord> QuantizedDestination<T> for *mut T {
    unsafe fn decode_padded(self, bytes: *const u8, len: usize) {
        T::quant_read_padded(bytes, len, self);
    }
}
impl<T: QuantizedRecord> QuantizedDestination<T> for &mut T {
    unsafe fn decode_padded(self, bytes: *const u8, len: usize) {
        T::quant_read_padded(bytes, len, self);
    }
}
