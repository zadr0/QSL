/// Flat, trusted record serialization. No dynamic dispatch, runtime validation
/// or `Result` is part of this API. The caller supplies schema-valid data.
pub trait Serializable: Sized {
    /// Number of optional fields declared in the schema.
    const OPTIONAL_FIELDS_COUNT: usize;
    /// Number of tag bits; zero for a bitmap or a single optional field.
    const SIZE_TAG: usize;

    /// Smallest possible encoded size determined at build time.
    fn get_size() -> usize;

    /// Required writable capacity. Exact for ordinary fields; records with
    /// compressed strings return the raw-size upper bound. Use the count from
    /// `serialize_into_unsafe`/`serialize_to` as the initialized wire length.
    fn serialized_size(&self) -> usize;

    /// Allocate the final buffer once and write directly through its pointer.
    /// # Safety
    /// Every field must satisfy its declared QSL range and exact-length contract.
    /// The combined encoded size must fit isize::MAX.
    #[cfg(feature = "native")]
    unsafe fn serialize(&self) -> SerOut {
        let mut out = unsafe { SerOut::uninit(self.serialized_size()) };
        unsafe {
            let written = self.serialize_into_unsafe(out.as_mut_ptr());
            out.len = written;
        }
        out
    }

    /// Single-allocation Vec encoder. No intermediate buffers or validations.
    /// Each encoded byte is initialized by the writer, avoiding a zero-fill pass.
    /// # Safety
    /// Every field must satisfy its declared QSL range and exact-length contract.
    /// The combined encoded size must fit isize::MAX.
    #[cfg(not(feature = "native"))]
    unsafe fn serialize(&self) -> Vec<u8> {
        let len = self.serialized_size();
        let mut out = Vec::with_capacity(len);
        unsafe {
            let written = self.serialize_into_unsafe(out.as_mut_ptr());
            out.set_len(written);
        }
        out
    }

    /// # Safety
    /// `to` must address `serialized_size()` writable, nonoverlapping bytes.
    /// Every field must obey the QSL bounds and fixed byte lengths.
    unsafe fn serialize_unsafe(&self, to: *mut u8);
    /// Encode into raw capacity and return the exact initialized wire length.
    /// Compressed records override this to avoid a preliminary compression pass.
    /// # Safety
    /// Same field, capacity and nonoverlap contract as `serialize_unsafe`.
    #[inline(always)]
    unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize {
        self.serialize_unsafe(to);
        self.serialized_size()
    }

    /// Store into a preallocated `SerOut` without comparing capacities.
    /// # Safety
    /// `to` must have at least `self.serialized_size()` writable bytes.
    #[cfg(feature = "native")]
    unsafe fn serialize_to(&self, to: &mut SerOut) -> usize {
        let written = self.serialize_into_unsafe(to.as_mut_ptr());
        to.len = written;
        written
    }

    /// Store into a preallocated byte slice without comparing capacities.
    /// # Safety
    /// `to` must have at least `self.serialized_size()` writable bytes.
    #[cfg(not(feature = "native"))]
    unsafe fn serialize_to(&self, to: &mut [u8]) -> usize {
        self.serialize_into_unsafe(to.as_mut_ptr())
    }

    /// Convenience pointer-based decoder with a borrowed-slice argument.
    /// # Safety
    /// `input` must contain a complete, valid record matching the schema.
    unsafe fn deserialize(input: &[u8]) -> Self {
        Self::deserialize_from_raw_parts(input.as_ptr(), input.len())
    }

    /// Decode into final native storage. Generated inline fields avoid return-value copies.
    /// # Safety
    /// bytes addresses len bytes of one complete valid record. out is aligned, writable,
    /// uninitialized storage for Self (or its previous value was dropped), nonoverlapping input.
    #[inline(always)]
    unsafe fn deserialize_into(bytes: *const u8, len: usize, out: *mut Self) {
        out.write(Self::deserialize_from_raw_parts(bytes, len));
    }

