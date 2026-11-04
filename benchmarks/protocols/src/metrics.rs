//! Boundary-only metrics; no counters are inserted into record loops.
use crate::Result;
#[cfg(windows)]
pub fn pin(cpu: usize) -> Result<()> {
    unsafe {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentThread() -> *mut core::ffi::c_void;
            fn SetThreadAffinityMask(thread: *mut core::ffi::c_void, mask: usize) -> usize;
            fn SetThreadPriority(thread: *mut core::ffi::c_void, priority: i32) -> i32;
        }
        if SetThreadAffinityMask(GetCurrentThread(), 1usize << cpu) == 0
            || SetThreadPriority(GetCurrentThread(), 1) == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    }
}
#[cfg(not(windows))]
pub fn pin(_: usize) -> Result<()> {
    Ok(())
}
pub fn gate() {
    if let Some(path) = std::env::var_os("QSL_BENCH_START_FILE") {
        let begin = std::time::Instant::now();
        while !std::path::Path::new(&path).exists() {
            assert!(begin.elapsed().as_secs() < 120, "job attachment timeout");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
