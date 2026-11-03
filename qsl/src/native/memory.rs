use std::{
    alloc::Layout,
    cell::Cell,
    io,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    rc::Rc,
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Arc,
    },
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, INVALID_HANDLE_VALUE},
    System::{
        Memory::*,
        SystemInformation::{GetSystemInfo, SYSTEM_INFO},
    },
};

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
fn exhausted() -> io::Error {
    io::Error::new(io::ErrorKind::OutOfMemory, "region capacity exhausted")
}
fn align_up(value: usize, align: usize) -> Option<usize> {
    value.checked_add(align - 1).map(|n| n & !(align - 1))
}
fn page_size() -> usize {
    let mut info = MaybeUninit::<SYSTEM_INFO>::uninit();
    unsafe {
        GetSystemInfo(info.as_mut_ptr());
        info.assume_init().dwPageSize as usize
    }
}

/// Committed virtual memory with a single-threaded bump cursor and no per-block heap allocation.
/// Borrowed allocations prevent reset/drop while they are accessible.
///
/// ```compile_fail
/// use qsl::native::NativeRegion;
/// let mut region = NativeRegion::new(4096).unwrap();
/// let mut buffer = region.buffer(1024, 64).unwrap();
/// region.reset(); // the allocation is still borrowed below
/// buffer.write_from_slice(b"live").unwrap();
/// ```
///
/// ```compile_fail
/// use qsl::native::NativeRegion;
/// let region = NativeRegion::new(4096).unwrap();
/// std::thread::spawn(move || drop(region)); // deliberately neither Send nor Sync
/// ```
pub struct NativeRegion {
    ptr: NonNull<u8>,
    capacity: usize,
    cursor: Cell<usize>,
    _local: PhantomData<Rc<()>>,
}
impl NativeRegion {
    pub fn new(capacity: usize) -> io::Result<Self> {
        if capacity == 0 || capacity > isize::MAX as usize {
            return Err(invalid("invalid region capacity"));
        }
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null(),
                capacity,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_READWRITE,
            )
        };
        Ok(Self {
            ptr: NonNull::new(ptr.cast()).ok_or_else(io::Error::last_os_error)?,
            capacity,
            cursor: Cell::new(0),
            _local: PhantomData,
        })
    }
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn used(&self) -> usize {
        self.cursor.get()
    }
    /// Reuse all storage. Rust borrows require existing buffers/values to be released first.
    pub fn reset(&mut self) {
        self.cursor.set(0);
    }
    /// Touch committed pages before allocating/serving; previous storage contents are discarded.
    /// Does not lock RAM or alter OS policy. Requires exclusive access to the region.
    pub fn prefault(&mut self) {
        for offset in (0..self.capacity).step_by(page_size()) {
            unsafe {
                let p = self.ptr.as_ptr().add(offset);
                p.write_volatile(0);
            }
        }
    }
    fn allocate(&self, layout: Layout) -> io::Result<NonNull<u8>> {
        let base = self.ptr.as_ptr() as usize;
        let start = align_up(
            base.checked_add(self.cursor.get()).ok_or_else(exhausted)?,
            layout.align(),
        )
        .and_then(|p| p.checked_sub(base))
        .ok_or_else(exhausted)?;
        let end = start
            .checked_add(layout.size().max(1))
            .ok_or_else(exhausted)?;
        if end > self.capacity {
            return Err(exhausted());
        }
        self.cursor.set(end);
        Ok(unsafe { NonNull::new_unchecked(self.ptr.as_ptr().add(start)) })
    }
    pub fn buffer(&self, capacity: usize, alignment: usize) -> io::Result<RegionBuffer<'_>> {
        let layout = Layout::from_size_align(capacity.max(1), alignment)
            .map_err(|_| invalid("invalid buffer layout"))?;
        Ok(RegionBuffer {
            ptr: self.allocate(layout)?,
            capacity,
            len: 0,
            _region: PhantomData,
        })
    }
    /// Storage for decoding directly into the final object, without an intermediate object copy.
    pub fn slot<T>(&self) -> io::Result<RegionSlot<'_, T>> {
        Ok(RegionSlot {
            ptr: self.allocate(Layout::new::<T>())?.cast(),
            _region: PhantomData,
        })
    }
}
impl Drop for NativeRegion {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(self.ptr.as_ptr().cast(), 0, MEM_RELEASE);
        }
    }
}

