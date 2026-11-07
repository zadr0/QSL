//! Per-schema static output regions. A lease owns a slot until Drop; concurrent
//! and nested callers cannot overwrite a live message. Exhaustion is explicit.
use crate::{QslError, QslResult};
use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU64, Ordering},
};

#[repr(C, align(64))]
struct Slot<const CAP: usize>(UnsafeCell<MaybeUninit<[u8; CAP]>>);
pub struct StaticRegions<const CAP: usize, const N: usize> {
    occupied: AtomicU64,
    epoch: crate::modernization::WaitWord,
    slots: [Slot<CAP>; N],
}
// Each initialized/readable slot has exactly one owning lease. Acquire/Release
// publication separates two successive writers to the same backing bytes.
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
            epoch: crate::modernization::WaitWord::new(N as u32),
            slots: [const { Slot(UnsafeCell::new(MaybeUninit::uninit())) }; N],
        }
    }
    pub fn try_acquire(&'static self) -> QslResult<RegionLease<CAP, N>> {
        if !self.epoch.try_take() {
            return Err(QslError::PoolExhausted);
        }
        let allowed = if N == 64 { u64::MAX } else { (1u64 << N) - 1 };
        let mut used = self.occupied.load(Ordering::Relaxed);
        loop {
            let free = (!used) & allowed;
            if free == 0 {
                used = self.occupied.load(Ordering::Acquire);
                continue;
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
    /// Wait for a free slot. Hold no other lease from this pool when all slots can be occupied.
    pub fn acquire(&'static self) -> RegionLease<CAP, N> {
        loop {
            match self.try_acquire() {
                Ok(lease) => return lease,
                Err(QslError::PoolExhausted) => self.epoch.wait(0),
                Err(_) => unreachable!(),
            }
        }
    }
    pub const fn new_zeroed() -> Self {
        assert!(N > 0 && N <= 64);
        Self {
            occupied: AtomicU64::new(0),
            epoch: crate::modernization::WaitWord::new(N as u32),
            slots: [const { Slot(UnsafeCell::new(MaybeUninit::new([0u8; CAP]))) }; N],
        }
    }
}
pub struct RegionLease<const CAP: usize, const N: usize> {
    pool: &'static StaticRegions<CAP, N>,
    index: usize,
    len: usize,
}
impl<const CAP: usize, const N: usize> RegionLease<CAP, N> {
    pub fn capacity(&self) -> usize {
        CAP
    }
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr(), self.len) }
    }
    fn ptr(&self) -> *mut u8 {
        self.pool.slots[self.index].0.get().cast::<u8>()
    }
    /// # Safety
    /// Only initialized bytes within CAP may be made readable; mutable access
    /// must not overlap any borrow obtained through this lease.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr()
    }
    /// # Safety
    /// The first len bytes must be initialized; len must not exceed CAP.
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= CAP);
        self.len = len;
    }
}
impl<const CAP: usize, const N: usize> std::ops::Deref for RegionLease<CAP, N> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl<const CAP: usize, const N: usize> Drop for RegionLease<CAP, N> {
    fn drop(&mut self) {
        self.pool
            .occupied
            .fetch_and(!(1u64 << self.index), Ordering::Release);
        self.pool.epoch.wake_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static POOL: StaticRegions<64, 2> = StaticRegions::new();
    #[test]
    fn leases_are_exclusive_and_exhaustion_is_recoverable() {
        let a = POOL.try_acquire().unwrap();
        let b = POOL.try_acquire().unwrap();
        assert!(matches!(POOL.try_acquire(), Err(QslError::PoolExhausted)));
        drop(a);
        let _c = POOL.try_acquire().unwrap();
        drop(b);
    }
    #[test]
    fn concurrent_leases_keep_bytes_until_drop() {
        static P: StaticRegions<8, 16> = StaticRegions::new();
        std::thread::scope(|s| {
            for x in 0u8..8 {
                s.spawn(move || {
                    for _ in 0..200 {
                        let mut out = P.try_acquire().unwrap();
                        unsafe {
                            out.as_mut_ptr().write_bytes(x, 8);
                            out.set_len(8);
                        }
                        std::thread::yield_now();
                        assert_eq!(out.as_slice(), &[x; 8]);
                    }
                });
            }
        });
    }
    #[test]
    fn blocking_waiters_resume_without_lost_notifications() {
        use std::sync::{mpsc, Arc, Barrier};
        use std::time::Duration;
        static SINGLE: StaticRegions<8, 1> = StaticRegions::new();
        let held = SINGLE.try_acquire().unwrap();
        let barrier = Arc::new(Barrier::new(5));
        let (send, receive) = mpsc::channel();
        let threads = (0..4)
            .map(|_| {
                let ready = barrier.clone();
                let completed = send.clone();
                std::thread::spawn(move || {
                    ready.wait();
                    let mut lease = SINGLE.acquire();
                    unsafe {
                        lease.as_mut_ptr().write(7);
                        lease.set_len(1);
                    }
                    assert_eq!(lease.as_slice(), &[7]);
                    completed.send(()).unwrap();
                })
            })
            .collect::<Vec<_>>();
        drop(send);
        barrier.wait();
        assert!(matches!(
            receive.recv_timeout(Duration::from_millis(20)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        drop(held);
        for _ in 0..4 {
            receive.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        for thread in threads {
            thread.join().unwrap();
        }
        assert!(SINGLE.try_acquire().is_ok());
    }
}
