//! Caller-owned single-thread regions. No atomics, allocator or platform I/O.
use super::Slot;
use core::{
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
};
use qsl_core::{FramePool, FrameStorage, QslError, QslResult};
/// A local pool; Cell intentionally prevents sharing it between threads.
/// Move the whole pool only when no lease borrows it.
///
/// ```compile_fail
/// fn needs_sync<T:Sync>(){}
/// needs_sync::<qsl_rt::LocalRegions<16,2>>();
/// ```
pub struct LocalRegions<const CAP: usize, const N: usize> {
    occupied: Cell<u64>,
    slots: [Slot<CAP>; N],
}
impl<const CAP: usize, const N: usize> Default for LocalRegions<CAP, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const CAP: usize, const N: usize> LocalRegions<CAP, N> {
    pub const fn new() -> Self {
        assert!(N > 0 && N <= 64);
        Self {
            occupied: Cell::new(0),
            slots: [const { Slot(UnsafeCell::new(MaybeUninit::uninit())) }; N],
        }
    }
    pub const fn new_zeroed() -> Self {
        assert!(N > 0 && N <= 64);
        Self {
            occupied: Cell::new(0),
            slots: [const { Slot(UnsafeCell::new(MaybeUninit::new([0; CAP]))) }; N],
        }
    }
    pub fn try_acquire(&self) -> QslResult<LocalLease<'_, CAP, N>> {
        let mask = if N == 64 { u64::MAX } else { (1u64 << N) - 1 };
        let used = self.occupied.get();
        let free = !used & mask;
        if free == 0 {
            return Err(QslError::PoolExhausted);
        }
        let index = free.trailing_zeros() as usize;
        self.occupied.set(used | (1u64 << index));
        Ok(LocalLease {
            pool: self,
            index,
            len: 0,
        })
    }
}
impl<const CAP: usize, const N: usize> FramePool for LocalRegions<CAP, N> {
    type Lease<'a>
        = LocalLease<'a, CAP, N>
    where
        Self: 'a;
    fn try_acquire(&self) -> QslResult<Self::Lease<'_>> {
        self.try_acquire()
    }
}
// ZEROED initializes all capacities; byte writes and releases preserve initialization.
unsafe impl<const CAP: usize, const N: usize> qsl_core::PreparedFramePool for LocalRegions<CAP, N> {
    const ZEROED: Self = Self::new_zeroed();
}
/// Exclusive initialized-prefix lease. Its reference to a !Sync pool makes it !Send.
///
/// ```compile_fail
/// fn needs_send<T:Send>(){}
/// needs_send::<qsl_rt::LocalLease<'static,16,2>>();
/// ```
pub struct LocalLease<'a, const CAP: usize, const N: usize> {
    pool: &'a LocalRegions<CAP, N>,
    index: usize,
    len: usize,
}
impl<const CAP: usize, const N: usize> LocalLease<'_, CAP, N> {
    pub fn capacity(&self) -> usize {
        CAP
    }
    fn ptr(&self) -> *mut u8 {
        self.pool.slots[self.index].0.get().cast()
    }
    pub fn as_slice(&self) -> &[u8] {
        // Exclusivity comes from the occupied bit; len only publishes initialized bytes.
        unsafe { core::slice::from_raw_parts(self.ptr(), self.len) }
    }
    /// # Safety
    /// Access is exclusive and cannot overlap live borrows of the lease.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr()
    }
    /// # Safety
    /// len <= CAP and all bytes of the prefix have been initialized.
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= CAP);
        self.len = len;
    }
}
// One occupied bit grants one live exclusive slot; no other access exposes its bytes.
unsafe impl<const CAP: usize, const N: usize> FrameStorage for LocalLease<'_, CAP, N> {
    fn capacity(&self) -> usize {
        CAP
    }
    fn as_bytes(&self) -> &[u8] {
        self.as_slice()
    }
    unsafe fn writable_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }
    unsafe fn set_initialized_len(&mut self, len: usize) {
        self.set_len(len)
    }
}
impl<const CAP: usize, const N: usize> core::ops::Deref for LocalLease<'_, CAP, N> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl<const CAP: usize, const N: usize> Drop for LocalLease<'_, CAP, N> {
    fn drop(&mut self) {
        self.pool
            .occupied
            .set(self.pool.occupied.get() & !(1u64 << self.index));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leases_are_disjoint_and_drop_restores_the_exact_slot() {
        let p = LocalRegions::<8, 2>::new();
        let mut a = p.try_acquire().unwrap();
        let mut b = p.try_acquire().unwrap();
        assert!(matches!(p.try_acquire(), Err(QslError::PoolExhausted)));
        unsafe {
            assert_ne!(a.as_mut_ptr(), b.as_mut_ptr());
            a.as_mut_ptr().write(7);
            a.set_len(1);
            b.as_mut_ptr().write(9);
            b.set_len(1);
        }
        assert_eq!(&*a, [7]);
        assert_eq!(&*b, [9]);
        drop(a);
        let c = p.try_acquire().unwrap();
        assert!(c.is_empty());
        assert_eq!(&*b, [9]);
    }
    #[test]
    fn prepared_provider_preserves_initialized_capacity_across_reuse() {
        let p = <LocalRegions<32, 1> as qsl_core::PreparedFramePool>::ZEROED;
        {
            let mut slot = p.try_acquire().unwrap();
            unsafe {
                assert_eq!(core::slice::from_raw_parts(slot.as_mut_ptr(), 32), [0; 32]);
                slot.as_mut_ptr().write_bytes(3, 8);
                slot.set_len(8);
            }
        }
        let mut slot = p.try_acquire().unwrap();
        assert!(slot.is_empty());
        unsafe {
            let bytes = core::slice::from_raw_parts(slot.as_mut_ptr(), 32);
            assert_eq!(&bytes[..8], [3; 8]);
            assert_eq!(&bytes[8..], [0; 24]);
        }
    }
    #[test]
    fn full_word_masks_and_empty_buffers_do_not_overflow_or_alias_data() {
        let p = LocalRegions::<0, 64>::new();
        let mut leases = qsl_core::InlineArray::<_, 64>::new();
        for _ in 0..64 {
            leases.push(p.try_acquire().unwrap()).unwrap();
        }
        assert!(matches!(p.try_acquire(), Err(QslError::PoolExhausted)));
        drop(leases);
        assert!(p.try_acquire().unwrap().is_empty());
    }
}