pub struct RegionBuffer<'r> {
    ptr: NonNull<u8>,
    capacity: usize,
    len: usize,
    _region: PhantomData<&'r NativeRegion>,
}
impl RegionBuffer<'_> {
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.ptr.as_ptr().add(self.len).cast(),
                self.capacity - self.len,
            )
        }
    }
    /// # Safety
    /// The first `len` bytes must be initialized; len must not exceed capacity.
    #[inline(always)]
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity);
        self.len = len;
    }
    pub fn write_from_slice(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() > self.capacity {
            return Err(exhausted());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.ptr.as_ptr(), bytes.len());
            self.set_len(bytes.len());
        }
        Ok(())
    }
}
impl Deref for RegionBuffer<'_> {
    type Target = [u8];
    #[inline(always)]
    fn deref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}
impl DerefMut for RegionBuffer<'_> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

pub struct RegionSlot<'r, T> {
    ptr: NonNull<T>,
    _region: PhantomData<&'r NativeRegion>,
}
impl<'r, T> RegionSlot<'r, T> {
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
    pub fn write(self, value: T) -> RegionValue<'r, T> {
        unsafe {
            self.ptr.as_ptr().write(value);
            self.assume_init()
        }
    }
    /// # Safety
    /// The pointed-to T must be fully initialized, valid, and owned by this slot.
    /// Any references inside T must remain valid for the returned value's lifetime.
    pub unsafe fn assume_init(self) -> RegionValue<'r, T> {
        RegionValue {
            ptr: self.ptr,
            _region: PhantomData,
        }
    }
}
pub struct RegionValue<'r, T> {
    ptr: NonNull<T>,
    _region: PhantomData<&'r NativeRegion>,
}
impl<T> Deref for RegionValue<'_, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}
impl<T> DerefMut for RegionValue<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}
impl<T> Drop for RegionValue<'_, T> {
    fn drop(&mut self) {
        unsafe {
            self.ptr.as_ptr().drop_in_place();
        }
    }
}

// Stable, pointer-free IPC header. Zero is the unpublished state of fresh OS pages.
#[repr(C, align(64))]
struct Header {
    ready: AtomicU32,
    version: AtomicU32,
    magic: AtomicU64,
    mapped_len: AtomicU64,
    data_start: AtomicU64,
    next: AtomicU64,
}
#[repr(C, align(64))]
struct Block {
    state: AtomicU32,
    len: AtomicU64,
    capacity: AtomicU64,
    data_offset: AtomicU64,
}
const MAGIC: u64 = 0x315043494c5351; // QSLIPC1, little endian
const VERSION: u32 = 1;
struct Mapping {
    ptr: NonNull<u8>,
    len: usize,
    handle: HANDLE,
    page_size: usize,
}
// The bump cursor and publication state synchronize writes. Published payloads never mutate.
unsafe impl Send for Mapping {}
unsafe impl Sync for Mapping {}
impl Mapping {
    fn header(&self) -> &Header {
        unsafe { &*self.ptr.as_ptr().cast::<Header>() }
    }
    unsafe fn map(handle: HANDLE) -> io::Result<Self> {
        let view = MapViewOfFile(handle, FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, 0);
        let Some(ptr) = NonNull::new(view.Value.cast::<u8>()) else {
            let error = io::Error::last_os_error();
            CloseHandle(handle);
            return Err(error);
        };
        let mut info = MaybeUninit::<MEMORY_BASIC_INFORMATION>::uninit();
        if VirtualQuery(
            ptr.as_ptr().cast(),
            info.as_mut_ptr(),
            std::mem::size_of_val(&info),
        ) == 0
        {
            let error = io::Error::last_os_error();
            UnmapViewOfFile(view);
            CloseHandle(handle);
            return Err(error);
        }
        let len = info.assume_init().RegionSize;
        if len < std::mem::size_of::<Header>() || len > isize::MAX as usize {
            UnmapViewOfFile(view);
            CloseHandle(handle);
            return Err(invalid("invalid shared mapping size"));
        }
        Ok(Self {
            ptr,
            len,
            handle,
            page_size: page_size(),
        })
    }
}
impl Drop for Mapping {
    fn drop(&mut self) {
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.ptr.as_ptr().cast(),
            });
            CloseHandle(self.handle);
        }
    }
}
fn name_wide(name: &str) -> io::Result<Vec<u16>> {
    if name.is_empty() || name.contains('\0') {
        return Err(invalid("invalid mapping name"));
    }
    Ok(name.encode_utf16().chain(Some(0)).collect())
}

