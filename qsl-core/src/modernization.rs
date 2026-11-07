//! Runtime storage and exact bounded register access.
pub use crate::schema_modernization::*;
use crate::{QslError, QslResult};
use std::{mem::MaybeUninit, ptr};
// Separate return types keep narrow loads narrow throughout generated kernels.
macro_rules! exact_register_loaders {
    ($(($name:ident, $word:ty, $bytes:literal)),* $(,)?) => {$(
        /// Zero-extend exactly N little-endian input bytes into this register.
        /// The width bound is checked at monomorphization, not on the hot path.
        /// # Safety
        /// source is non-null and addresses N initialized readable bytes.
        #[inline]
        pub unsafe fn $name<const N: usize>(source: *const u8) -> $word {
            const { assert!(N <= $bytes, "exact load exceeds register width"); }
            let mut word = [0u8; $bytes];
            ptr::copy_nonoverlapping(source, word.as_mut_ptr(), N);
            <$word>::from_le_bytes(word)
        }
    )*};
}
exact_register_loaders!(
    (load_u8, u8, 1),
    (load_u16, u16, 2),
    (load_u32, u32, 4),
    (load_u64, u64, 8),
    (load_u128, u128, 16),
);

/// Legacy u128-result API. Generated kernels use the typed load_u* entry points.
/// # Safety
/// source is non-null and addresses N initialized readable bytes; N <= 16.
#[inline]
pub unsafe fn load_exact<const N: usize>(source: *const u8) -> u128 {
    load_u128::<N>(source)
}

