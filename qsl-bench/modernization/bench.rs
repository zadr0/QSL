use criterion::{Criterion, Throughput};
use qsl::{payload_quant::QuantizedRecord, Serializable};
use qsl_modernization_bench::*;
use std::{hint::black_box, mem::MaybeUninit, time::Duration};
fn main() {
    if let Some(gate) = std::env::var_os("QSL_BENCH_START_FILE") {
        let gate = std::path::PathBuf::from(gate);
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        while !gate.exists() {
            assert!(std::time::Instant::now() < deadline, "job attach timeout");
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
            let mut s = "я".repeat(1023);
            s.push(char::from_u32(0x400 + (i % 64) as u32).unwrap());
            s
        })
        .collect();
    let frames: Vec<_> = texts
        .iter()
        .map(|text| Text { text: text.clone() }.serialize_checked().unwrap())
        .collect();
    for f in &frames {
        StackText::validate(f).unwrap();
    }
    let mut group = c.benchmark_group("modernization_string_1024_utf8_scalars");
    group.throughput(Throughput::Bytes(2048));
    group.bench_function("String_deserialize_into", |b| {
        let mut i = 0usize;
        let mut out = MaybeUninit::<Text>::uninit();
        b.iter(|| unsafe {
            let f = frames.get_unchecked(i);
            i = (i + 1) & 1023;
            <Text as Serializable>::deserialize_into(
                black_box(f.as_ptr()),
                black_box(f.len()),
                out.as_mut_ptr(),
            );
            black_box(out.assume_init_ref().text.as_bytes());
            out.assume_init_drop();
        });
    });
    group.bench_function("Stack_deserialize_into", |b| {
        let mut i = 0usize;
        let mut out = MaybeUninit::<StackText>::uninit();
        b.iter(|| unsafe {
            let f = frames.get_unchecked(i);
            i = (i + 1) & 1023;
            <StackText as Serializable>::deserialize_into(
                black_box(f.as_ptr()),
                black_box(f.len()),
                out.as_mut_ptr(),
            );
            black_box(out.assume_init_ref().text.as_bytes());
            out.assume_init_drop();
        });
    });
    group.finish();
    let raw: Vec<_> = (0..1024)
        .map(|i| {
            Raw {
                id: (i % 1001) as u64,
                signed: (i % 201) as i64 - 100,
                value: (i * 61) as u64,
            }
            .serialize_checked()
            .unwrap()
        })
        .collect();
    let packed: Vec<_> = (0..1024)
        .map(|i| {
            Packed {
                id: (i % 1001) as u64,
                signed: (i % 201) as i64 - 100,
                value: (i * 61) as u64,
            }
            .serialize_checked()
            .unwrap()
        })
        .collect();
    for f in &packed {
        Packed::validate(f).unwrap();
    }
    let mut group = c.benchmark_group("modernization_numeric_24_useful_bytes");
    group.throughput(Throughput::Bytes(24));
    group.bench_function("default_deserialize_into", |b| {
        let mut i = 0usize;
        let mut out = MaybeUninit::<Raw>::uninit();
        b.iter(|| unsafe {
            let f = raw.get_unchecked(i);
            i = (i + 1) & 1023;
            <Raw as Serializable>::deserialize_into(
                black_box(f.as_ptr()),
                black_box(f.len()),
                out.as_mut_ptr(),
            );
            black_box(out.assume_init_ref());
        });
    });
    group.bench_function("Q16_prepared_decode_into", |b| {
        let mut i = 0usize;
        let mut out = MaybeUninit::<Packed>::uninit();
        b.iter(|| unsafe {
            let f = packed.get_unchecked(i);
            i = (i + 1) & 1023;
            <Packed as QuantizedRecord>::quant_read_padded(
                black_box(f.as_ptr()),
                black_box(f.len()),
                out.as_mut_ptr(),
            );
            black_box(out.assume_init_ref());
        });
    });
    group.finish();
    c.final_summary();
}