    /// Decode into an initialized value, retaining prepared CopyGuard buffers where generated.
    /// Other layouts replace the value and may allocate normally.
    /// # Safety
    /// input must be one schema-valid record and not overlap any destination storage.
    /// For a generated reuse layout, every CopyGuard capacity must fit its decoded payload.
    #[inline(always)]
    unsafe fn deserialize_reuse(input: &[u8], output: &mut Self) {
        *output = Self::deserialize(input);
    }
    /// Decode into an initialized destination. Prepared buffers are retained where generated.
    /// # Safety
    /// bytes/len describe a schema-valid record, disjoint from output and its storage.
    /// Every retained destination buffer must have capacity for its decoded field.
    #[inline(always)]
    unsafe fn deserialize_prepared_into(bytes: *const u8, len: usize, output: &mut Self) {
        Self::deserialize_reuse(std::slice::from_raw_parts(bytes, len), output);
    }
    /// True when the generated decoder updates required buffer/array fields directly.
    const REUSES_STORAGE: bool = false;

    /// # Safety
    /// `bytes` must point to a complete, valid schema-encoded record.
    unsafe fn deserialize_unsafe(bytes: *const u8) -> Self;
    /// Length-aware trusted decoder. Required for predict_seq tail presence.
    /// # Safety
    /// `bytes` is valid for `len` readable bytes forming exactly one schema-valid record.
    #[inline(always)]
    unsafe fn deserialize_from_raw_parts(bytes: *const u8, _len: usize) -> Self {
        Self::deserialize_unsafe(bytes)
    }
}

/// Zero-copy records borrow their string/byte payloads from the input memory.
/// The backing allocation must remain alive for the complete `'a` lifetime.
pub trait BorrowSerializable<'a>: Sized {
    /// Number of optional fields declared in the schema.
    const OPTIONAL_FIELDS_COUNT: usize;
    /// Number of tag bits, or zero for a bitmap/single optional field.
    const SIZE_TAG: usize;
    /// Minimum encoded size including required fields and header bytes.
    fn get_size() -> usize;
    /// Required output capacity; compressed strings use their raw-size bound.
    fn serialized_size(&self) -> usize;

    /// Allocate once and encode borrowed fields without a zero-fill pass.
    /// # Safety
    /// Values must satisfy all QSL bounds and exact byte lengths.
    /// The combined encoded size must fit isize::MAX.
    #[cfg(feature = "native")]
    unsafe fn serialize(&self) -> SerOut {
        let mut out = unsafe { SerOut::uninit(self.serialized_size()) };
        unsafe {
            let written = self.serialize_into_unsafe(out.as_mut_ptr());
            out.len = written;
        }
        out
    }
    #[cfg(not(feature = "native"))]
    /// Allocate once and encode into bytes initialized by the generated writer.
    /// # Safety
    /// Values must satisfy all QSL bounds and exact byte lengths.
    /// The combined encoded size must fit isize::MAX.
    unsafe fn serialize(&self) -> Vec<u8> {
        let len = self.serialized_size();
        let mut out = Vec::with_capacity(len);
        unsafe {
            let written = self.serialize_into_unsafe(out.as_mut_ptr());
            out.set_len(written);
        }
        out
    }
    /// # Safety
    /// `to` must address `self.serialized_size()` exclusively writable bytes.
    /// Every field must obey the QSL bounds and fixed byte lengths.
    unsafe fn serialize_unsafe(&self, to: *mut u8);
    /// Encode into raw capacity and return the exact initialized wire length.
    /// Compressed records override this to avoid a preliminary compression pass.
    /// # Safety
    /// Same field, capacity and nonoverlap contract as `serialize_unsafe`.
    #[inline(always)]
    unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize {
        self.serialize_unsafe(to);
        self.serialized_size()
    }

    /// # Safety
    /// `to` must contain the complete writable output region.
    #[cfg(feature = "native")]
    unsafe fn serialize_to(&self, to: &mut SerOut) -> usize {
        let written = self.serialize_into_unsafe(to.as_mut_ptr());
        to.len = written;
        written
    }

    /// # Safety
    /// `to` must contain the complete writable output region.
    #[cfg(not(feature = "native"))]
    unsafe fn serialize_to(&self, to: &mut [u8]) -> usize {
        self.serialize_into_unsafe(to.as_mut_ptr())
    }

    /// # Safety
    /// `input` must be a complete, valid QSL record for the lifetime `'a`.
    unsafe fn deserialize_borrowed(input: &'a [u8]) -> Self {
        Self::deserialize_borrowed_from_raw_parts(input.as_ptr(), input.len())
    }

    /// Decode a borrowed record into uninitialized native storage.
    /// # Safety
    /// bytes addresses len schema-valid bytes alive for 'a. out is aligned, writable,
    /// uninitialized storage for Self (or its old value was dropped), nonoverlapping input.
    #[inline(always)]
    unsafe fn deserialize_borrowed_into(bytes: *const u8, len: usize, out: *mut Self) {
        out.write(Self::deserialize_borrowed_from_raw_parts(bytes, len));
    }

    /// # Safety
    /// Caller must guarantee that `bytes` points to a complete valid record
    /// and remains alive for `'a`.
    unsafe fn deserialize_borrowed_unsafe(bytes: *const u8) -> Self;
    /// Length-aware trusted decoder. The input remains alive for 'a.
    /// # Safety
    /// `bytes` is valid for `len` readable bytes forming exactly one schema-valid record.
    #[inline(always)]
    unsafe fn deserialize_borrowed_from_raw_parts(bytes: *const u8, _len: usize) -> Self {
        Self::deserialize_borrowed_unsafe(bytes)
    }
}

