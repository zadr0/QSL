use criterion::{Criterion, Throughput};
use qsl::{BorrowSerializable, Serializable};
use qsl_strings_bench::*;
use std::{hint::black_box, mem::MaybeUninit, time::Duration};
fn main() {
    if let Some(gate) = std::env::var_os("QSL_BENCH_START_FILE") {
        let gate = std::path::PathBuf::from(gate);
        let deadline = std::time::Instant::now() + Duration::from_secs(60);
        while !gate.exists() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    let mut c = Criterion::default()
        .warm_up_time(Duration::from_secs(15))
        .measurement_time(Duration::from_secs(10))
        .sample_size(200)
        .confidence_level(0.99)
        .configure_from_args();
    let texts: Vec<_> = (0..1024)
        .map(|i| {
            let mut s = "Я".repeat(1023);
            s.push(char::from_u32(0x410 + (i % 32) as u32).unwrap());
            s
        })
        .collect();
    let frames: Vec<_> = texts
        .iter()
        .map(|s| Owned { text: s.clone() }.serialize_checked().unwrap())
        .collect();
    let inlines: Vec<_> = texts
        .iter()
        .map(|s| {
            Inline {
                text: unsafe { qsl::InlineString::from_str_unchecked(s) },
            }
            .serialize_checked()
            .unwrap()
        })
        .collect();
    for f in &frames {
        Owned::validate(f).unwrap();
        Stack::validate(f).unwrap();
        View::validate(f).unwrap();
    }
    for f in &inlines {
        Inline::validate(f).unwrap();
    }
    let mut group = c.benchmark_group("utf8_1024_scalars_2048_bytes");
    group.throughput(Throughput::Bytes(2048));
    macro_rules! fresh {
        ($name:literal,$ty:ty,$frames:expr) => {
            group.bench_function($name, |b| {
                let mut i = 0;
                let mut out = MaybeUninit::<$ty>::uninit();
                b.iter(|| unsafe {
                    let frame = $frames.get_unchecked(i);
                    i = (i + 1) & 1023;
                    <$ty as Serializable>::deserialize_into(
                        black_box(frame.as_ptr()),
                        black_box(frame.len()),
                        black_box(out.as_mut_ptr()),
                    );
                    black_box(&out.assume_init_ref().text);
                    out.assume_init_drop();
                });
            });
        };
    }
    fresh!("String_new", Owned, frames);
    fresh!("StackString_new", Stack, frames);
    fresh!("InlineString_new", Inline, inlines);
    group.bench_function("str_view", |b| {
        let mut i = 0;
        let mut out = MaybeUninit::<View<'_>>::uninit();
        b.iter(|| unsafe {
            let frame = frames.get_unchecked(i);
            i = (i + 1) & 1023;
            View::deserialize_borrowed_into(
                black_box(frame.as_ptr()),
                black_box(frame.len()),
                black_box(out.as_mut_ptr()),
            );
            black_box(out.assume_init_ref().text);
        });
    });
    group.bench_function("String_reuse", |b| {
        let mut i = 0;
        let mut out = Owned {
            text: String::with_capacity(2048),
        };
        b.iter(|| unsafe {
            let f = frames.get_unchecked(i);
            i = (i + 1) & 1023;
            Owned::deserialize_prepared_into(
                black_box(f.as_ptr()),
                black_box(f.len()),
                black_box(&mut out),
            );
            black_box(out.text.as_str());
        });
    });
    group.bench_function("StackString_reuse", |b| {
        let mut i = 0;
        let mut out = Stack {
            text: qsl::StackString::default(),
        };
        b.iter(|| unsafe {
            let f = frames.get_unchecked(i);
            i = (i + 1) & 1023;
            Stack::deserialize_prepared_into(
                black_box(f.as_ptr()),
                black_box(f.len()),
                black_box(&mut out),
            );
            black_box(out.text.as_str());
        });
    });
    group.bench_function("InlineString_reuse", |b| {
        let mut i = 0;
        let mut out = Inline {
            text: qsl::InlineString::new(),
        };
        b.iter(|| unsafe {
            let f = inlines.get_unchecked(i);
            i = (i + 1) & 1023;
            Inline::deserialize_prepared_into(
                black_box(f.as_ptr()),
                black_box(f.len()),
                black_box(&mut out),
            );
            black_box(out.text.as_str());
        });
    });
    group.finish();
    let small: Vec<_> = (0..1024)
        .map(|i| {
            Owned {
                text: format!("{}{}", "a".repeat(254), char::from(b'A' + (i % 26) as u8)),
            }
            .serialize_checked()
            .unwrap()
        })
        .collect();
    let mut g = c.benchmark_group("utf8_255_scalars_255_bytes");
    g.throughput(Throughput::Bytes(255));
    g.bench_function("String_new", |b| {
        let mut i = 0;
        b.iter(|| unsafe {
            let f = small.get_unchecked(i);
            i = (i + 1) & 1023;
            let v = <Owned as Serializable>::deserialize(black_box(f));
            black_box(v.text.as_str());
        });
    });
    g.bench_function("String_reuse", |b| {
        let mut i = 0;
        let mut out = Owned {
            text: String::with_capacity(255),
        };
        b.iter(|| unsafe {
            let f = small.get_unchecked(i);
            i = (i + 1) & 1023;
            Owned::deserialize_reuse(black_box(f), black_box(&mut out));
            black_box(out.text.as_str());
        });
    });
    g.finish();
    c.final_summary();
}
