use qsl_optimized_bench::{
    cases::{registry, selection, VALIDATION_OFFSET},
    Result, MAX_FRAME,
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
    let selected = selection()?.ok_or("probe requires the confirmed candidate manifest")?;
    let mut output = Box::new([0u8; MAX_FRAME]);
    let mut profiles = Vec::new();
    for case in registry()
        .into_iter()
        .filter(|case| selected.contains(&case.id()))
    {
        let fixture = case.warm(VALIDATION_OFFSET, &mut output)?;
        let mut samples = Vec::new();
        for record in &fixture {
            let (result, [allocs, bytes]) =
                measured(|| case.run(black_box(record), black_box(&mut output)));
            result?;
            samples.push(
                json!({"wire_bytes":record.wire_len(),"alloc_calls":allocs,"alloc_bytes":bytes}),
            );
        }
        profiles.push(json!({"id":case.id(),"samples":samples}));
    }
    assert_eq!(profiles.len(), selected.len());
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"profiles":profiles,"bmi2":std::is_x86_feature_detected!("bmi2")})
        )?
    );
    Ok(())
}
