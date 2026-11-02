//! QSL feature survey. Preparation is outside timing; owned read includes destruction.
use criterion::{Criterion, criterion_group, criterion_main};
use qsl::qps_bytes::{ByteQps, CheckedRecord};
use qsl::{BorrowSerializable, InlineString, Serializable, Utf8CopyGuard, Utf8Guard};
use qsl_feature_bench::*;
use std::{
    hint::black_box,
    mem::MaybeUninit,
    sync::{Mutex, OnceLock},
    time::Duration,
};
#[repr(C, align(64))]
struct Wire {
    bytes: [u8; 8192],
    len: usize,
}
#[repr(C, align(64))]
struct Output<T>(MaybeUninit<T>);
#[derive(Clone)]
struct Meta {
    group: String,
    mode: String,
    op: &'static str,
    useful: f64,
    wire: f64,
    min: usize,
    max: usize,
    notes: &'static str,
}
fn metadata(
    group: &str,
    mode: &str,
    op: &'static str,
    useful: f64,
    lens: &[usize],
    notes: &'static str,
) {
    static ITEMS: OnceLock<Mutex<Vec<Meta>>> = OnceLock::new();
    let mut rows = ITEMS.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
    rows.push(Meta {
        group: group.into(),
        mode: mode.into(),
        op,
        useful,
        wire: lens.iter().sum::<usize>() as f64 / lens.len() as f64,
        min: *lens.iter().min().unwrap(),
        max: *lens.iter().max().unwrap(),
        notes,
    });
    if let Some(path) = std::env::var_os("QSL_FEATURE_META") {
        let rows=rows.iter().map(|m|format!("{{\"case\":\"{}/{}\",\"group\":\"{}\",\"mode\":\"{}\",\"operation\":\"{}\",\"useful_bytes\":{},\"wire_mean_bytes\":{},\"wire_min\":{},\"wire_max\":{},\"notes\":\"{}\"}}",m.group,m.mode,m.group,m.mode,m.op,m.useful,m.wire,m.min,m.max,m.notes)).collect::<Vec<_>>().join(",");
        std::fs::write(path, format!("[{rows}]\n")).unwrap();
    }
}
fn owned<T: CheckedRecord>(
    c: &mut Criterion,
    group: &str,
    mode: &str,
    values: Vec<T>,
    useful: f64,
    read: bool,
    write: bool,
) {
    assert!(values.len().is_power_of_two());
    let mask = values.len() - 1;
    let frames: Vec<_> = values
        .iter()
        .map(|v| {
            v.qps_validate_values().unwrap();
            let mut f = Wire {
                bytes: [0; 8192],
                len: 0,
            };
            f.len = unsafe { v.serialize_into_unsafe(f.bytes.as_mut_ptr()) };
            T::qps_validate_wire(&f.bytes[..f.len]).unwrap();
            f
        })
        .collect();
    let lens: Vec<_> = frames.iter().map(|f| f.len).collect();
    if read {
        let name = format!("features/{group}/read");
        metadata(
            &name,
            mode,
            "read",
            useful,
            &lens,
            "trusted decode_into; owned allocation and destruction included when required",
        );
        let mut g = c.benchmark_group(name);
        g.bench_function(mode, |b| {
            let mut i = 0;
            let mut out = Output(MaybeUninit::<T>::uninit());
            b.iter(|| {
                let f = black_box(&frames[i]);
                unsafe {
                    T::deserialize_into(f.bytes.as_ptr(), f.len, black_box(out.0.as_mut_ptr()));
                    black_box(out.0.assume_init_ref());
                    out.0.assume_init_drop();
                }
                i = (i + 1) & mask;
            });
        });
        g.finish();
    }
    if write {
        let name = format!("features/{group}/write");
        metadata(
            &name,
            mode,
            "write",
            useful,
            &lens,
            "trusted serialize_into; reusable aligned wire output",
        );
        let mut g = c.benchmark_group(name);
        g.bench_function(mode, |b| {
            let mut i = 0;
            let mut out = Wire {
                bytes: [0; 8192],
                len: 0,
            };
            b.iter(|| {
                black_box(unsafe {
                    black_box(&values[i]).serialize_into_unsafe(black_box(out.bytes.as_mut_ptr()))
                });
                black_box(&out);
                i = (i + 1) & mask;
            });
        });
        g.finish();
    }
}
fn reuse<T: CheckedRecord>(
    c: &mut Criterion,
    group: &str,
    mode: &str,
    values: Vec<T>,
    useful: f64,
) {
    let frames: Vec<_> = values
        .iter()
        .map(|v| {
            let mut f = Wire {
                bytes: [0; 8192],
                len: 0,
            };
            f.len = unsafe { v.serialize_into_unsafe(f.bytes.as_mut_ptr()) };
            T::qps_validate_wire(&f.bytes[..f.len]).unwrap();
            f
        })
        .collect();
    let lens: Vec<_> = frames.iter().map(|f| f.len).collect();
    let mut out = unsafe { T::deserialize(&frames[0].bytes[..frames[0].len]) };
    assert!(T::REUSES_STORAGE);
    let name = format!("features/{group}/read");
    metadata(
        &name,
        mode,
        "read",
        useful,
        &lens,
        "prepared reuse; destination storage prepared outside timing",
    );
    let mut g = c.benchmark_group(name);
    g.bench_function(mode, |b| {
        let mut i = 0;
        b.iter(|| {
            let f = black_box(&frames[i]);
            unsafe {
                T::deserialize_prepared_into(f.bytes.as_ptr(), f.len, black_box(&mut out));
            }
            black_box(&out);
            i = (i + 1) & (frames.len() - 1);
        });
    });
    g.finish();
}
macro_rules! borrowed {
    ($c:expr,$group:expr,$mode:expr,$values:expr,$ty:ident,$useful:expr) => {{
        let values = $values;
        let frames: Vec<_> = values
            .iter()
            .map(|v| {
                let mut f = Wire {
                    bytes: [0; 8192],
                    len: 0,
                };
                f.len =
                    unsafe { BorrowSerializable::serialize_into_unsafe(v, f.bytes.as_mut_ptr()) };
                $ty::validate(&f.bytes[..f.len]).unwrap();
                f
            })
            .collect();
        let lens: Vec<_> = frames.iter().map(|f| f.len).collect();
        let name = format!("features/{}/read", $group);
        metadata(
            &name,
            $mode,
            "read",
            $useful,
            &lens,
            "borrowed decode; source remains alive; no payload copy",
        );
        let mut g = $c.benchmark_group(name);
        g.bench_function($mode, |b| {
            let mut i = 0;
            b.iter(|| {
                let f = black_box(&frames[i]);
                let out =
                    unsafe { $ty::deserialize_borrowed_from_raw_parts(f.bytes.as_ptr(), f.len) };
                black_box(out);
                i = (i + 1) & (frames.len() - 1);
            });
        });
        g.finish();
        let name = format!("features/{}/write", $group);
        metadata(
            &name,
            $mode,
            "write",
            $useful,
            &lens,
            "borrowed source; payload copied to reusable aligned wire output",
        );
        let mut g = $c.benchmark_group(name);
        g.bench_function($mode, |b| {
            let mut i = 0;
            let mut out = Wire {
                bytes: [0; 8192],
                len: 0,
            };
            b.iter(|| {
                black_box(unsafe {
                    BorrowSerializable::serialize_into_unsafe(
                        black_box(&values[i]),
                        black_box(out.bytes.as_mut_ptr()),
                    )
                });
                black_box(&out);
                i = (i + 1) & (values.len() - 1);
            });
        });
        g.finish();
    }};
}
fn text(seed: u32, count: usize, width: usize, repeated: bool) -> String {
    let mut x = seed;
    (0..count)
        .map(|i| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            let n = if repeated { i as u32 % 8 } else { x };
            char::from_u32(if width == 1 {
                32 + n % 95
            } else {
                0x410 + n % 64
            })
            .unwrap()
        })
        .collect()
}
fn strings(c: &mut Criterion, name: &str, chars: usize, width: usize, repeated: bool, full: bool) {
    let texts: Vec<_> = (1..=16).map(|i| text(i, chars, width, repeated)).collect();
    let m = texts[0].len() as f64;
    let group = format!("strings/{name}");
    if full {
        owned(
            c,
            &group,
            "String_new",
            texts.iter().map(|s| StrOwned { text: s.clone() }).collect(),
            m,
            true,
            true,
        );
        reuse(
            c,
            &group,
            "String_reuse",
            texts.iter().map(|s| StrOwned { text: s.clone() }).collect(),
            m,
        );
        owned(
            c,
            &group,
            "CopyGuard_new",
            texts
                .iter()
                .map(|s| StrCopy {
                    text: Utf8CopyGuard::copy_of(s),
                })
                .collect(),
            m,
            true,
            true,
        );
        reuse(
            c,
            &group,
            "CopyGuard_reuse",
            texts
                .iter()
                .map(|s| StrCopy {
                    text: Utf8CopyGuard::copy_of(s),
                })
                .collect(),
            m,
        );
        borrowed!(
            c,
            &group,
            "str_borrow",
            texts
                .iter()
                .map(|s| StrBorrow { text: s.as_str() })
                .collect::<Vec<_>>(),
            StrBorrow,
            m
        );
        borrowed!(
            c,
            &group,
            "bytes_borrow",
            texts
                .iter()
                .map(|s| StrBytes { text: s.as_bytes() })
                .collect::<Vec<_>>(),
            StrBytes,
            m
        );
        borrowed!(
            c,
            &group,
            "view_borrow",
            texts
                .iter()
                .map(|s| StrView {
                    text: Utf8Guard::from_str(s)
                })
                .collect::<Vec<_>>(),
            StrView,
            m
        );
    }
    owned(
        c,
        &group,
        "InlineString",
        texts
            .iter()
            .map(|s| StrInline {
                text: unsafe { InlineString::from_str_unchecked(s) },
            })
            .collect(),
        m,
        true,
        true,
    );
    owned(
        c,
        &group,
        "string_LZ",
        texts
            .iter()
            .map(|s| StrLz {
                text: unsafe { InlineString::from_str_unchecked(s) },
            })
            .collect(),
        m,
        true,
        true,
    );
}
fn primitives(c: &mut Criterion) {
    macro_rules! dense {
        ($ty:ident,$label:literal) => {
            owned(
                c,
                "layout",
                $label,
                (0..16)
                    .map(|i| $ty {
                        a: i % 8,
                        b: i + 3,
                        c: i as u16 * 37,
                    })
                    .collect(),
                4.0,
                true,
                true,
            )
        };
    }
    dense!(DenseRaw, "default");
    dense!(DenseCompact, "compact");
    dense!(DenseAligned, "compact_alignment");
    macro_rules! opt {
        ($ty:ident,$label:literal) => {
            owned(
                c,
                "optionals_unique",
                $label,
                (0..16)
                    .map(|i| $ty {
                        id: i,
                        a: (i & 1 != 0).then_some(3),
                        b: (i & 2 != 0).then_some(37),
                        c: (i & 4 != 0).then_some(999),
                    })
                    .collect(),
                7.5,
                true,
                true,
            )
        };
    }
    opt!(OptionalDefault, "default");
    opt!(OptionalBitmap, "bitmap");
    opt!(OptionalPredict, "predict_seq");
    opt!(OptionalTable, "opt_table");
    opt!(OptionalDirect, "opt_table_predict_seq");
    macro_rules! calls {
        ($ty:ident,$label:literal) => {
            owned(
                c,
                "inline",
                $label,
                (0..16)
                    .map(|i| $ty {
                        id: i,
                        a: (i & 1 != 0).then_some(3),
                        b: (i & 2 != 0).then_some(37),
                        c: (i & 4 != 0).then_some(999),
                    })
                    .collect(),
                7.5,
                true,
                true,
            )
        };
    }
    calls!(CallDefault, "compiler_default");
    calls!(CallHint, "hint");
    calls!(CallAlways, "always");
    calls!(CallNever, "never");
    let data: Vec<_> = (0..16)
        .map(|i| (0..32).map(|j| (i * 7 + j) as u8).collect::<Vec<_>>())
        .collect();
    owned(
        c,
        "arrays",
        "Vec_dynamic",
        data.iter()
            .map(|d| VecDynamic { data: d.clone() })
            .collect(),
        32.0,
        true,
        true,
    );
    owned(
        c,
        "arrays",
        "Vec_fixed_count",
        data.iter().map(|d| VecFixed { data: d.clone() }).collect(),
        32.0,
        true,
        true,
    );
    borrowed!(
        c,
        "arrays",
        "bytes_borrow",
        data.iter()
            .map(|d| VecBorrow { data: d.as_slice() })
            .collect::<Vec<_>>(),
        VecBorrow,
        32.0
    );
    owned(
        c,
        "custom",
        "primitive",
        (0..16).map(|i| PrimitiveUnit { word: i * 37 }).collect(),
        2.0,
        true,
        true,
    );
    owned(
        c,
        "custom",
        "QslCustom",
        (0..16)
            .map(|i| CustomUnit {
                word: Token(i * 37),
            })
            .collect(),
        2.0,
        true,
        true,
    );
    owned(
        c,
        "macros",
        "manual",
        (0..16)
            .map(|i| ManualUnit {
                meta_id: i,
                meta_tick: i as u64 * 999,
                code: 37,
            })
            .collect(),
        14.0,
        true,
        true,
    );
    owned(
        c,
        "macros",
        "expanded_Rust",
        (0..16)
            .map(|i| MacroUnit {
                meta_id: i,
                meta_tick: i as u64 * 999,
                code: 37,
            })
            .collect(),
        14.0,
        true,
        true,
    );
}
fn validation(c: &mut Criterion) {
    let v = DenseRaw {
        a: 3,
        b: 17,
        c: 999,
    };
    let mut wire = Wire {
        bytes: [0; 8192],
        len: 0,
    };
    wire.len = unsafe { v.serialize_into_unsafe(wire.bytes.as_mut_ptr()) };
    let mut out = Output(MaybeUninit::<DenseRaw>::uninit());
    let name = "features/validation/read";
    let mut g = c.benchmark_group(name);
    metadata(
        name,
        "trusted",
        "read",
        4.0,
        &[wire.len],
        "trusted deserialize_into; caller has already validated",
    );
    g.bench_function("trusted", |b| {
        b.iter(|| {
            unsafe {
                DenseRaw::deserialize_into(
                    black_box(wire.bytes.as_ptr()),
                    black_box(wire.len),
                    black_box(out.0.as_mut_ptr()),
                )
            };
            black_box(&out);
        })
    });
    metadata(
        name,
        "validate_only",
        "read",
        4.0,
        &[wire.len],
        "validation only; does not materialize a native object",
    );
    g.bench_function("validate_only", |b| {
        b.iter(|| black_box(DenseRaw::validate(black_box(&wire.bytes[..wire.len]))).unwrap())
    });
    metadata(
        name,
        "checked_into",
        "read",
        4.0,
        &[wire.len],
        "complete validation plus decode into final storage",
    );
    g.bench_function("checked_into", |b| {
        b.iter(|| {
            DenseRaw::deserialize_checked_into(
                black_box(&wire.bytes[..wire.len]),
                black_box(&mut out.0),
            )
            .unwrap();
            black_box(&out);
        })
    });
    g.finish();
}
fn buffers(c: &mut Criterion) {
    let v = FixedIo {
        id: 17,
        tick: 999,
        code: 37,
    };
    let r = rt::RtIo {
        id: v.id,
        tick: v.tick,
        code: v.code,
    };
    let mut heap = qsl::SerOut::with_capacity_aligned(64, 64);
    let mut native = qsl::native::NativeRegion::new(4096).unwrap();
    native.prefault();
    let mut region = native.buffer(64, 64).unwrap();
    let name = "features/buffers/write";
    let mut g = c.benchmark_group(name);
    for mode in [
        "allocate_SerOut",
        "reuse_SerOut",
        "reuse_NativeRegion",
        "RT_lease",
    ] {
        metadata(
            name,
            mode,
            "write",
            14.0,
            &[14],
            "one record; allocation/lease release included where named; buffer creation otherwise excluded",
        );
        g.bench_function(mode, |b| match mode {
            "allocate_SerOut" => b.iter(|| black_box(v.serialize())),
            "reuse_SerOut" => b.iter(|| {
                black_box(unsafe { v.serialize_to(black_box(&mut heap)) });
                black_box(&heap);
            }),
            "reuse_NativeRegion" => b.iter(|| {
                black_box(unsafe { v.serialize_into_unsafe(black_box(region.as_mut_ptr())) });
                black_box(&region);
            }),
            _ => b.iter(|| black_box(r.serialize().unwrap())),
        });
    }
    g.finish();
}
fn byte_qps<const Q: usize>(c: &mut Criterion, name: &str, data: &[u8]) {
    let mut wire = vec![0; data.len() + 16];
    let frame = ByteQps::<Q>::encode_to(data, &mut wire).unwrap();
    let n = frame.as_slice().len();
    let mut restored = vec![0; data.len()];
    assert_eq!(frame.decode_to(&mut restored).unwrap(), data);
    let input = frame.as_slice().to_vec();
    let mode = format!("Q{Q}");
    let group = format!("features/qps_bytes/{name}/write");
    metadata(
        &group,
        &mode,
        "write",
        data.len() as f64,
        &[n],
        "safe whole-byte QPS; all scans, rank operands, raw fallback and framing included",
    );
    let mut g = c.benchmark_group(group);
    g.bench_function(&mode, |b| {
        b.iter(|| {
            black_box(
                ByteQps::<Q>::encode_to(black_box(data), black_box(&mut wire))
                    .unwrap()
                    .as_slice(),
            );
        })
    });
    g.finish();
    let group = format!("features/qps_bytes/{name}/read");
    metadata(
        &group,
        &mode,
        "read",
        data.len() as f64,
        &[n],
        "safe restore into separate prepared region; descriptor/bounds checks included",
    );
    let mut g = c.benchmark_group(group);
    g.bench_function(&mode, |b| {
        b.iter(|| {
            black_box(
                ByteQps::<Q>::decode_to(black_box(&input), black_box(&mut restored)).unwrap(),
            );
        })
    });
    g.finish();
}
fn raw_bytes(c: &mut Criterion, name: &str, data: &[u8]) {
    for op in ["read", "write"] {
        let group = format!("features/qps_bytes/{name}/{op}");
        metadata(
            &group,
            "raw_copy",
            op,
            data.len() as f64,
            &[data.len()],
            "unframed raw copy baseline; no rank analysis or envelope",
        );
        let mut output = vec![0; data.len()];
        let mut g = c.benchmark_group(group);
        g.bench_function("raw_copy", |b| {
            b.iter(|| {
                black_box(&mut output).copy_from_slice(black_box(data));
                black_box(&output);
            })
        });
        g.finish();
    }
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

fn measure(c: &mut Criterion) {
    pin();
    primitives(c);
    validation(c);
    buffers(c);
    strings(c, "ascii_255_noise", 255, 1, false, true);
    strings(c, "utf8_1024_repeated", 1024, 2, true, true);
    strings(c, "utf8_1024_noise", 1024, 2, false, false);
    let low: Vec<_> = (0..1024).map(|i| (i % 16) as u8).collect();
    let noise: Vec<_> = (0..1024).map(|i| ((i * 73 + 19) & 255) as u8).collect();
    raw_bytes(c, "low_range_1024", &low);
    raw_bytes(c, "full_range_1024", &noise);
    byte_qps::<2>(c, "low_range_1024", &low);
    byte_qps::<4>(c, "low_range_1024", &low);
    byte_qps::<8>(c, "low_range_1024", &low);
    byte_qps::<16>(c, "low_range_1024", &low);
    byte_qps::<16>(c, "full_range_1024", &noise);
}
criterion_group! {name=benches;config=Criterion::default().warm_up_time(Duration::from_secs(15)).measurement_time(Duration::from_secs(10)).sample_size(200).confidence_level(0.99).significance_level(0.01);targets=measure}
criterion_main!(benches);