/// Named, append-only shared memory for cooperating threads/processes.
/// Published blocks are immutable; aborted reservations consume capacity until all views close.
/// Use `Local\name` for processes in the same Windows session. No reset/reclaim races are allowed.
#[derive(Clone)]
pub struct SharedRegion {
    mapping: Arc<Mapping>,
}
impl SharedRegion {
    pub fn create(name: &str, capacity: usize) -> io::Result<Self> {
        let name = name_wide(name)?;
        let start = page_size();
        let total = capacity
            .checked_add(start)
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or_else(|| invalid("invalid shared capacity"))?;
        if capacity == 0 {
            return Err(invalid("invalid shared capacity"));
        }
        unsafe {
            let handle = CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                std::ptr::null(),
                PAGE_READWRITE,
                (total as u64 >> 32) as u32,
                total as u32,
                name.as_ptr(),
            );
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "shared region already exists",
                ));
            }
            let mut mapping = Mapping::map(handle)?;
            let h = mapping.ptr.as_ptr().cast::<Header>();
            // Do not overwrite ready nonatomically: an opener may already be polling it.
            (*h).version.store(VERSION, Ordering::Relaxed);
            (*h).magic.store(MAGIC, Ordering::Relaxed);
            (*h).mapped_len.store(total as u64, Ordering::Relaxed);
            (*h).data_start.store(start as u64, Ordering::Relaxed);
            (*h).next.store(start as u64, Ordering::Relaxed);
            (*h).ready.store(1, Ordering::Release);
            mapping.len = total;
            Ok(Self {
                mapping: Arc::new(mapping),
            })
        }
    }
    /// Open an existing IPC mapping; returns WouldBlock if its header is not yet published.
    /// # Safety
    /// All processes accessing this mapping must follow this module's allocation/publication
    /// protocol, use the same header version, and never mutate published bytes/header metadata.
    /// The name must not be controlled by a process that violates this contract.
    pub unsafe fn open(name: &str) -> io::Result<Self> {
        let name = name_wide(name)?;
        let handle = OpenFileMappingW(FILE_MAP_READ | FILE_MAP_WRITE, 0, name.as_ptr());
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut mapping = Mapping::map(handle)?;
        let h = mapping.header();
        if h.ready.load(Ordering::Acquire) != 1 {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "shared header not published",
            ));
        }
        if h.version.load(Ordering::Relaxed) != VERSION
            || h.magic.load(Ordering::Relaxed) != MAGIC
            || h.mapped_len.load(Ordering::Relaxed) > mapping.len as u64
            || h.data_start.load(Ordering::Relaxed) < std::mem::size_of::<Header>() as u64
            || h.data_start.load(Ordering::Relaxed) >= h.mapped_len.load(Ordering::Relaxed)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "incompatible shared region",
            ));
        }
        mapping.len = h.mapped_len.load(Ordering::Relaxed) as usize;
        Ok(Self {
            mapping: Arc::new(mapping),
        })
    }
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.mapping.len - self.mapping.header().data_start.load(Ordering::Relaxed) as usize
    }
    pub fn used(&self) -> usize {
        self.mapping.header().next.load(Ordering::Relaxed) as usize
            - self.mapping.header().data_start.load(Ordering::Relaxed) as usize
    }
    /// Reserve an exclusive unpublished block. Alignment is a power of two <= the OS page size.
    pub fn reserve(&self, capacity: usize, alignment: usize) -> io::Result<SharedWrite> {
        if !alignment.is_power_of_two() || alignment > self.mapping.page_size {
            return Err(invalid("invalid shared alignment"));
        }
        let h = self.mapping.header();
        let mut next = h.next.load(Ordering::Relaxed);
        loop {
            let block = align_up(next as usize, 64).ok_or_else(exhausted)?;
            let data = block
                .checked_add(std::mem::size_of::<Block>())
                .and_then(|v| align_up(v, alignment))
                .ok_or_else(exhausted)?;
            let end = data.checked_add(capacity.max(1)).ok_or_else(exhausted)?;
            if end > self.mapping.len {
                return Err(exhausted());
            }
            match h.next.compare_exchange_weak(
                next,
                end as u64,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => unsafe {
                    let p = self.mapping.ptr.as_ptr().add(block).cast::<Block>();
                    (*p).state.store(0, Ordering::Relaxed);
                    (*p).capacity.store(capacity as u64, Ordering::Relaxed);
                    (*p).data_offset.store(data as u64, Ordering::Relaxed);
                    return Ok(SharedWrite {
                        mapping: Arc::clone(&self.mapping),
                        block,
                        data,
                        capacity,
                        len: 0,
                        published: false,
                    });
                },
                Err(actual) => next = actual,
            }
        }
    }
    /// Acquire an immutable block, or None if its writer has not published it.
    /// # Safety
    /// `offset` must identify a block returned by reserve/publish in THIS mapping,
    /// communicated without tearing. Arbitrary or cross-region offsets are forbidden.
    pub unsafe fn read_published(&self, offset: u64) -> io::Result<Option<SharedBytes>> {
        let offset = usize::try_from(offset).map_err(|_| invalid("invalid shared offset"))?;
        if offset < self.mapping.header().data_start.load(Ordering::Relaxed) as usize
            || offset % 64 != 0
            || offset
                .checked_add(std::mem::size_of::<Block>())
                .is_none_or(|n| n > self.mapping.len)
        {
            return Err(invalid("invalid shared offset"));
        }
        let b = &*self.mapping.ptr.as_ptr().add(offset).cast::<Block>();
        match b.state.load(Ordering::Acquire) {
            0 => Ok(None),
            1 => {
                let len = b.len.load(Ordering::Relaxed);
                let capacity = b.capacity.load(Ordering::Relaxed);
                let data_offset = b.data_offset.load(Ordering::Relaxed);
                if len > capacity
                    || data_offset < (offset + std::mem::size_of::<Block>()) as u64
                    || data_offset
                        .checked_add(capacity)
                        .is_none_or(|n| n > self.mapping.len as u64)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid shared block",
                    ));
                }
                Ok(Some(SharedBytes {
                    mapping: Arc::clone(&self.mapping),
                    offset,
                    data: data_offset as usize,
                    len: len as usize,
                }))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "shared reservation aborted",
            )),
        }
    }
}

