//! Build-time representation contracts and exact bounded register access.
use crate::{QslError, QslResult, QslSchema, QslType};
use std::{mem::MaybeUninit, ptr};
#[derive(Debug, Clone, Default)]
pub struct SchemaOptions {
    pub aggressive: bool,
    pub transmutate: bool,
    pub(crate) native_overlay: bool,
    pub loyal_cache: bool,
    pub slots: Option<usize>,
    pub cache_limit: Option<usize>,
    pub regions: Vec<NativeRegion>,
}
/// Zero-extend exactly N initialized bytes; N must not exceed 16.
/// # Safety
/// `source` points to N initialized readable bytes.
#[inline]
pub unsafe fn load_exact<const N: usize>(source: *const u8) -> u128 {
    assert!(N <= 16);
    let mut word = [0u8; 16];
    ptr::copy_nonoverlapping(source, word.as_mut_ptr(), N);
    u128::from_le_bytes(word)
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
#[derive(Clone, Debug)]
pub struct RegisterGroup {
    pub first: usize,
    pub fields: usize,
    pub offset: Option<usize>,
    pub bytes: usize,
    pub register_bits: usize,
    pub metadata_bytes: usize,
}
pub fn register_groups(s: &QslSchema) -> QslResult<Vec<RegisterGroup>> {
    let mut out = Vec::new();
    if s.compact() || s.qps() {
        return Ok(out);
    }
    let mut i = 0;
    let mut offset = if s.optional_count() == 0 {
        Some(0usize)
    } else {
        None
    };
    while i < s.fields.len() {
        let start = i;
        let mut bytes = 0;
        while let Some(f) = s.fields.get(i) {
            if f.optional
                || !matches!(
                    f.data,
                    QslType::Byte
                        | QslType::Integer { .. }
                        | QslType::RangedInteger { .. }
                        | QslType::Flags(_)
                )
            {
                break;
            }
            bytes += f.data.fixed_wire_size()?.ok_or(QslError::InvalidSchema)?;
            i += 1;
        }
        if i != start {
            let count = i - start;
            let metadata = match s.fields.get(i).filter(|f| !f.optional).map(|f| &f.data) {
                Some(
                    QslType::String(_)
                    | QslType::RangedString { .. }
                    | QslType::GuardString { .. }
                    | QslType::CopyGuardString { .. }
                    | QslType::BinaryString { .. },
                ) => 4,
                Some(QslType::InlineRawString { max, .. }) => {
                    if *max <= 127 {
                        1
                    } else if *max <= 32767 {
                        2
                    } else {
                        4
                    }
                }
                _ => 0,
            };
            let total = bytes + metadata;
            for at in (0..total).step_by(16) {
                let n = (total - at).min(16);
                out.push(RegisterGroup {
                    first: start,
                    fields: count,
                    offset: offset.and_then(|v| v.checked_add(at)),
                    bytes: n,
                    register_bits: n.next_power_of_two() * 8,
                    metadata_bytes: metadata,
                });
            }
            offset = offset.and_then(|v| v.checked_add(bytes));
            if metadata != 0 {
                i += 1;
                offset = None;
            }
            continue;
        }
        let f = &s.fields[i];
        offset = if f.optional {
            None
        } else {
            offset.and_then(|v| v.checked_add(f.data.fixed_wire_size().ok().flatten()?))
        };
        i += 1;
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arrays_use_native_data_bytes_for_the_strict_limit() {
        let schema = QslSchema {
            name: "A".into(),
            max_tag: 1,
            fields: vec![crate::QslField {
                name: "values".into(),
                tag: 1,
                data: QslType::FixedArray(Box::new(QslType::Flags(17)), 2),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(
            lower_storage(&schema, 6).unwrap().fields[0].storage,
            StorageKind::Default
        );
        assert_eq!(
            lower_storage(&schema, 8).unwrap().fields[0].storage,
            StorageKind::FixedArray(2)
        );
        let mut native = schema;
        native.options.transmutate = true;
        assert!(check_storage_limit(&native, 6).is_err());
    }
    #[test]
    fn optional_accessor_collisions_are_rejected() {
        for source in ["&A{u8? NATIVE_BYTES;}", "&A{u8? a;u8? set_a;}"] {
            assert!(crate::parse_qsl(source).is_err(), "{source}");
        }
    }
    #[test]
    fn exact_partial_register() {
        let b = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(unsafe { load_exact::<9>(b.as_ptr()) }, 0x090807060504030201);
    }
    #[test]
    fn drops_only_initialized() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        struct D(Arc<AtomicUsize>);
        impl Drop for D {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
        let n = Arc::new(AtomicUsize::new(0));
        {
            let mut a = InlineArray::<D, 256>::new();
            a.push(D(n.clone())).unwrap();
            a.push(D(n.clone())).unwrap();
        }
        assert_eq!(n.load(Ordering::Relaxed), 2);
    }
}

/// Availability is the wait predicate; release publishes a free slot before waking.
pub(crate) struct WaitWord {
    value: std::sync::atomic::AtomicU32,
    #[cfg(not(any(
        windows,
        all(
            target_os = "linux",
            any(
                target_arch = "x86_64",
                target_arch = "aarch64",
                target_arch = "riscv64",
                target_arch = "x86"
            )
        )
    )))]
    lock: std::sync::Mutex<()>,
    #[cfg(not(any(
        windows,
        all(
            target_os = "linux",
            any(
                target_arch = "x86_64",
                target_arch = "aarch64",
                target_arch = "riscv64",
                target_arch = "x86"
            )
        )
    )))]
    condition: std::sync::Condvar,
}
impl WaitWord {
    pub const fn new(value: u32) -> Self {
        Self {
            value: std::sync::atomic::AtomicU32::new(value),
            #[cfg(not(any(
                windows,
                all(
                    target_os = "linux",
                    any(
                        target_arch = "x86_64",
                        target_arch = "aarch64",
                        target_arch = "riscv64",
                        target_arch = "x86"
                    )
                )
            )))]
            lock: std::sync::Mutex::new(()),
            #[cfg(not(any(
                windows,
                all(
                    target_os = "linux",
                    any(
                        target_arch = "x86_64",
                        target_arch = "aarch64",
                        target_arch = "riscv64",
                        target_arch = "x86"
                    )
                )
            )))]
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
        #[cfg(windows)]
        unsafe {
            extern "system" {
                fn WaitOnAddress(
                    address: *const core::ffi::c_void,
                    compare: *const core::ffi::c_void,
                    size: usize,
                    milliseconds: u32,
                ) -> i32;
            }
            #[link(name = "kernel32")]
            extern "system" {}
            let _ = WaitOnAddress(
                self.value.as_ptr().cast(),
                (&expected as *const u32).cast(),
                4,
                u32::MAX,
            );
        }
        #[cfg(all(
            target_os = "linux",
            any(
                target_arch = "x86_64",
                target_arch = "aarch64",
                target_arch = "riscv64",
                target_arch = "x86"
            )
        ))]
        unsafe {
            extern "C" {
                fn syscall(number: core::ffi::c_long, ...) -> core::ffi::c_long;
            }
            #[cfg(target_arch = "x86_64")]
            const SYS: core::ffi::c_long = 202;
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            const SYS: core::ffi::c_long = 98;
            #[cfg(target_arch = "x86")]
            const SYS: core::ffi::c_long = 240;
            let _ = syscall(
                SYS,
                self.value.as_ptr(),
                128i32,
                expected,
                std::ptr::null::<u8>(),
                0usize,
                0usize,
            );
        }
        #[cfg(not(any(
            windows,
            all(
                target_os = "linux",
                any(
                    target_arch = "x86_64",
                    target_arch = "aarch64",
                    target_arch = "riscv64",
                    target_arch = "x86"
                )
            )
        )))]
        {
            let guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
            drop(
                self.condition
                    .wait_while(guard, |_| {
                        self.value.load(std::sync::atomic::Ordering::Acquire) == expected
                    })
                    .unwrap_or_else(|e| e.into_inner()),
            );
        }
    }
    pub fn wake_all(&self) {
        #[cfg(not(any(
            windows,
            all(
                target_os = "linux",
                any(
                    target_arch = "x86_64",
                    target_arch = "aarch64",
                    target_arch = "riscv64",
                    target_arch = "x86"
                )
            )
        )))]
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.value
            .fetch_add(1, std::sync::atomic::Ordering::Release);
        #[cfg(windows)]
        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn WakeByAddressAll(address: *const core::ffi::c_void);
            }
            WakeByAddressAll(self.value.as_ptr().cast());
        }
        #[cfg(all(
            target_os = "linux",
            any(
                target_arch = "x86_64",
                target_arch = "aarch64",
                target_arch = "riscv64",
                target_arch = "x86"
            )
        ))]
        unsafe {
            extern "C" {
                fn syscall(number: core::ffi::c_long, ...) -> core::ffi::c_long;
            }
            #[cfg(target_arch = "x86_64")]
            const SYS: core::ffi::c_long = 202;
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            const SYS: core::ffi::c_long = 98;
            #[cfg(target_arch = "x86")]
            const SYS: core::ffi::c_long = 240;
            let _ = syscall(
                SYS,
                self.value.as_ptr(),
                129i32,
                i32::MAX,
                0usize,
                0usize,
                0usize,
            );
        }
        #[cfg(not(any(
            windows,
            all(
                target_os = "linux",
                any(
                    target_arch = "x86_64",
                    target_arch = "aarch64",
                    target_arch = "riscv64",
                    target_arch = "x86"
                )
            )
        )))]
        self.condition.notify_all();
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StorageKind {
    #[default]
    Default,
    StackString(usize),
    StackArray(usize),
    FixedArray(usize),
    StrView,
    HeapString,
}
pub fn field_rust_type(f: &crate::QslField, borrowed: bool) -> QslResult<String> {
    Ok(match f.storage {
        StorageKind::Default => f.data.rust_type(borrowed)?,
        StorageKind::StackString(n) => format!("::qsl::StackString<{n}>"),
        StorageKind::StackArray(n) => {
            let e = match &f.data {
                QslType::RangedArray { element, .. } | QslType::Array(element, _) => element,
                _ => return Err(QslError::InvalidSchema),
            };
            format!("::qsl::InlineArray<{}, {n}>", e.rust_type(false)?)
        }
        StorageKind::FixedArray(n) => {
            let QslType::FixedArray(e, _) = &f.data else {
                return Err(QslError::InvalidSchema);
            };
            format!("[{};{n}]", e.rust_type(false)?)
        }
        StorageKind::StrView => "&'a str".into(),
        StorageKind::HeapString => "String".into(),
    })
}
pub fn lower_storage(s: &QslSchema, limit: usize) -> QslResult<QslSchema> {
    let mut s = s.clone();
    check_storage_limit(&s, limit).map_err(|_| QslError::InvalidSchema)?;
    let borrowed = s.borrow_strings();
    for (index, f) in s.fields.iter_mut().enumerate() {
        let native = s.options.transmutate
            || s.options
                .regions
                .iter()
                .any(|r| r.start <= index && index < r.end);
        f.storage = match &f.data {
            QslType::FixedString(_) | QslType::InlineRawString { .. } if borrowed && !native => {
                StorageKind::StrView
            }
            QslType::FixedString(n) | QslType::FixedCompressedString(n)
                if *n > limit && !native =>
            {
                StorageKind::HeapString
            }
            QslType::CompressedString { max, .. } | QslType::InlineRawString { max, .. }
                if *max > limit =>
            {
                StorageKind::HeapString
            }
            QslType::GuardString { .. } => StorageKind::StrView,
            QslType::FixedArray(e, n)
                if (native || !borrowed)
                    && (native
                        || fixed_native_layout(e)
                            .map(|(size, _)| size)
                            .and_then(|width| width.checked_mul(*n))
                            .is_some_and(|n| n <= limit)) =>
            {
                StorageKind::FixedArray(*n)
            }
            QslType::RangedString { max, .. }
                if s.options.aggressive && !borrowed && *max <= limit =>
            {
                StorageKind::StackString(*max)
            }
            QslType::RangedArray { element, max, .. }
                if s.options.aggressive
                    && !borrowed
                    && element.primitive()?.is_some()
                    && fixed_native_layout(element)
                        .map(|(size, _)| size)
                        .and_then(|n| n.checked_mul(*max))
                        .is_some_and(|n| n <= limit) =>
            {
                StorageKind::StackArray(*max)
            }
            _ => StorageKind::Default,
        };
        // Compatibility syntax maps onto ordinary ownership in new generated code.
        if let QslType::CopyGuardString { min, max } = f.data {
            f.data = match max {
                Some(max) => QslType::RangedString { min, max },
                None => QslType::String(min),
            };
            f.storage = StorageKind::HeapString;
            if s.options.aggressive {
                if let Some(max) = max.filter(|&n| n <= limit) {
                    f.storage = StorageKind::StackString(max)
                }
            }
        }
    }
    Ok(s)
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
pub fn validate_options(s: &QslSchema) -> Result<(), String> {
    let mut names = [
        "validate",
        "validate_values",
        "serialize",
        "serialize_checked",
        "serialize_checked_to",
        "deserialize_checked",
        "deserialize_checked_into",
        "try_new",
        "deserialize_into",
        "deserialize_overlay",
        "deserialize_overlay_unchecked",
        "deserialize_transmutate_copy",
        "deserialize_transmutate_copy_unchecked",
        "serialize_cached",
        "serialize_quantized_to",
        "serialize_qps_to",
        "deserialize_qps",
        "deserialize_qps_into",
        "predict_index",
        "MAX_DATA_SIZE",
        "NATIVE_BYTES",
        "NATIVE_ALIGNMENT",
        "NATIVE_WIRE_COMPATIBLE",
        "WIRE_MAX_BYTES",
        "CACHE_BYTES",
        "SCHEMA_HASHES",
        "SCHEMA_HASH_STRICT",
        "SCHEMA_HASH_LIGHT",
        "QUANT_ACTIVE",
        "QUANTUM",
        "QUANT_PADDING_BYTES",
        "QUANT_MAX_BYTES",
        "QUANT_RESTORE_BYTES",
        "QUANT_WHOLE_WORD_READ",
        "QUANT_WHOLE_WORD_WRITE",
        "QPS_SCALAR_PREFIX_BYTES",
        "QPS_RAW_PREFIX_BYTES",
        "QPS_PREFIX_LENGTH_HEADER",
        "QPS_LENGTHS_QUANTIZED",
        "OPT_TABLE_EVENT_COUNT",
        "OPT_TABLE_MAX_PAYLOAD",
        "OPT_TABLE_HEADER_BYTES",
        "OPT_TABLE_TAG_BYTES",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<std::collections::HashSet<_>>();
    for i in 0..s.options.regions.len() {
        for method in ["copy", "view"] {
            names.insert(format!("region_{i}_{method}"));
            names.insert(format!("deserialize_region_{i}_{method}"));
            names.insert(format!("region_{i}_{method}_unchecked"));
            names.insert(format!("deserialize_region_{i}_{method}_unchecked"));
            names.insert(format!("REGION_{i}_{}_COMPATIBLE", method.to_uppercase()));
            names.insert(format!(
                "REGION_{i}_WIRE_{}_COMPATIBLE",
                method.to_uppercase()
            ));
        }
    }
    for f in &s.fields {
        if optional_representation(f, s.options.aggressive).is_some()
            && (!names.insert(f.name.clone()) || !names.insert(format!("set_{}", f.name)))
        {
            return Err("optional accessor conflicts with a generated method name".into());
        }
    }
    for f in &s.fields {
        if f.name.starts_with("__qsl_") {
            return Err("field names beginning with __qsl_ are reserved".into());
        }
        if optional_representation(f, s.options.aggressive).is_some()
            && [
                "validate",
                "validate_values",
                "serialize",
                "serialize_checked",
                "serialize_checked_to",
                "deserialize_checked",
                "deserialize_checked_into",
                "try_new",
            ]
            .contains(&f.name.as_str())
        {
            return Err("optional accessor conflicts with a generated method name".into());
        }
    }

    if s.options.slots.is_some_and(|n| n == 0 || n > 64) {
        return Err("slots must be 1..64".into());
    }
    if s.options.transmutate
        && (s.compact()
            || s.qps()
            || s.optional_count() != 0
            || s.fields.iter().any(|f| {
                f.data.fixed_wire_size().ok().flatten().is_none()
                    || matches!(
                        f.data,
                        QslType::Custom { .. } | QslType::FixedCompressedString(_)
                    )
            }))
    {
        return Err("transmutate requires fixed compatible fields; compact, QPS, dynamic and optional fields are not native overlays".into());
    }
    if s.options.transmutate && s.borrow_strings() {
        check_native_offsets(s)?;
    }
    if s.qps() {
        let mut dynamic = false;
        for f in &s.fields {
            let numeric = matches!(
                f.data,
                QslType::Byte
                    | QslType::Integer { .. }
                    | QslType::RangedInteger { .. }
                    | QslType::Flags(_)
            );
            if numeric && dynamic {
                return Err("QPS scalar numeric fields must precede strings and arrays; their lengths and contents are never quantized".into());
            }
            dynamic |= !numeric;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct NativeRegion {
    pub rust_type: String,
    pub start: usize,
    pub end: usize,
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
    /// Source covers size_of::<Self>() allocated bytes; every non-padding field is initialized.
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
fn fixed_native_layout(t: &QslType) -> Option<(usize, usize)> {
    let native = match t {
        QslType::None | QslType::Padding(_) => (0, 1),
        QslType::FixedString(n) => (*n, 1),
        QslType::FixedArray(e, n) => {
            let (s, a) = fixed_native_layout(e)?;
            (s.checked_mul(*n)?, a)
        }
        _ => {
            let p = t.primitive().ok()??;
            let n = p[1..].parse::<usize>().ok()? / 8;
            let a = match n {
                1 => std::mem::align_of::<u8>(),
                2 => std::mem::align_of::<u16>(),
                4 => std::mem::align_of::<u32>(),
                8 => std::mem::align_of::<u64>(),
                _ => return None,
            };
            (n, a)
        }
    };
    Some(native)
}
pub fn check_native_offsets(s: &QslSchema) -> Result<(), String> {
    let (mut wire, mut native, mut alignment) = (0usize, 0usize, 1usize);
    for f in &s.fields {
        let (n, a) = fixed_native_layout(&f.data)
            .ok_or_else(|| format!("{}: native layout is not statically known", f.name))?;
        alignment = alignment.max(a);
        native = native
            .checked_add(a - 1)
            .map(|n| n & !(a - 1))
            .ok_or("native layout overflow")?;
        if !matches!(f.data, QslType::Padding(_) | QslType::None) && native != wire {
            return Err(format!("{}: transmutate wire offset {wire} differs from native offset {native}; add explicit padding or reorder fields",f.name));
        }
        native = native.checked_add(n).ok_or("native layout overflow")?;
        wire = wire
            .checked_add(
                f.data
                    .fixed_wire_size()
                    .ok()
                    .flatten()
                    .ok_or("variable native field")?,
            )
            .ok_or("wire layout overflow")?;
    }
    native = native
        .checked_add(alignment - 1)
        .map(|n| n & !(alignment - 1))
        .ok_or("native layout overflow")?;
    if native != wire {
        return Err(format!("transmutate wire size {wire} differs from native size {native}; add explicit trailing padding"));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct OptionalRepresentation {
    pub rust_type: String,
    pub niche: Option<String>,
    pub minimum: i128,
    pub maximum: i128,
    pub shifted: bool,
}
pub fn optional_representation(
    f: &crate::QslField,
    aggressive: bool,
) -> Option<OptionalRepresentation> {
    if !f.optional {
        return None;
    }
    let ty = f.data.primitive().ok()??.to_string();
    let (min, max) = match f.data {
        QslType::Byte => (0, 255),
        QslType::Flags(n) => (0, (1i128 << n) - 1),
        QslType::RangedInteger { min, max, .. } => (min, max),
        QslType::Integer { bits, signed, .. } => {
            if signed {
                (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
            } else {
                (0, (1i128 << bits) - 1)
            }
        }
        _ => return None,
    };
    let (niche, shifted) = if min > 0 || max < 0 {
        (
            Some(format!(
                "std::num::NonZero{}{}",
                if ty.starts_with('i') { "I" } else { "U" },
                &ty[1..]
            )),
            false,
        )
    } else if aggressive && max - min < (u64::MAX as i128) {
        let code = (max - min + 1) as u64;
        let bits = if code <= u8::MAX as u64 {
            8
        } else if code <= u16::MAX as u64 {
            16
        } else if code <= u32::MAX as u64 {
            32
        } else {
            64
        };
        if bits <= ty[1..].parse::<usize>().ok()? {
            (Some(format!("std::num::NonZeroU{bits}")), true)
        } else {
            (None, false)
        }
    } else {
        (None, false)
    };
    Some(OptionalRepresentation {
        rust_type: ty,
        niche,
        minimum: min,
        maximum: max,
        shifted,
    })
}

pub fn check_storage_limit(s: &QslSchema, limit: usize) -> Result<(), String> {
    if s.borrow_strings()
        && !s.options.transmutate
        && s.fields.iter().any(|f| {
            matches!(
                f.data,
                QslType::CompressedString { .. } | QslType::FixedCompressedString(_)
            )
        })
    {
        return Err(
            "LIFETIME cannot borrow compressed strings; use COPY with a bounded restoration buffer"
                .into(),
        );
    }
    for (index, f) in s.fields.iter().enumerate() {
        if !(s.options.transmutate
            || s.options
                .regions
                .iter()
                .any(|r| r.start <= index && index < r.end))
        {
            continue;
        }
        let bytes = match &f.data {
            QslType::FixedString(n) => Some(*n),
            QslType::FixedArray(e, n) => {
                fixed_native_layout(e).and_then(|(size, _)| size.checked_mul(*n))
            }
            _ => None,
        };
        if let Some(bytes) = bytes {
            if bytes > limit {
                return Err(format!("{}: fixed transmutate buffer {bytes} B exceeds STACK_LIMIT {limit} B; raise the limit explicitly",f.name));
            }
        }
    }
    Ok(())
}

#[doc(hidden)]
pub static ZERO_WORD: [u8; 16] = [0; 16];
