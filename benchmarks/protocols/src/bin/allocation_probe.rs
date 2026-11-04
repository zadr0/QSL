//! Diagnostic executable only. This allocator is not linked into Criterion.
use qsl_protocol_comparison::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
static ENABLED: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOCS: AtomicU64 = AtomicU64::new(0);
struct Counting;
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = System.alloc(l);
        if ENABLED.load(Relaxed) {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = System.alloc_zeroed(l);
        if ENABLED.load(Relaxed) {
            CALLS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
        }
        p
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let p = System.realloc(p, l, n);
        if ENABLED.load(Relaxed) {
            REALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(n as u64, Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
}
#[global_allocator]
static ALLOC: Counting = Counting;
fn main() -> Result<()> {
    metrics::gate();
    metrics::pin(2)?;
    let fixtures = (0..FIXTURES).map(Fixture::new).collect::<Vec<_>>();
    let mut rows = Vec::new();
    for kind in [Kind::Numeric, Kind::Mixed] {
        for codec in Codec::ALL {
            let mut scratch = vec![0; 8192];
            let mut encoder = Encoder::new(&mut scratch);
            let frames = fixtures
                .iter()
                .map(|f| Frame::new(encoder.encode(codec, kind, f).unwrap()))
                .collect::<Vec<_>>();
            let mut decoder = Decoder::default();
            for frame in &frames {
                if codec == Codec::Protobuf {
                    decoder.protobuf(kind, frame.as_slice())?;
                }
            }
            for write in [true, false] {
                for batch in if kind == Kind::Numeric {
                    vec![1]
                } else {
                    BATCHES.to_vec()
                } {
                    for i in 0..64 {
                        if write {
                            std::hint::black_box(encoder.encode(codec, kind, &fixtures[i])?);
                        } else {
                            std::hint::black_box(read(codec, kind, frames[i].as_slice())?);
                        }
                    }
                    CALLS.store(0, Relaxed);
                    BYTES.store(0, Relaxed);
                    REALLOCS.store(0, Relaxed);
                    ENABLED.store(true, Relaxed);
                    for i in 0..1024 / batch {
                        for j in 0..batch {
                            let n = (i * batch + j) & (FIXTURES - 1);
                            if write {
                                std::hint::black_box(encoder.encode(codec, kind, &fixtures[n])?);
                            } else {
                                std::hint::black_box(if codec == Codec::Protobuf {
                                    decoder.protobuf(kind, frames[n].as_slice())
                                } else if codec == Codec::QslOptimized && kind == Kind::Numeric {
                                    read_frame::<1, 0>(&frames[n])
                                } else {
                                    read(codec, kind, frames[n].as_slice())
                                }?);
                            }
                        }
                    }
                    ENABLED.store(false, Relaxed);
                    rows.push(serde_json::json!({"kind":kind.name(),"codec":codec.name(),"operation":if write{"write"}else{"read"},"batch":batch,"alloc_calls_per_record":CALLS.load(Relaxed)as f64/1024.,"realloc_calls_per_record":REALLOCS.load(Relaxed)as f64/1024.,"requested_bytes_per_record":BYTES.load(Relaxed)as f64/1024.}));
                }
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&rows)?);
    Ok(())
}