pub struct SerOut {
    ptr: std::ptr::NonNull<u8>,
    layout: std::alloc::Layout,
    len: usize,
}

impl SerOut {
    /// Allocate initialized zero-filled bytes using the global Rust allocator.
    pub fn new(len: usize) -> Self {
        use std::alloc::{alloc_zeroed, handle_alloc_error, Layout};
        // alloc_zeroed is not valid for a zero-sized layout.
        if len == 0 {
            return Self::with_capacity_aligned(0, 4);
        }
        let layout = Layout::from_size_align(len, 4).expect("QSL: allocation overflow");
        let ptr = unsafe { alloc_zeroed(layout) };
        let ptr = std::ptr::NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(layout));
        Self { ptr, layout, len }
    }
    /// Allocate output which must be completely initialized before observation.
    /// Used only by the generated encoder's single-write allocation path.
    #[cfg(feature = "native")]
    unsafe fn uninit(len: usize) -> Self {
        Self::with_capacity_aligned(len, 4)
    }
    /// Allocate reusable storage with architecture-selected power-of-two alignment.
    /// The initialized slice is initially empty; capacity is retained after reuse.
    pub fn with_capacity_aligned(capacity: usize, alignment: usize) -> Self {
        use std::alloc::{alloc, handle_alloc_error, Layout};
        let layout =
            Layout::from_size_align(capacity, alignment).expect("QSL: invalid buffer layout");
        let ptr = if capacity == 0 {
            std::ptr::NonNull::new(alignment as *mut u8).unwrap()
        } else {
            let raw = unsafe { alloc(layout) };
            std::ptr::NonNull::new(raw).unwrap_or_else(|| handle_alloc_error(layout))
        };
        Self {
            ptr,
            layout,
            len: 0,
        }
    }
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.layout.size()
    }
    #[inline(always)]
    pub fn alignment(&self) -> usize {
        self.layout.align()
    }
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }
    /// Publish bytes initialized by a native producer, e.g. a socket receive.
    /// # Safety
    /// The first len bytes must be initialized and len must not exceed capacity.
    #[inline(always)]
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity());
        self.len = len;
    }
    /// Takes ownership of a caller allocation with the exact given layout.
    /// # Safety
    /// Must exclusively own an allocation obtained with the same layout/global allocator.
    /// The first len bytes must be initialized and len must not exceed layout.size().
    pub unsafe fn from_raw(ptr: *mut u8, layout: std::alloc::Layout, len: usize) -> Self {
        assert!(len <= layout.size() && layout.size() > 0);
        Self {
            ptr: std::ptr::NonNull::new(ptr).expect("QSL: null pointer"),
            layout,
            len,
        }
    }
    /// Return the stable output allocation pointer; does not advance a cursor.
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}
impl std::ops::Deref for SerOut {
    type Target = [u8];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}
