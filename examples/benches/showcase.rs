use criterion::{criterion_group, criterion_main, Criterion};
use qsl::{BorrowSerializable, Serializable};
use qsl_examples::*;
use std::{
    hint::black_box,
    mem::MaybeUninit,
    sync::{Mutex, OnceLock},
    time::Duration,
};
trait Measured: Serializable {
    fn checked_write(&self, out: &mut [u8]) -> qsl::QslResult<usize>;
    fn checked_read<'out>(
        input: &[u8],
        out: &'out mut MaybeUninit<Self>,
    ) -> qsl::QslResult<&'out mut Self>;
    fn validate_wire(input: &[u8]) -> qsl::QslResult<()>;
}
macro_rules! measured {($($ty:ty),*)=>{$(impl Measured for $ty {
 fn checked_write(&self,out:&mut[u8])->qsl::QslResult<usize>{self.serialize_checked_to(out)}
 fn checked_read<'out>(input:&[u8],out:&'out mut MaybeUninit<Self>)->qsl::QslResult<&'out mut Self>{Self::deserialize_checked_into(input,out)}
 fn validate_wire(input:&[u8])->qsl::QslResult<()>{Self::validate(input)}
})*};}
measured!(
    Numbers,
    CompactNumbers,
    PackedNumbers,
    OwnedText,
    GuardText,
    CompressedText,
    WholeText,
    Optional1Default,
    Optional1Table,
    Optional2Default,
    Optional2Table,
    Optional3Default,
    Optional3Table,
    Optional5Default,
    Optional5Table,
    Optional10Default,
    Optional10Table
);
fn meta(
    case: &str,
    group: &str,
    operation: &str,
    contract: &str,
    useful: f64,
    lens: &[usize],
    note: &str,
) {
    static ROWS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    let quote = |s: &str| format!("\"{}\"", s.replace('\\', "\\\\").replace('\"', "\\\""));
    let row=format!("{{\"case\":{},\"group\":{},\"operation\":{},\"contract\":{},\"useful_bytes\":{},\"wire_bytes\":{},\"wire_min\":{},\"wire_max\":{},\"fixtures\":{},\"note\":{}}}",quote(case),quote(group),quote(operation),quote(contract),useful,lens.iter().sum::<usize>() as f64/lens.len() as f64,lens.iter().min().unwrap(),lens.iter().max().unwrap(),lens.len(),quote(note));
    let mut rows = ROWS.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
    rows.push(row);
    if let Some(path) = std::env::var_os("QSL_EXAMPLE_META") {
        std::fs::write(path, format!("[{}]\n", rows.join(","))).unwrap();
    }
}
#[allow(clippy::too_many_arguments)] // Benchmark fixture and operation selection, not a codec API.
fn owned<T: Measured>(
    c: &mut Criterion,
    group: &str,
    mode: &str,
    values: Vec<T>,
    useful: f64,
    read: bool,
    write: bool,
    trusted: bool,
) {
    let mut buffer = [0u8; 8192];
    let frames: Vec<_> = values
        .iter()
        .map(|v| {
            let n = v.checked_write(&mut buffer).unwrap();
            let frame = buffer[..n].to_vec();
            T::validate_wire(&frame).unwrap();
            frame
        })
        .collect();
    let lens: Vec<_> = frames.iter().map(Vec::len).collect();
    let contract = if trusted { "trusted" } else { "checked" };
    if read {
        let name = format!("{group}/read/{mode}_{contract}");
        meta(&name,group,"read",contract,useful,&lens,"deserialize_into; owned construction and drop included; prepared wire and output outside timing");
        let mut i = 0usize;
        let mut out = MaybeUninit::<T>::uninit();
        c.bench_function(&name, |b| {
            b.iter(|| {
                let frame = black_box(&frames[i]);
                if trusted {
                    unsafe {
                        T::deserialize_into(
                            frame.as_ptr(),
                            frame.len(),
                            black_box(out.as_mut_ptr()),
                        );
                        black_box(out.assume_init_ref());
                    }
                } else {
                    black_box(T::checked_read(frame, black_box(&mut out)).unwrap());
                }
                unsafe { out.assume_init_drop() };
                i = (i + 1) & (FIXTURES - 1);
            })
        });
    }
    if write {
        let name = format!("{group}/write/{mode}_{contract}");
        meta(
            &name,
            group,
            "write",
            contract,
            useful,
            &lens,
            "reused output; fixture construction excluded",
        );
        let mut i = 0usize;
        c.bench_function(&name, |b| {
            b.iter(|| {
                let value = black_box(&values[i]);
                let n = if trusted {
                    unsafe { value.serialize_into_unsafe(black_box(buffer.as_mut_ptr())) }
                } else {
                    value.checked_write(black_box(&mut buffer)).unwrap()
                };
                black_box(n);
                black_box(&buffer[..n]);
                i = (i + 1) & (FIXTURES - 1);
            })
        });
    }
}
fn borrowed(c: &mut Criterion, texts: &[String], trusted: bool) {
    let values: Vec<_> = texts.iter().map(|s| BorrowedText { text: s }).collect();
    let frames: Vec<_> = values
        .iter()
        .map(|v| v.serialize_checked().unwrap())
        .collect();
    let lens: Vec<_> = frames.iter().map(Vec::len).collect();
    let contract = if trusted { "trusted" } else { "checked" };
    let useful = texts[0].len() as f64;
    let name = format!("strings_utf8_1024/read/borrowed_{contract}");
    meta(
        &name,
        "strings_utf8_1024",
        "read",
        contract,
        useful,
        &lens,
        "borrowed UTF-8 view; no owned payload copy; not memcpy throughput",
    );
    let mut i = 0usize;
    let mut out = MaybeUninit::uninit();
    c.bench_function(&name, |b| {
        b.iter(|| {
            let frame = black_box(&frames[i]);
            if trusted {
                unsafe {
                    BorrowedText::deserialize_borrowed_into(
                        frame.as_ptr(),
                        frame.len(),
                        black_box(out.as_mut_ptr()),
                    );
                    black_box(out.assume_init_ref());
                }
            } else {
                black_box(
                    BorrowedText::deserialize_checked_into(frame, black_box(&mut out)).unwrap(),
                );
            }
            i = (i + 1) & (FIXTURES - 1);
        })
    });
    if !trusted {
        let name = "strings_utf8_1024/write/borrowed_checked";
        meta(
            name,
            "strings_utf8_1024",
            "write",
            "checked",
            useful,
            &lens,
            "prepared borrowed source and reused output",
        );
        let mut i = 0usize;
        let mut out = [0; 8192];
        c.bench_function(name, |b| {
            b.iter(|| {
                let n = black_box(&values[i])
                    .serialize_checked_to(black_box(&mut out))
                    .unwrap();
                black_box(&out[..n]);
                i = (i + 1) & (FIXTURES - 1);
            })
        });
    }
}
fn optional_cases(c: &mut Criterion) {
    owned(
        c,
        "optional_N1",
        "default",
        (0..FIXTURES)
            .map(|i| Optional1Default {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
            })
            .collect(),
        4.5,
        true,
        false,
        false,
    );
    owned(
        c,
        "optional_N1",
        "opt_table",
        (0..FIXTURES)
            .map(|i| Optional1Table {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
            })
            .collect(),
        4.5,
        true,
        false,
        false,
    );
    owned(
        c,
        "optional_N2",
        "default",
        (0..FIXTURES)
            .map(|i| Optional2Default {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
                f1: ((i & (1 << 1)) != 0).then_some((i + 1) as u8),
            })
            .collect(),
        5.0,
        true,
        false,
        false,
    );
    owned(
        c,
        "optional_N2",
        "opt_table",
        (0..FIXTURES)
            .map(|i| Optional2Table {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
                f1: ((i & (1 << 1)) != 0).then_some((i + 1) as u8),
            })
            .collect(),
        5.0,
        true,
        false,
        false,
    );
    owned(
        c,
        "optional_N3",
        "default",
        (0..FIXTURES)
            .map(|i| Optional3Default {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
                f1: ((i & (1 << 1)) != 0).then_some((i + 1) as u8),
                f2: ((i & (1 << 2)) != 0).then_some((i + 2) as u8),
            })
            .collect(),
        5.5,
        true,
        false,
        false,
    );
    owned(
        c,
        "optional_N3",
        "opt_table",
        (0..FIXTURES)
            .map(|i| Optional3Table {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
                f1: ((i & (1 << 1)) != 0).then_some((i + 1) as u8),
                f2: ((i & (1 << 2)) != 0).then_some((i + 2) as u8),
            })
            .collect(),
        5.5,
        true,
        false,
        false,
    );
    owned(
        c,
        "optional_N5",
        "default",
        (0..FIXTURES)
            .map(|i| Optional5Default {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
                f1: ((i & (1 << 1)) != 0).then_some((i + 1) as u8),
                f2: ((i & (1 << 2)) != 0).then_some((i + 2) as u8),
                f3: ((i & (1 << 3)) != 0).then_some((i + 3) as u8),
                f4: ((i & (1 << 4)) != 0).then_some((i + 4) as u8),
            })
            .collect(),
        6.5,
        true,
        false,
        false,
    );
    owned(
        c,
        "optional_N5",
        "opt_table",
        (0..FIXTURES)
            .map(|i| Optional5Table {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
                f1: ((i & (1 << 1)) != 0).then_some((i + 1) as u8),
                f2: ((i & (1 << 2)) != 0).then_some((i + 2) as u8),
                f3: ((i & (1 << 3)) != 0).then_some((i + 3) as u8),
                f4: ((i & (1 << 4)) != 0).then_some((i + 4) as u8),
            })
            .collect(),
        6.5,
        true,
        false,
        false,
    );
    owned(
        c,
        "optional_N10",
        "default",
        (0..FIXTURES)
            .map(|i| Optional10Default {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
                f1: ((i & (1 << 1)) != 0).then_some((i + 1) as u8),
                f2: ((i & (1 << 2)) != 0).then_some((i + 2) as u8),
                f3: ((i & (1 << 3)) != 0).then_some((i + 3) as u8),
                f4: ((i & (1 << 4)) != 0).then_some((i + 4) as u8),
                f5: ((i & (1 << 5)) != 0).then_some((i + 5) as u8),
                f6: ((i & (1 << 6)) != 0).then_some((i + 6) as u8),
                f7: ((i & (1 << 7)) != 0).then_some((i + 7) as u8),
                f8: ((i & (1 << 8)) != 0).then_some((i + 8) as u8),
                f9: ((i & (1 << 9)) != 0).then_some((i + 9) as u8),
            })
            .collect(),
        9.0,
        true,
        false,
        false,
    );
    owned(
        c,
        "optional_N10",
        "opt_table",
        (0..FIXTURES)
            .map(|i| Optional10Table {
                id: i as u32,
                f0: ((i & (1 << 0)) != 0).then_some(i as u8),
                f1: ((i & (1 << 1)) != 0).then_some((i + 1) as u8),
                f2: ((i & (1 << 2)) != 0).then_some((i + 2) as u8),
                f3: ((i & (1 << 3)) != 0).then_some((i + 3) as u8),
                f4: ((i & (1 << 4)) != 0).then_some((i + 4) as u8),
                f5: ((i & (1 << 5)) != 0).then_some((i + 5) as u8),
                f6: ((i & (1 << 6)) != 0).then_some((i + 6) as u8),
                f7: ((i & (1 << 7)) != 0).then_some((i + 7) as u8),
                f8: ((i & (1 << 8)) != 0).then_some((i + 8) as u8),
                f9: ((i & (1 << 9)) != 0).then_some((i + 9) as u8),
            })
            .collect(),
        9.0,
        true,
        false,
        false,
    );
}
fn whole_qps(c: &mut Criterion) {
    let values: Vec<_> = (0..FIXTURES)
        .map(|i| WholeText {
            text: format!("{i:04}{}", "A".repeat(1020)),
        })
        .collect();
    let mut scratch = [0; 8192];
    let mut wire = [0; 8208];
    let frames: Vec<_> = values
        .iter()
        .map(|v| {
            v.serialize_qps_to(&mut scratch, &mut wire)
                .unwrap()
                .as_slice()
                .to_vec()
        })
        .collect();
    let lens: Vec<_> = frames.iter().map(Vec::len).collect();
    let name = "whole_qps_ascii_1024/read/qps_Q16_checked";
    meta(name,"whole_qps_ascii_1024","read","checked",1024.0,&lens,"QPB3 envelope restoration, inner validation, owned String creation and drop; scratch prepared");
    let mut i = 0usize;
    let mut restoration = [0; 8192];
    let mut out = MaybeUninit::uninit();
    c.bench_function(name, |b| {
        b.iter(|| {
            black_box(
                WholeText::deserialize_qps_into(
                    black_box(&frames[i]),
                    black_box(&mut restoration),
                    black_box(&mut out),
                )
                .unwrap(),
            );
            unsafe { out.assume_init_drop() };
            i = (i + 1) & (FIXTURES - 1);
        })
    });
    let name = "whole_qps_ascii_1024/write/qps_Q16_checked";
    meta(
        name,
        "whole_qps_ascii_1024",
        "write",
        "checked",
        1024.0,
        &lens,
        "includes inner serialization and complete QPB3 encoding; both buffers reused",
    );
    let mut i = 0usize;
    c.bench_function(name, |b| {
        b.iter(|| {
            let frame = black_box(&values[i])
                .serialize_qps_to(black_box(&mut scratch), black_box(&mut wire))
                .unwrap();
            black_box(frame.as_slice());
            i = (i + 1) & (FIXTURES - 1);
        })
    });
    owned(
        c,
        "whole_qps_ascii_1024",
        "default",
        values,
        1024.0,
        true,
        true,
        false,
    );
}
fn regions(c: &mut Criterion) {
    let values: Vec<_> = (0..FIXTURES).map(numbers).collect();
    let lens = vec![64; FIXTURES];
    let name = "buffers/write/heap_checked";
    meta(
        name,
        "buffers",
        "write",
        "checked",
        64.0,
        &lens,
        "fresh Vec allocation and deallocation included",
    );
    let mut i = 0usize;
    c.bench_function(name, |b| {
        b.iter(|| {
            let frame = black_box(&values[i]).serialize_checked().unwrap();
            black_box(&frame);
            drop(frame);
            i = (i + 1) & (FIXTURES - 1);
        })
    });
    let values: Vec<_> = (0..FIXTURES).map(rt_numbers).collect();
    let name = "buffers/write/rt_checked";
    meta(
        name,
        "buffers",
        "write",
        "checked",
        64.0,
        &lens,
        "one live lease; acquisition, value validation, write and release; no contention",
    );
    let mut i = 0usize;
    c.bench_function(name, |b| {
        b.iter(|| {
            let lease = black_box(&values[i]).serialize().unwrap();
            black_box(&lease[..]);
            drop(lease);
            i = (i + 1) & (FIXTURES - 1);
        })
    });
}
fn pin() {
    #[cfg(windows)]
    unsafe {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn GetCurrentThread() -> *mut std::ffi::c_void;
            fn GetProcessAffinityMask(
                p: *mut std::ffi::c_void,
                process: *mut usize,
                system: *mut usize,
            ) -> i32;
            fn SetThreadAffinityMask(t: *mut std::ffi::c_void, mask: usize) -> usize;
            fn SetThreadPriority(t: *mut std::ffi::c_void, priority: i32) -> i32;
        }
        let (mut process, mut system) = (0usize, 0usize);
        assert_ne!(
            GetProcessAffinityMask(GetCurrentProcess(), &mut process, &mut system),
            0
        );
        let cpu = if process & 4 != 0 {
            4
        } else {
            process.isolate_lowest_one()
        };
        assert_ne!(SetThreadAffinityMask(GetCurrentThread(), cpu), 0);
        assert_ne!(SetThreadPriority(GetCurrentThread(), 1), 0);
        eprintln!("QSL measuring thread affinity={cpu:#x}, priority=above-normal");
    }
}
fn measure(c: &mut Criterion) {
    if let Some(path) = std::env::var_os("QSL_BENCH_START_FILE") {
        let start = std::time::Instant::now();
        while !std::path::Path::new(&path).exists() {
            assert!(
                start.elapsed() < Duration::from_secs(30),
                "runner did not attach memory job"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    pin();
    owned(
        c,
        "numbers_8",
        "raw",
        (0..FIXTURES).map(numbers).collect(),
        64.0,
        true,
        true,
        false,
    );
    owned(
        c,
        "numbers_8",
        "raw",
        (0..FIXTURES).map(numbers).collect(),
        64.0,
        true,
        true,
        true,
    );
    owned(
        c,
        "numbers_8",
        "compact",
        (0..FIXTURES).map(compact_numbers).collect(),
        64.0,
        true,
        true,
        false,
    );
    owned(
        c,
        "numbers_8",
        "qps_Q16",
        (0..FIXTURES).map(packed_numbers).collect(),
        64.0,
        true,
        true,
        false,
    );
    let texts = texts();
    let useful = texts[0].len() as f64;
    owned(
        c,
        "strings_utf8_1024",
        "String",
        texts
            .iter()
            .map(|s| OwnedText { text: s.clone() })
            .collect(),
        useful,
        true,
        true,
        false,
    );
    owned(
        c,
        "strings_utf8_1024",
        "String",
        texts
            .iter()
            .map(|s| OwnedText { text: s.clone() })
            .collect(),
        useful,
        true,
        true,
        true,
    );
    borrowed(c, &texts, false);
    borrowed(c, &texts, true);
    owned(
        c,
        "strings_utf8_1024",
        "CopyGuard",
        texts
            .iter()
            .map(|s| GuardText {
                text: qsl::Utf8CopyGuard::try_from_bytes(s.as_bytes()).unwrap(),
            })
            .collect(),
        useful,
        true,
        false,
        false,
    );
    owned(
        c,
        "strings_utf8_1024",
        "LZ_inline",
        texts
            .iter()
            .map(|s| CompressedText {
                text: inline(s).unwrap(),
            })
            .collect(),
        useful,
        true,
        true,
        false,
    );
    let uniform = uniform_texts();
    owned(
        c,
        "strings_utf8_1024_uniform",
        "String",
        uniform
            .iter()
            .map(|s| OwnedText { text: s.clone() })
            .collect(),
        2048.0,
        true,
        true,
        false,
    );
    owned(
        c,
        "strings_utf8_1024_uniform",
        "LZ_inline",
        uniform
            .iter()
            .map(|s| CompressedText {
                text: inline(s).unwrap(),
            })
            .collect(),
        2048.0,
        true,
        true,
        false,
    );
    optional_cases(c);
    whole_qps(c);
    regions(c);
    let frames: Vec<_> = (0..FIXTURES)
        .map(|i| numbers(i).serialize_checked().unwrap())
        .collect();
    let lens = vec![64; FIXTURES];
    let name = "safety/validate/numbers_checked";
    meta(
        name,
        "safety",
        "validate",
        "checked",
        64.0,
        &lens,
        "validation only; no result is constructed; not interchangeable with read",
    );
    let mut i = 0usize;
    c.bench_function(name, |b| {
        b.iter(|| {
            Numbers::validate(black_box(&frames[i])).unwrap();
            i = (i + 1) & (FIXTURES - 1);
        })
    });
}
criterion_group! {name=benches;config=Criterion::default().warm_up_time(Duration::from_secs(15)).measurement_time(Duration::from_secs(10)).sample_size(200).confidence_level(0.99).significance_level(0.01);targets=measure}
criterion_main!(benches);
