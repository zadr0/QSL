use qsl_v040_comparison::*;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
static TRACK: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
struct Counting;
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACK.load(Ordering::Relaxed) {
            CALLS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if TRACK.load(Ordering::Relaxed) {
            CALLS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        unsafe { System.dealloc(p, layout) }
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, new: usize) -> *mut u8 {
        if TRACK.load(Ordering::Relaxed) {
            CALLS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(new as u64, Ordering::Relaxed);
        }
        unsafe { System.realloc(p, layout, new) }
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;
fn measured(f: impl FnOnce()) -> (f64, f64) {
    CALLS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    TRACK.store(true, Ordering::Relaxed);
    f();
    TRACK.store(false, Ordering::Relaxed);
    (
        CALLS.load(Ordering::Relaxed) as f64 / FIXTURES as f64,
        BYTES.load(Ordering::Relaxed) as f64 / FIXTURES as f64,
    )
}
fn probe<C: Codec>(rows: &mut Vec<serde_json::Value>, len: usize) {
    let fixtures: Vec<_> = (0..FIXTURES).map(|i| Fixture::new(i, len)).collect();
    let inputs: Vec<_> = fixtures.iter().map(C::prepare).collect();
    let mut scratch = vec![0; CAPACITY];
    let mut b = Buffers::new(&mut scratch);
    let frames: Vec<_> = inputs
        .iter()
        .map(|v| {
            let n = C::encode(v, &mut b).unwrap();
            b.out[..n].to_vec()
        })
        .collect();
    for (f, bytes) in fixtures.iter().zip(&frames) {
        C::read(bytes, |v| assert_eq!(v, f.view(C::KIND))).unwrap();
    }
    let (write_alloc, write_bytes) = measured(|| {
        for v in &inputs {
            let n = C::encode(black_box(v), &mut b).unwrap();
            black_box(&b.out[..n]);
        }
    });
    let (read_alloc, read_bytes) = measured(|| {
        for bytes in &frames {
            black_box(C::read(black_box(bytes), |v| v.digest()).unwrap());
        }
    });
    let sizes: Vec<_> = frames.iter().map(Vec::len).collect();
    rows.push(serde_json::json!({"workload":C::KIND.name(len),"codec":C::NAME,"read_mode":if C::BORROWED{"read_borrowed"}else{"read_owned"},"write_measured":C::WRITE,"wire_min":sizes.iter().min(),"wire_max":sizes.iter().max(),"wire_mean":sizes.iter().sum::<usize>()as f64/FIXTURES as f64,"write_allocations":write_alloc,"write_allocated_bytes":write_bytes,"read_allocations":read_alloc,"read_allocated_bytes":read_bytes,"fixture_count":FIXTURES}));
}
fn main() {
    let mut rows = vec![];
    visit_cases!(probe, &mut rows);
    #[cfg(target_arch = "x86_64")]
    let bmi2 = std::is_x86_feature_detected!("bmi2");
    #[cfg(not(target_arch = "x86_64"))]
    let bmi2 = false;
    let output = serde_json::json!({"qsl_version":"0.4.0","bmi2":bmi2,"profiles":rows,"qsl_inline_object_bytes":std::mem::size_of::<generated::TextInline>(),"qsl_heap_object_bytes":std::mem::size_of::<generated::TextHeap>(),"qsl_borrow_object_bytes":std::mem::size_of::<generated::TextBorrow<'_>>()});
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