impl std::ops::DerefMut for SerOut {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}
impl Drop for SerOut {
    fn drop(&mut self) {
        unsafe {
            if self.layout.size() != 0 {
                std::alloc::dealloc(self.ptr.as_ptr(), self.layout);
            }
        }
    }
}

// SerOut uniquely owns a global-allocator byte allocation. Only &mut exposes mutation.
unsafe impl Send for SerOut {}
unsafe impl Sync for SerOut {}
impl Default for SerOut {
    fn default() -> Self {
        Self::with_capacity_aligned(0, 1)
    }
}
impl std::fmt::Debug for SerOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerOut")
            .field("len", &self.len)
            .field("capacity", &self.capacity())
            .finish()
    }
}
impl SerOut {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Copy bytes into retained storage. Failure preserves the previous initialized contents.
    #[inline]
    pub fn copy_from_slice(&mut self, bytes: &[u8]) -> crate::QslResult<()> {
        if bytes.len() > self.capacity() {
            return Err(crate::QslError::BufferTooSmall);
        }
        unsafe {
            self.copy_from_raw_parts(bytes.as_ptr(), bytes.len());
        }
        Ok(())
    }
    /// Copy exactly len bytes, publishing the initialized prefix after the copy.
    /// # Safety
    /// source addresses len initialized, readable bytes disjoint from this allocation;
    /// len <= capacity. No alias may mutate source during the copy.
    #[inline(always)]
    pub unsafe fn copy_from_raw_parts(&mut self, source: *const u8, len: usize) {
        debug_assert!(len <= self.capacity());
        std::ptr::copy_nonoverlapping(source, self.ptr.as_ptr(), len);
        self.len = len;
    }
    pub fn spare_capacity_mut(&mut self) -> &mut [std::mem::MaybeUninit<u8>] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.ptr.as_ptr().add(self.len).cast(),
                self.capacity() - self.len,
            )
        }
    }
    /// Transfer ownership of an initialized UTF-8 buffer, without copying/allocating.
    /// Failure returns the original storage with the validation error.
    pub fn into_utf8_guard(self) -> Result<crate::Utf8CopyGuard, (std::str::Utf8Error, Self)> {
        crate::Utf8CopyGuard::try_from_ser_out(self)
    }
}

mod destination_private {
    pub trait Sealed {}
    impl<T> Sealed for *mut T {}
    impl<T> Sealed for &mut T {}
}
/// Statically selected destination of a generated type's deserialize_into entrypoint.
/// *mut T initializes fresh storage; &mut T reuses an already initialized value.
/// External implementations are sealed to keep both ownership contracts explicit.
pub trait DeserializeDestination<T: Serializable>: destination_private::Sealed {
    /// # Safety
    /// Input is a complete schema-valid record, disjoint from destination storage.
    /// A raw pointer must address aligned uninitialized storage for T. A prepared
    /// mutable destination must retain sufficient capacity for every reused field.
    unsafe fn decode_into(self, bytes: *const u8, len: usize);
}
impl<T: Serializable> DeserializeDestination<T> for *mut T {
    #[inline(always)]
    unsafe fn decode_into(self, bytes: *const u8, len: usize) {
        <T as Serializable>::deserialize_into(bytes, len, self);
    }
}
impl<T: Serializable> DeserializeDestination<T> for &mut T {
    #[inline(always)]
    unsafe fn decode_into(self, bytes: *const u8, len: usize) {
        T::deserialize_prepared_into(bytes, len, self);
    }
}
