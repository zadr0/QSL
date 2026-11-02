use criterion::Criterion;
use qsl::{Serializable, qps_bytes::ByteQps};
use qsl_feature_bench::Whole;
use std::{
    hint::black_box,
    mem::MaybeUninit,
    sync::{Mutex, OnceLock},
    time::Duration,
};
#[repr(C, align(64))]
struct Buffer([u8; 8192]);
fn meta(group: &str, mode: &str, op: &str, m: usize, wire: usize) {
    static ROWS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    let mut rows = ROWS.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
    rows.push(format!("{{\"case\":\"{group}/{mode}\",\"group\":\"{group}\",\"mode\":\"{mode}\",\"operation\":\"{op}\",\"useful_bytes\":{m},\"wire_mean_bytes\":{wire},\"wire_min\":{wire},\"wire_max\":{wire},\"notes\":\"focused UTF-8 QPS; checks included in record APIs; prepared regions\"}}"));
    if let Some(path) = std::env::var_os("QSL_EXTRA_META") {
        std::fs::write(path, format!("[{}]", rows.join(","))).unwrap();
    }
}
fn measure(c: &mut Criterion) {
    let text = (0..1024)
        .map(|i| char::from_u32(0x410 + i % 8).unwrap())
        .collect::<String>();
    assert_eq!(text.len(), 2048);
    let input = text.as_bytes();
    let mut encoded = Buffer([0; 8192]);
    let mut restored = Buffer([0; 8192]);
    let n = ByteQps::<16>::encode_to(input, &mut encoded.0)
        .unwrap()
        .as_slice()
        .len();
    assert_eq!(
        ByteQps::<16>::decode_to(&encoded.0[..n], &mut restored.0).unwrap(),
        input
    );
    for op in ["read", "write"] {
        let name = format!("features/qps_utf8/bytes/{op}");
        let mut g = c.benchmark_group(&name);
        meta(&name, "raw_copy", op, input.len(), input.len());
        g.bench_function("raw_copy", |b| {
            b.iter(|| {
                black_box(&mut restored.0[..input.len()]).copy_from_slice(black_box(input));
                black_box(&restored);
            })
        });
        meta(&name, "Q16", op, input.len(), n);
        if op == "write" {
            g.bench_function("Q16", |b| {
                b.iter(|| {
                    black_box(
                        ByteQps::<16>::encode_to(black_box(input), black_box(&mut encoded.0))
                            .unwrap()
                            .as_slice(),
                    );
                })
            });
        } else {
            g.bench_function("Q16", |b| {
                b.iter(|| {
                    black_box(
                        ByteQps::<16>::decode_to(
                            black_box(&encoded.0[..n]),
                            black_box(&mut restored.0),
                        )
                        .unwrap(),
                    );
                })
            });
        }
        g.finish();
    }
    let value = Whole {
        id: 42,
        text,
        data: (0..32).map(|i| (i % 16) as u8).collect(),
    };
    let m = 4 + value.text.len() + value.data.len();
    let mut raw = Buffer([0; 8192]);
    let raw_len = unsafe { value.serialize_into_unsafe(raw.0.as_mut_ptr()) };
    let mut scratch = Buffer([0; 8192]);
    let n = value
        .serialize_qps_to(&mut scratch.0, &mut encoded.0)
        .unwrap()
        .as_slice()
        .len();
    let mut output = MaybeUninit::<Whole>::uninit();
    let v = Whole::deserialize_qps_into(&encoded.0[..n], &mut restored.0, &mut output).unwrap();
    assert_eq!(
        (v.id, &v.text, &v.data),
        (value.id, &value.text, &value.data)
    );
    unsafe { output.assume_init_drop() };
    let name = "features/qps_utf8/record/write";
    let mut g = c.benchmark_group(name);
    meta(name, "checked_default", "write", m, raw_len);
    g.bench_function("checked_default", |b| {
        b.iter(|| {
            black_box(value.validate_values()).unwrap();
            black_box(unsafe { value.serialize_into_unsafe(black_box(raw.0.as_mut_ptr())) });
            black_box(&raw);
        })
    });
    meta(name, "Q16", "write", m, n);
    g.bench_function("Q16", |b| {
        b.iter(|| {
            black_box(
                value
                    .serialize_qps_to(black_box(&mut scratch.0), black_box(&mut encoded.0))
                    .unwrap()
                    .as_slice(),
            );
        })
    });
    g.finish();
    let name = "features/qps_utf8/record/read";
    let mut g = c.benchmark_group(name);
    meta(name, "checked_default", "read", m, raw_len);
    g.bench_function("checked_default", |b| {
        b.iter(|| {
            Whole::deserialize_checked_into(black_box(&raw.0[..raw_len]), black_box(&mut output))
                .unwrap();
            black_box(unsafe { output.assume_init_ref() });
            unsafe { output.assume_init_drop() };
        })
    });
    meta(name, "Q16", "read", m, n);
    g.bench_function("Q16", |b| {
        b.iter(|| {
            Whole::deserialize_qps_into(
                black_box(&encoded.0[..n]),
                black_box(&mut restored.0),
                black_box(&mut output),
            )
            .unwrap();
            black_box(unsafe { output.assume_init_ref() });
            unsafe { output.assume_init_drop() };
        })
    });
    g.finish();
}
fn pin() {
    #[cfg(windows)]
    unsafe {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentThread() -> *mut std::ffi::c_void;
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn GetProcessAffinityMask(
                p: *mut std::ffi::c_void,
                allowed: *mut usize,
                system: *mut usize,
            ) -> i32;
            fn SetThreadAffinityMask(t: *mut std::ffi::c_void, mask: usize) -> usize;
            fn SetThreadPriority(t: *mut std::ffi::c_void, priority: i32) -> i32;
        }
        let mut allowed = 0;
        let mut system = 0;
        assert_ne!(
            GetProcessAffinityMask(GetCurrentProcess(), &mut allowed, &mut system),
            0
        );
        let cpu = if allowed & 4 != 0 {
            4
        } else {
            1usize << allowed.trailing_zeros()
        };
        assert_ne!(SetThreadAffinityMask(GetCurrentThread(), cpu), 0);
        assert_ne!(SetThreadPriority(GetCurrentThread(), 1), 0);
        eprintln!("Measurement affinity mask: {cpu:#x}; measuring thread priority: above normal");
    }
    eprintln!(
        "CPU features: avx2={}, bmi2={}",
        std::is_x86_feature_detected!("avx2"),
        std::is_x86_feature_detected!("bmi2")
    );
}

fn main() {
    pin();
    let mut c = Criterion::default()
        .warm_up_time(Duration::from_secs(15))
        .measurement_time(Duration::from_secs(10))
        .sample_size(200)
        .confidence_level(0.99)
        .significance_level(0.01)
        .configure_from_args();
    measure(&mut c);
    c.final_summary();
}