/// Exclusive unpublished reservation. Dropping without publish marks the block aborted.
pub struct SharedWrite {
    mapping: Arc<Mapping>,
    block: usize,
    data: usize,
    capacity: usize,
    len: usize,
    published: bool,
}
impl SharedWrite {
    #[inline(always)]
    pub fn offset(&self) -> u64 {
        self.block as u64
    }
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        unsafe { self.mapping.ptr.as_ptr().add(self.data) }
    }
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut_ptr().add(self.len).cast(),
                self.capacity - self.len,
            )
        }
    }
    /// # Safety
    /// First len bytes must be initialized, and len <= capacity.
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity);
        self.len = len;
    }
    pub fn write_from_slice(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() > self.capacity {
            return Err(exhausted());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.as_mut_ptr(), bytes.len());
            self.set_len(bytes.len());
        }
        Ok(())
    }
    pub fn publish(mut self) -> SharedBytes {
        unsafe {
            let p = self.mapping.ptr.as_ptr().add(self.block).cast::<Block>();
            (*p).len.store(self.len as u64, Ordering::Relaxed);
            (*p).state.store(1, Ordering::Release);
        }
        self.published = true;
        SharedBytes {
            mapping: Arc::clone(&self.mapping),
            offset: self.block,
            data: self.data,
            len: self.len,
        }
    }
}
impl Deref for SharedWrite {
    type Target = [u8];
    #[inline(always)]
    fn deref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.mapping.ptr.as_ptr().add(self.data), self.len) }
    }
}
impl DerefMut for SharedWrite {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.mapping.ptr.as_ptr().add(self.data), self.len)
        }
    }
}
impl Drop for SharedWrite {
    fn drop(&mut self) {
        if !self.published {
            unsafe {
                (*self.mapping.ptr.as_ptr().add(self.block).cast::<Block>())
                    .state
                    .store(2, Ordering::Release);
            }
        }
    }
}
/// Immutable published bytes. Keeps this process's mapping alive, even after the region is dropped.
#[derive(Clone)]
pub struct SharedBytes {
    mapping: Arc<Mapping>,
    offset: usize,
    data: usize,
    len: usize,
}
impl SharedBytes {
    #[inline(always)]
    pub fn offset(&self) -> u64 {
        self.offset as u64
    }
}
impl Deref for SharedBytes {
    type Target = [u8];
    #[inline(always)]
    fn deref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.mapping.ptr.as_ptr().add(self.data), self.len) }
    }
}
