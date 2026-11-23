use qsl_realistic_bench::{
    fixture::{make, RECORDS, WORKLOADS},
    model::Envelope,
    Result, CODECS,
};
use serde_json::json;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

struct Counter;
static ENABLED: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
#[global_allocator]
static ALLOCATOR: Counter = Counter;
fn account(bytes: usize) {
    if ENABLED.load(Ordering::Relaxed) {
        CALLS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(bytes, Ordering::Relaxed);
    }
}
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        account(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        account(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, len: usize) -> *mut u8 {
        account(len);
        unsafe { System.realloc(ptr, layout, len) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
fn measured<T>(f: impl FnOnce() -> T) -> (T, [usize; 2]) {
    CALLS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    ENABLED.store(true, Ordering::Relaxed);
    let value = f();
    ENABLED.store(false, Ordering::Relaxed);
    (
        value,
        [CALLS.load(Ordering::Relaxed), BYTES.load(Ordering::Relaxed)],
    )
}
fn main() -> Result<()> {
    let mut profiles = Vec::new();
    for workload in WORKLOADS {
        for codec in CODECS {
            let mut samples = Vec::new();
            for index in 0..RECORDS {
                let source = make(workload, index);
                let expected = source.clone();
                let (frame, [encode_allocs, encode_bytes]) =
                    measured(|| codec.encode(black_box(source)));
                let frame = frame?;
                let (value, [decode_allocs, decode_bytes]) =
                    measured(|| codec.decode(black_box(&frame)));
                assert_eq!(value?, expected);
                samples.push(json!({
                    "wire_bytes":frame.len(), "encode_alloc_calls":encode_allocs, "encode_alloc_bytes":encode_bytes,
                    "decode_alloc_calls":decode_allocs, "decode_alloc_bytes":decode_bytes,
                }));
            }
            profiles.push(json!({"workload":workload, "codec":codec.name(), "samples":samples}));
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"owned_inline_bytes":std::mem::size_of::<Envelope>(), "profiles":profiles})
        )?
    );
    Ok(())
}
