use qsl_layout_bench::{generated::*, *};
use std::{
    alloc::{GlobalAlloc, Layout as AllocLayout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

struct Counter;
static ENABLED: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicUsize = AtomicUsize::new(0);
#[global_allocator]
static ALLOCATOR: Counter = Counter;

unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: AllocLayout) -> *mut u8 {
        if ENABLED.load(Ordering::Relaxed) {
            CALLS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: AllocLayout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

fn count_allocations(operation: impl FnOnce()) -> usize {
    CALLS.store(0, Ordering::Relaxed);
    ENABLED.store(true, Ordering::Relaxed);
    operation();
    ENABLED.store(false, Ordering::Relaxed);
    CALLS.load(Ordering::Relaxed)
}

fn profile<T: Layout>() -> serde_json::Value {
    validate::<T>();
    let field_example = Fields {
        flag: 0x11,
        count: 0x99_88_77_66_55_44_33_22,
        id: 0xbbaa,
        time: 0xffeeddcc,
        quality: 0x10,
    };
    let frame = example_frame::<T>(field_example);
    let record = T::from_fields(field_example);
    let mut output = [0; FRAME_SIZE];
    let write_allocs = count_allocations(|| {
        for _ in 0..HOT_COUNT {
            black_box(black_box(&record).serialize(black_box(&mut output)));
        }
    });
    let read_allocs = count_allocations(|| {
        for _ in 0..HOT_COUNT {
            black_box(T::deserialize(black_box(&frame)));
        }
    });
    serde_json::json!({
        "case": T::NAME, "size": std::mem::size_of::<T>(), "alignment": std::mem::align_of::<T>(),
        "offsets": T::OFFSETS, "frame_bytes": FRAME_SIZE, "frame_example_hex": frame.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "write_allocs_256": write_allocs, "read_allocs_256": read_allocs,
    })
}
fn main() {
    let records = [
        profile::<Declared>(),
        profile::<Auto>(),
        profile::<Canonical>(),
        profile::<Manual>(),
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "field_order": ["flag", "count", "id", "time", "quality"],
            "fixture_count": HOT_COUNT, "large_count": LARGE_COUNT, "profiles": records,
        }))
        .unwrap()
    );
}
