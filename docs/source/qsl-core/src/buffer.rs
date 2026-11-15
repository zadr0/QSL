//! Storage contracts shared by generated codecs and no_std memory providers.
use crate::QslResult;
/// Exclusive output storage with an initialized readable prefix.
/// # Safety
/// Capacity addresses one stable writable allocation/region; mutable access is
/// exclusive. The initialized prefix is always valid bytes. Publishing a prefix
/// must never expose uninitialized memory. Drop preserves the provider contract.
pub unsafe trait FrameStorage {
    fn capacity(&self) -> usize;
    fn as_bytes(&self) -> &[u8];
    /// # Safety
    /// The returned capacity is exclusively writable while this borrow is active.
    unsafe fn writable_ptr(&mut self) -> *mut u8;
    /// # Safety
    /// len <= capacity and all bytes in the published prefix are initialized.
    unsafe fn set_initialized_len(&mut self, len: usize);
}
/// Obtains exclusive storage without choosing a codec or allocating globally.
pub trait FramePool {
    type Lease<'a>: FrameStorage
    where
        Self: 'a;
    fn try_acquire(&self) -> QslResult<Self::Lease<'_>>;
}
unsafe impl FrameStorage for crate::SliceOut<'_> {
    fn capacity(&self) -> usize {
        self.capacity()
    }
    fn as_bytes(&self) -> &[u8] {
        self.frame()
    }
    unsafe fn writable_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }
    unsafe fn set_initialized_len(&mut self, len: usize) {
        self.commit(len).expect("FrameStorage length contract");
    }
}

/// A provider with a const pool whose entire backing capacity is initialized.
/// # Safety
/// Every lease obtained from ZEROED must have initialized bytes throughout its
/// capacity, and writes/releases must preserve that fact for successive leases.
pub unsafe trait PreparedFramePool: FramePool {
    const ZEROED: Self;
}

/// Validate and encode into any core storage implementation, publishing only success.
pub fn write_checked<T: crate::qps_bytes::CheckedRecord, B: FrameStorage>(
    value: &T,
    out: &mut B,
) -> QslResult<usize> {
    value.qps_validate_values()?;
    let cap = value.serialized_size();
    crate::validation::check_size(cap, T::MAX_DATA_SIZE)?;
    if out.capacity() < cap {
        return Err(crate::QslError::BufferTooSmall);
    }
    let n = unsafe { value.serialize_into_unsafe(out.writable_ptr()) };
    crate::validation::check_size(n, T::MAX_DATA_SIZE)?;
    unsafe { out.set_initialized_len(n) };
    Ok(n)
}
