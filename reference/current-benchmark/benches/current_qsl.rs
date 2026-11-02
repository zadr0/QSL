//! Current QSL only: one data shape, four narrow prepared-I/O measurements.
use criterion::{Criterion, criterion_group, criterion_main};
use qsl::{Serializable, payload_quant::QuantizedRecord};
use qsl_current_bench::{Qps, Raw};
use std::{hint::black_box, mem::MaybeUninit, time::Duration};
#[repr(C, align(64))]
struct Wire {
    bytes: [u8; 128],
    len: usize,
}
#[repr(C, align(64))]
struct Output<T>(MaybeUninit<T>);
fn measure(c: &mut Criterion) {
    let raw: Vec<_> = (0..16)
        .map(|m| Raw {
            base: -7,
            a: 13,
            b: (m & 1 != 0).then_some(999),
            c: (m & 2 != 0).then_some(-37),
            constant: (m & 4 != 0).then_some(7),
            wide: (m & 8 != 0).then_some(0x123456789abcdef0),
        })
        .collect();
    let packed: Vec<_> = raw
        .iter()
        .map(|v| Qps {
            base: v.base,
            a: v.a,
            b: v.b,
            c: v.c,
            constant: v.constant,
            wide: v.wide,
        })
        .collect();
    let raws: Vec<_> = raw
        .iter()
        .map(|v| {
            let mut f = Wire {
                bytes: [0; 128],
                len: 0,
            };
            f.len = unsafe { v.serialize_into_unsafe(f.bytes.as_mut_ptr()) };
            Raw::validate(&f.bytes[..f.len]).unwrap();
            f
        })
        .collect();
    let qps: Vec<_> = packed
        .iter()
        .map(|v| {
            let mut f = Wire {
                bytes: [0; 128],
                len: 0,
            };
            f.len = v
                .serialize_quantized_to(&mut f.bytes)
                .unwrap()
                .as_slice()
                .len();
            Qps::validate(&f.bytes[..f.len]).unwrap();
            f
        })
        .collect();
    for (a, b) in raws.iter().zip(&qps) {
        let x = Raw::deserialize_checked(&a.bytes[..a.len]).unwrap();
        let y = Qps::deserialize_checked(&b.bytes[..b.len]).unwrap();
        assert_eq!(
            (x.base, x.a, x.b, x.c, x.constant, x.wide),
            (y.base, y.a, y.b, y.c, y.constant, y.wide)
        );
        assert_eq!(a as *const _ as usize % 64, 0);
        assert_eq!(b as *const _ as usize % 64, 0);
    }
    let mut g = c.benchmark_group("qsl_current/deserialize_into");
    g.bench_function("raw_opt_table", |b| {
        let mut i = 0;
        let mut out = Output(MaybeUninit::<Raw>::uninit());
        b.iter(|| {
            let f = black_box(&raws[i]);
            unsafe {
                Raw::deserialize_into(f.bytes.as_ptr(), f.len, black_box(out.0.as_mut_ptr()));
            }
            black_box(&out);
            i = (i + 1) & 15;
        });
    });
    g.bench_function("qps_Q16_opt_table", |b| {
        let mut i = 0;
        let mut out = Output(MaybeUninit::<Qps>::uninit());
        b.iter(|| {
            let f = black_box(&qps[i]);
            unsafe {
                Qps::deserialize_into(f.bytes.as_ptr(), f.len, black_box(out.0.as_mut_ptr()));
            }
            black_box(&out);
            i = (i + 1) & 15;
        });
    });
    g.finish();
    let mut g = c.benchmark_group("qsl_current/serialize_into");
    g.bench_function("raw_opt_table", |b| {
        let mut i = 0;
        let mut out = Wire {
            bytes: [0; 128],
            len: 0,
        };
        b.iter(|| {
            let v = black_box(&raw[i]);
            black_box(unsafe { v.serialize_into_unsafe(black_box(out.bytes.as_mut_ptr())) });
            black_box(&out);
            i = (i + 1) & 15;
        });
    });
    g.bench_function("qps_Q16_opt_table", |b| {
        let mut i = 0;
        let mut out = Wire {
            bytes: [0; 128],
            len: 0,
        };
        b.iter(|| {
            let v = black_box(&packed[i]);
            black_box(unsafe { v.quant_write_prepared(black_box(out.bytes.as_mut_ptr())) });
            black_box(&out);
            i = (i + 1) & 15;
        });
    });
    g.finish();
}
criterion_group! {name=benches;config=Criterion::default().warm_up_time(Duration::from_secs(15)).measurement_time(Duration::from_secs(10)).sample_size(200).confidence_level(0.99).significance_level(0.01);targets=measure}
criterion_main!(benches);
