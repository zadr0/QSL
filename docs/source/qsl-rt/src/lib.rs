#![no_std]
#![doc = include_str!("../README.md")]
//! Static memory providers. Wire codecs and I/O are implemented by qsl-core.
#[cfg(test)]
extern crate std;
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU64, Ordering},
};
use qsl_core::{FramePool, FrameStorage, QslError, QslResult};
mod local;
pub use local::{LocalLease, LocalRegions};
#[repr(C, align(64))]
struct Slot<const CAP: usize>(UnsafeCell<MaybeUninit<[u8; CAP]>>);
pub struct StaticRegions<const CAP: usize, const N: usize> {
    occupied: AtomicU64,
    slots: [Slot<CAP>; N],
}
// Acquire gives exactly one lease per slot; Release publishes its reuse.
unsafe impl<const CAP: usize, const N: usize> Sync for StaticRegions<CAP, N> {}
impl<const CAP: usize, const N: usize> Default for StaticRegions<CAP, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const CAP: usize, const N: usize> StaticRegions<CAP, N> {
    pub const fn new() -> Self {
        assert!(N > 0 && N <= 64);
        Self {
            occupied: AtomicU64::new(0),
            slots: [const { Slot(UnsafeCell::new(MaybeUninit::uninit())) }; N],
        }
    }
    pub const fn new_zeroed() -> Self {
        assert!(N > 0 && N <= 64);
        Self {
            occupied: AtomicU64::new(0),
            slots: [const { Slot(UnsafeCell::new(MaybeUninit::new([0; CAP]))) }; N],
        }
    }
    pub fn try_acquire(&self) -> QslResult<RegionLease<'_, CAP, N>> {
        let mask = if N == 64 { u64::MAX } else { (1u64 << N) - 1 };
        let mut used = self.occupied.load(Ordering::Relaxed);
        loop {
            let free = (!used) & mask;
            if free == 0 {
                return Err(QslError::PoolExhausted);
            }
            let index = free.trailing_zeros() as usize;
            match self.occupied.compare_exchange_weak(
                used,
                used | (1u64 << index),
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Ok(RegionLease {
                        pool: self,
                        index,
                        len: 0,
                    })
                }
                Err(actual) => used = actual,
            }
        }
    }
}
impl<const CAP: usize, const N: usize> FramePool for StaticRegions<CAP, N> {
    type Lease<'a>
        = RegionLease<'a, CAP, N>
    where
        Self: 'a;
    fn try_acquire(&self) -> QslResult<Self::Lease<'_>> {
        StaticRegions::try_acquire(self)
    }
}
pub struct RegionLease<'a, const CAP: usize, const N: usize> {
    pool: &'a StaticRegions<CAP, N>,
    index: usize,
    len: usize,
}
impl<const CAP: usize, const N: usize> RegionLease<'_, CAP, N> {
    pub fn capacity(&self) -> usize {
        CAP
    }
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr(), self.len) }
    }
    fn ptr(&self) -> *mut u8 {
        self.pool.slots[self.index].0.get().cast()
    }
    /// # Safety
    /// Access is exclusive and cannot overlap any current borrow of the lease.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr()
    }
    /// # Safety
    /// len <= CAP and the first len bytes are initialized.
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= CAP);
        self.len = len;
    }
}
unsafe impl<const CAP: usize, const N: usize> FrameStorage for RegionLease<'_, CAP, N> {
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
impl<const CAP: usize, const N: usize> core::ops::Deref for RegionLease<'_, CAP, N> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl<const CAP: usize, const N: usize> Drop for RegionLease<'_, CAP, N> {
    fn drop(&mut self) {
        self.pool
            .occupied
            .fetch_and(!(1u64 << self.index), Ordering::Release);
    }
}
unsafe impl<const CAP: usize, const N: usize> qsl_core::buffer::PreparedFramePool
    for StaticRegions<CAP, N>
{
    const ZEROED: Self = Self::new_zeroed();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_pool_returns_and_release_restores_capacity() {
        let p = StaticRegions::<8, 2>::new();
        let a = p.try_acquire().unwrap();
        let b = p.try_acquire().unwrap();
        assert!(matches!(p.try_acquire(), Err(QslError::PoolExhausted)));
        drop(a);
        let _c = p.try_acquire().unwrap();
        drop(b);
    }
    #[test]
    fn generic_contract_and_local_lifetime() {
        fn fill<P: FramePool>(p: &P) {
            let mut b = p.try_acquire().unwrap();
            assert!(b.capacity() >= 3);
            unsafe {
                b.writable_ptr()
                    .copy_from_nonoverlapping(b"qsl".as_ptr(), 3);
                b.set_initialized_len(3);
            }
            assert_eq!(b.as_bytes(), b"qsl");
        }
        let p = StaticRegions::<8, 1>::new();
        fill(&p);
        fill(&p);
    }
    #[test]
    fn concurrent_slots_do_not_alias() {
        let p = StaticRegions::<8, 16>::new();
        std::thread::scope(|s| {
            for x in 0..8u8 {
                let p = &p;
                s.spawn(move || {
                    for _ in 0..200 {
                        let mut b = p.try_acquire().unwrap();
                        unsafe {
                            b.as_mut_ptr().write_bytes(x, 8);
                            b.set_len(8)
                        }
                        std::thread::yield_now();
                        assert_eq!(b.as_slice(), [x; 8]);
                    }
                });
            }
        });
    }
}
