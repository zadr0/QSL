use qsl_feature_bench::{generated::*, *};
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

fn profile<T: TextCodec>(corpus: &str) -> serde_json::Value {
    let samples: Vec<_> = (0..RECORDS).map(|i| {
        let original = text(corpus, i);
        let value = T::prepare(i as u32, original.clone());
        let mut buffer = [0u8; BUFFER];
        let (used, [encode_allocs, encode_bytes]) = measured(|| black_box(&value).write(black_box(&mut buffer)));
        let (decoded, [decode_allocs, decode_bytes]) = measured(|| T::read(black_box(&buffer[..used])));
        assert_eq!(decoded.fields(), (i as u32, original.as_str()));
        json!({"wire_bytes":used, "encode_alloc_calls":encode_allocs, "encode_alloc_bytes":encode_bytes,
            "decode_alloc_calls":decode_allocs, "decode_alloc_bytes":decode_bytes})
    }).collect();
    json!({"workload": corpus, "codec": T::NAME, "object_bytes": std::mem::size_of::<T>(), "samples":samples})
}
fn main() {
    let mut profiles = Vec::new();
    for corpus in CORPORA {
        profiles.extend([
            profile::<HeapText>(corpus),
            profile::<StackText>(corpus),
            profile::<CompressedText>(corpus),
        ]);
    }
    let (aligned, unaligned) = native_frames();
    let (_, field_cost) = measured(|| fields(black_box(&aligned[0].0)));
    let (_, copy_cost) = measured(|| copied(black_box(&aligned[0].0)));
    let (_, copy_unaligned_cost) = measured(|| copied(black_box(&unaligned[0].0[1..])));
    let (_, overlay_cost) = measured(|| overlay(black_box(&aligned[0].0)));
    println!("{}", serde_json::to_string_pretty(&json!({"profiles":profiles, "native":{
        "wire_bytes":16, "alignment":std::mem::align_of::<Native>(), "native_object_bytes":std::mem::size_of::<Native>(),
        "field_allocs":field_cost, "copy_allocs":copy_cost, "copy_unaligned_allocs":copy_unaligned_cost, "overlay_allocs":overlay_cost
    }})).unwrap());
}