/// Only the first len elements are initialized and dropped.
#[repr(C)]
pub struct InlineArray<T, const N: usize> {
    len: u32,
    buffer: [MaybeUninit<T>; N],
}
impl<T, const N: usize> Default for InlineArray<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const N: usize> InlineArray<T, N> {
    pub const fn new() -> Self {
        assert!(N <= u32::MAX as usize);
        Self {
            len: 0,
            buffer: [const { MaybeUninit::uninit() }; N],
        }
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn capacity(&self) -> usize {
        N
    }
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.buffer.as_ptr().cast(), self.len()) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.buffer.as_mut_ptr().cast(), self.len as usize)
        }
    }
    pub fn push(&mut self, value: T) -> QslResult<()> {
        if self.len() == N {
            return Err(QslError::BufferTooSmall);
        }
        self.buffer[self.len()].write(value);
        self.len += 1;
        Ok(())
    }
    pub fn into_array(mut self) -> QslResult<[T; N]> {
        if self.len() != N {
            return Err(QslError::InvalidValue);
        }
        let array = unsafe { self.buffer.as_ptr().cast::<[T; N]>().read() };
        self.len = 0;
        Ok(array)
    }
    pub fn clear(&mut self) {
        while self.len != 0 {
            self.len -= 1;
            unsafe { self.buffer[self.len as usize].assume_init_drop() }
        }
    }
    /// # Safety
    /// Source contains len valid initialized native T representations, len <= N;
    /// output is aligned exclusive uninitialized Self, disjoint from the source.
    pub unsafe fn copy_native_into(source: *const u8, len: usize, out: *mut Self)
    where
        T: Copy,
    {
        ptr::copy_nonoverlapping(
            source,
            ptr::addr_of_mut!((*out).buffer).cast::<u8>(),
            len * std::mem::size_of::<T>(),
        );
        ptr::addr_of_mut!((*out).len).write(len as u32);
    }
    pub fn try_from_slice(values: &[T]) -> QslResult<Self>
    where
        T: Copy,
    {
        if values.len() > N {
            return Err(QslError::BufferTooSmall);
        }
        let mut out = Self::new();
        unsafe {
            ptr::copy_nonoverlapping(
                values.as_ptr(),
                out.buffer.as_mut_ptr().cast(),
                values.len(),
            )
        };
        out.len = values.len() as u32;
        Ok(out)
    }
    pub fn try_from_iter(values: impl IntoIterator<Item = T>) -> QslResult<Self> {
        let mut out = Self::new();
        for v in values {
            out.push(v)?;
        }
        Ok(out)
    }
}
impl<T: Clone, const N: usize> Clone for InlineArray<T, N> {
    fn clone(&self) -> Self {
        Self::try_from_iter(self.as_slice().iter().cloned()).expect("same capacity")
    }
}
impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for InlineArray<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_slice().fmt(f)
    }
}
impl<T, const N: usize> Drop for InlineArray<T, N> {
    fn drop(&mut self) {
        self.clear()
    }
}
impl<T, const N: usize> std::ops::Deref for InlineArray<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, const N: usize> AsRef<[T]> for InlineArray<T, N> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
#[repr(transparent)]
pub struct StackString<const N: usize>(InlineArray<u8, N>);
impl<const N: usize> StackString<N> {
    pub fn new() -> Self {
        Self(InlineArray::new())
    }
    pub fn try_from_str(text: &str) -> QslResult<Self> {
        Ok(Self(InlineArray::try_from_slice(text.as_bytes())?))
    }
    /// # Safety
    /// Source contains len valid UTF-8 bytes, len <= N; output is aligned,
    /// exclusive uninitialized Self, disjoint from the source.
    pub unsafe fn copy_into(source: *const u8, len: usize, out: *mut Self) {
        InlineArray::<u8, N>::copy_native_into(source, len, ptr::addr_of_mut!((*out).0));
    }
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl<const N: usize> Default for StackString<N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const N: usize> std::ops::Deref for StackString<N> {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
impl<const N: usize> std::fmt::Debug for StackString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

/// Availability is the wait predicate; release publishes a free slot before waking.
/// The atomic acquisition path stays independent of the portable blocking path.
pub(crate) struct WaitWord {
    value: std::sync::atomic::AtomicU32,
    lock: std::sync::Mutex<()>,
    condition: std::sync::Condvar,
}
impl WaitWord {
    pub const fn new(value: u32) -> Self {
        Self {
            value: std::sync::atomic::AtomicU32::new(value),
            lock: std::sync::Mutex::new(()),
            condition: std::sync::Condvar::new(),
        }
    }
    pub fn try_take(&self) -> bool {
        self.value
            .fetch_update(
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
                |n| n.checked_sub(1),
            )
            .is_ok()
    }
    pub fn wait(&self, expected: u32) {
        let guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        drop(
            self.condition
                .wait_while(guard, |_| {
                    self.value.load(std::sync::atomic::Ordering::Acquire) == expected
                })
                .unwrap_or_else(|e| e.into_inner()),
        );
    }
    pub fn wake_all(&self) {
        // The same lock pairs the predicate check with sleep, preventing a
        // release from notifying between that check and Condvar::wait.
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Release);
        self.condition.notify_all();
    }
}

#[inline]
unsafe fn exact_load<const N: usize>(source: *const u8) -> u128 {
    load_exact::<N>(source)
}
#[inline]
unsafe fn exact_store<const N: usize>(destination: *mut u8, value: u128) {
    ptr::copy_nonoverlapping(value.to_le_bytes().as_ptr(), destination, N)
}
static EXACT_LOAD: [unsafe fn(*const u8) -> u128; 17] = [
    exact_load::<0>,
    exact_load::<1>,
    exact_load::<2>,
    exact_load::<3>,
    exact_load::<4>,
    exact_load::<5>,
    exact_load::<6>,
    exact_load::<7>,
    exact_load::<8>,
    exact_load::<9>,
    exact_load::<10>,
    exact_load::<11>,
    exact_load::<12>,
    exact_load::<13>,
    exact_load::<14>,
    exact_load::<15>,
    exact_load::<16>,
];
static EXACT_STORE: [unsafe fn(*mut u8, u128); 17] = [
    exact_store::<0>,
    exact_store::<1>,
    exact_store::<2>,
    exact_store::<3>,
    exact_store::<4>,
    exact_store::<5>,
    exact_store::<6>,
    exact_store::<7>,
    exact_store::<8>,
    exact_store::<9>,
    exact_store::<10>,
    exact_store::<11>,
    exact_store::<12>,
    exact_store::<13>,
    exact_store::<14>,
    exact_store::<15>,
    exact_store::<16>,
];
/// Table dispatch selects an exact-width load without a data branch in the caller.
/// # Safety
/// `count <= 16`; `source` addresses count initialized readable bytes.
#[inline]
pub unsafe fn load_word(source: *const u8, count: usize) -> u128 {
    (*EXACT_LOAD.get_unchecked(count))(source)
}
/// # Safety
/// `count <= 16`; `destination` addresses count exclusively writable bytes.
#[inline]
pub unsafe fn store_word(destination: *mut u8, value: u128, count: usize) {
    (*EXACT_STORE.get_unchecked(count))(destination, value)
}

#[repr(C, align(64))]
pub struct CachedFrame<const N: usize> {
    bytes: MaybeUninit<[u8; N]>,
    len: usize,
}
impl<const N: usize> Default for CachedFrame<N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const N: usize> CachedFrame<N> {
    pub const fn new() -> Self {
        Self {
            bytes: MaybeUninit::uninit(),
            len: 0,
        }
    }
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.bytes.as_ptr().cast(), self.len) }
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes.as_mut_ptr().cast()
    }
    /// # Safety
    /// len <= N and those bytes have been initialized.
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= N);
        self.len = len;
    }
}
/// Initialized leaf metadata; padding is deliberately absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeLeaf {
    pub offset: usize,
    pub bytes: usize,
    pub kind: u16,
    pub elements: usize,
}
#[derive(Clone, Copy)]
pub struct NativeLayout {
    pub fields: [NativeLeaf; 64],
    pub count: usize,
}
impl NativeLayout {
    pub const EMPTY: Self = Self {
        fields: [NativeLeaf {
            offset: 0,
            bytes: 0,
            kind: 0,
            elements: 0,
        }; 64],
        count: 0,
    };
    pub const fn leaf(bytes: usize, kind: u16, elements: usize) -> Self {
        let mut out = Self::EMPTY;
        out.fields[0] = NativeLeaf {
            offset: 0,
            bytes,
            kind,
            elements,
        };
        out.count = 1;
        out
    }
    pub const fn fields(fields: &[(usize, Self)]) -> Self {
        let mut out = Self::EMPTY;
        let mut i = 0;
        while i < fields.len() {
            let mut j = 0;
            while j < fields[i].1.count {
                assert!(out.count < 64, "native layout exceeds 64 leaves");
                let mut field = fields[i].1.fields[j];
                field.offset += fields[i].0;
                out.fields[out.count] = field;
                out.count += 1;
                j += 1;
            }
            i += 1;
        }
        out
    }
    pub const fn compatible(&self, other: &Self) -> bool {
        if !self.same_fields(other) {
            return false;
        }
        let mut i = 0;
        while i < self.count {
            if self.fields[i].offset != other.fields[i].offset {
                return false;
            }
            i += 1;
        }
        true
    }
    pub const fn same_fields(&self, other: &Self) -> bool {
        if self.count != other.count {
            return false;
        }
        let mut i = 0;
        while i < self.count {
            let a = self.fields[i];
            let b = other.fields[i];
            let kinds = a.kind == b.kind
                || (a.kind == 200 && b.kind == 33)
                || (a.kind == 33 && b.kind == 200);
            if a.bytes != b.bytes || a.elements != b.elements || !kinds {
                return false;
            }
            i += 1;
        }
        true
    }
}
/// Fixed native representations. Accepted initialized fields form a valid Copy
/// value. Validators must inspect leaves only, since native padding may be uninitialized.
/// # Safety
/// Layout metadata describes every non-padding field and accepts only valid Self values.
pub unsafe trait NativeField: Copy {
    const NATIVE_LAYOUT: NativeLayout;
    fn validate_native(bytes: &[u8]) -> QslResult<()> {
        if bytes.len() != std::mem::size_of::<Self>() {
            return Err(QslError::InvalidValue);
        }
        unsafe { Self::validate_native_fields(bytes.as_ptr()) }
    }
    /// # Safety
    /// Source covers `size_of::<Self>()` allocated bytes; every non-padding field is initialized.
    unsafe fn validate_native_fields(address: *const u8) -> QslResult<()>;
}
macro_rules! native_integer{($($t:ty=>$kind:expr),*)=>{$(unsafe impl NativeField for $t{const NATIVE_LAYOUT:NativeLayout=NativeLayout::leaf(std::mem::size_of::<Self>(),$kind,1);unsafe fn validate_native_fields(_address:*const u8)->QslResult<()>{Ok(())}})*};}
native_integer!(u8=>1,i8=>2,u16=>3,i16=>4,u32=>5,i32=>6,u64=>7,i64=>8);
unsafe impl<T: NativeField, const N: usize> NativeField for [T; N] {
    const NATIVE_LAYOUT: NativeLayout = {
        let layout = T::NATIVE_LAYOUT;
        if layout.count == 1
            && layout.fields[0].offset == 0
            && layout.fields[0].bytes == std::mem::size_of::<T>()
            && layout.fields[0].kind <= 8
        {
            NativeLayout::leaf(std::mem::size_of::<Self>(), 32 + layout.fields[0].kind, N)
        } else {
            let mut out = NativeLayout::EMPTY;
            let mut i = 0;
            while i < N {
                let mut j = 0;
                while j < layout.count {
                    assert!(out.count < 64, "native layout exceeds 64 leaves");
                    let mut leaf = layout.fields[j];
                    leaf.offset += i * std::mem::size_of::<T>();
                    out.fields[out.count] = leaf;
                    out.count += 1;
                    j += 1;
                }
                i += 1;
            }
            out
        }
    };
    unsafe fn validate_native_fields(address: *const u8) -> QslResult<()> {
        for i in 0..N {
            T::validate_native_fields(address.add(i * std::mem::size_of::<T>()))?;
        }
        Ok(())
    }
}
#[doc(hidden)]
pub static ZERO_WORD: [u8; 16] = [0; 16];
