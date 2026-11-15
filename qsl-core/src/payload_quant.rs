//! Exact payload operands and typed padded regions. Q_value and Q_table are
//! independent: quotient and remainder are both retained, never rounded away.

/// Scalar wire revision, independent of the byte-envelope revision.
pub const QPS_CODEC_REVISION: u32 = 2;

/// An optimizer boundary, not a CPU fence. Keep the selected scalar address
/// opaque so LLVM cannot turn a safe load selection back into conditional I/O.
#[doc(hidden)]
#[inline(always)]
#[allow(clippy::pointers_in_nomem_asm_block)] // Pointer value only; the assembly never dereferences it.
pub fn selected_address<T>(mut value: *const T) -> *const T {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("/* {0} */",inout(reg) value,options(nomem,nostack,preserves_flags));
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        value = core::hint::black_box(value);
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
        core::arch::asm!("/* {0} */",inout(reg) value,options(nomem,nostack,preserves_flags));
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        value = core::hint::black_box(value);
    }
    value
}

#[cfg(feature = "alloc")]
use crate::SerOut;
use crate::{FrameStorage, QslError, QslResult, Serializable};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::marker::PhantomData;

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
    pub fn decode_into<'out>(&self, out: &'out mut core::mem::MaybeUninit<T>) -> &'out mut T {
        unsafe {
            T::quant_read_padded(self.storage.as_ptr(), self.len, out.as_mut_ptr());
            out.assume_init_mut()
        }
    }
    #[cfg(feature = "alloc")]
    pub fn decode_boxed(&self) -> Box<T> {
        let mut out = Box::<T>::new_uninit();
        unsafe {
            T::quant_read_padded(self.storage.as_ptr(), self.len, out.as_mut_ptr());
            out.assume_init()
        }
    }
}

#[cfg(feature = "alloc")]
mod owned {
    use super::*;
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
    impl<T: QuantizedRecord> core::ops::Deref for QuantizedOut<T> {
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
}
#[cfg(feature = "alloc")]
pub use owned::QuantizedOut;

pub struct QuantizedRegions<T: QuantizedRecord, P: crate::buffer::PreparedFramePool> {
    pool: P,
    marker: PhantomData<fn() -> T>,
}
impl<T: QuantizedRecord, P: crate::buffer::PreparedFramePool> Default for QuantizedRegions<T, P> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: QuantizedRecord, P: crate::buffer::PreparedFramePool> QuantizedRegions<T, P> {
    pub const fn new() -> Self {
        Self {
            pool: P::ZEROED,
            marker: PhantomData,
        }
    }
    pub fn encode<'a>(&'a self, value: &T) -> QslResult<QuantizedLease<T, P::Lease<'a>>> {
        value.quant_validate_values()?;
        let expected = value.serialized_size();
        crate::validation::check_size(expected, T::MAX_DATA_SIZE)?;
        let end = expected
            .checked_add(T::PADDING_BYTES)
            .ok_or(QslError::LengthOverflow)?;
        let mut storage = self.pool.try_acquire()?;
        if end > storage.capacity() {
            return Err(QslError::BufferTooSmall);
        }
        let len = unsafe { value.quant_write_prepared(storage.writable_ptr()) };
        debug_assert_eq!(len, expected);
        unsafe { storage.set_initialized_len(len + T::PADDING_BYTES) };
        Ok(QuantizedLease {
            storage,
            len,
            marker: PhantomData,
        })
    }
}
pub struct QuantizedLease<T: QuantizedRecord, B: FrameStorage> {
    storage: B,
    len: usize,
    marker: PhantomData<fn() -> T>,
}
impl<T: QuantizedRecord, B: FrameStorage> QuantizedLease<T, B> {
    pub fn as_slice(&self) -> &[u8] {
        &self.storage.as_bytes()[..self.len]
    }
    pub fn frame(&self) -> QuantizedFrame<'_, T> {
        QuantizedFrame {
            storage: self.storage.as_bytes(),
            len: self.len,
            marker: PhantomData,
        }
    }
}
impl<T: QuantizedRecord, B: FrameStorage> core::ops::Deref for QuantizedLease<T, B> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl<T: QuantizedRecord, B: FrameStorage> AsRef<[u8]> for QuantizedLease<T, B> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<T: QuantizedRecord, B: crate::OutputBuffer> QuantizedLease<T, B> {
    pub fn encode_owned(value: &T) -> QslResult<Self> {
        value.quant_validate_values()?;
        let len = value.serialized_size();
        crate::validation::check_size(len, T::MAX_DATA_SIZE)?;
        let capacity = len
            .checked_add(T::PADDING_BYTES)
            .ok_or(QslError::LengthOverflow)?;
        let mut storage = B::try_with_capacity(capacity)?;
        unsafe {
            let pointer = storage.writable_ptr();
            pointer.write_bytes(0, capacity);
            let written = value.quant_write_prepared(pointer);
            debug_assert_eq!(written, len);
            storage.set_initialized_len(len + T::PADDING_BYTES);
        }
        Ok(Self {
            storage,
            len,
            marker: PhantomData,
        })
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
