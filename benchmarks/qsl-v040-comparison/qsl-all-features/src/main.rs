use criterion::Criterion;
use qsl::{BorrowSerializable, Serializable};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    hint::black_box,
    path::PathBuf,
    sync::OnceLock,
};

const COUNT: usize = 64;
const CAPACITY: usize = 32768;
const SCHEMAS: &str = include_str!(concat!(env!("OUT_DIR"), "/schemas.qsl"));
const PROFILES: &str = include_str!(concat!(env!("OUT_DIR"), "/profiles.json"));

fn number(index: usize, salt: u32) -> u32 {
    (index as u32).wrapping_mul(1031 + salt * 571) & 4095
}

fn text(index: usize, length: usize, periodic: bool) -> String {
    if periodic {
        return format!("{index:08x}:é=yes!")
            .repeat(length.div_ceil(16))
            .chars()
            .scan(0, |n, c| {
                *n += c.len_utf8();
                Some((*n, c))
            })
            .take_while(|(n, _)| *n <= length)
            .map(|(_, c)| c)
            .collect();
    }
    let mut state = (index as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            char::from(b' ' + (state % 95) as u8)
        })
        .collect()
}
fn corpus(length: usize, periodic: bool, offset: usize) -> &'static [String] {
    type Corpora = HashMap<(usize, bool, usize), Vec<String>>;
    static TEXTS: OnceLock<Corpora> = OnceLock::new();
    &TEXTS.get_or_init(|| {
        let mut data = HashMap::new();
        for (length, periodic) in [
            (0, false),
            (32, false),
            (128, false),
            (256, false),
            (1024, false),
            (1024, true),
            (8192, true),
            (4096, false),
            (8192, false),
        ] {
            for offset in [0, 1024] {
                data.insert(
                    (length, periodic, offset),
                    (0..COUNT)
                        .map(|i| text(offset + i, length, periodic))
                        .collect(),
                );
            }
        }
        data
    })[&(length, periodic, offset)]
}

struct Frame {
    storage: Vec<u128>,
    len: usize,
}
impl Frame {
    fn new(input: &[u8]) -> Self {
        let mut storage = vec![0; input.len().div_ceil(16) + 1];
        // The u128 backing storage is aligned and every byte is initialized.
        let dst = unsafe {
            std::slice::from_raw_parts_mut(storage.as_mut_ptr().cast(), storage.len() * 16)
        };
        dst[..input.len()].copy_from_slice(input);
        Self {
            storage,
            len: input.len(),
        }
    }
    fn bytes(&self) -> &[u8] {
        // The backing allocation contains at least len initialized bytes.
        unsafe { std::slice::from_raw_parts(self.storage.as_ptr().cast(), self.len) }
    }
}

type Write<T> = fn(&T, &mut [u8]) -> usize;
type Read = fn(&[u8]);
type RoundTrip = fn(&[u8], &mut [u8]) -> usize;

struct Codec<T> {
    checked_write: Write<T>,
    trusted_write: Write<T>,
    checked_read: Read,
    trusted_read: Read,
    validate: Read,
    validate_values: fn(&T),
    checked_roundtrip: RoundTrip,
    trusted_roundtrip: RoundTrip,
    valid: fn(&[u8]) -> bool,
}

struct Runner {
    criterion: Criterion,
    selection: Option<HashSet<String>>,
    sizes: Vec<Value>,
    visited: HashSet<String>,
    output: PathBuf,
    verify: bool,
    reverse: bool,
}
impl Runner {
    fn regular<T>(
        &mut self,
        group: &str,
        profile: &str,
        make: fn(usize, &'static str) -> T,
        codec: Codec<T>,
        extra: Option<(&'static str, Write<T>)>,
        extra_read: Option<(&'static str, Read)>,
    ) {
        let workloads: &[(&str, usize, bool)] = match group {
            "quantized" if profile.ends_with("_ot") => &[("quantized_optional", 0, false)],
            "quantized" => &[("quantized_fixed", 0, false)],
            "safety_text" => &[
                ("text_32", 32, false),
                ("text_256", 256, false),
                ("text_1024", 1024, false),
                ("text_4096", 4096, false),
                ("text_8192", 8192, false),
                ("utf8_1024", 1024, true),
                ("utf8_8192", 8192, true),
            ],
            "string_storage" => &[("periodic_1024", 1024, true), ("noise_1024", 1024, false)],
            "optional_text" => &[("optional_text", 128, false)],
            _ => &[(group, 0, false)],
        };
        for &(workload, length, periodic) in workloads {
            let base = format!("{workload}/{profile}");
            let offset = if self.selection.is_some() { 1024 } else { 0 };
            if self
                .selection
                .as_ref()
                .is_some_and(|set| !set.iter().any(|id| id.starts_with(&format!("{base}/"))))
            {
                continue;
            }
            let mut output = vec![0; CAPACITY];
            let mut check = vec![0; CAPACITY];
            let values: Vec<_> = corpus(length, periodic, offset)
                .iter()
                .enumerate()
                .map(|(i, s)| make(offset + i, s))
                .collect();
            let frames: Vec<_> = values
                .iter()
                .map(|value| {
                    let used = (codec.checked_write)(value, &mut output);
                    let frame = Frame::new(&output[..used]);
                    let input = frame.bytes();
                    (codec.validate)(input);
                    assert_eq!((codec.trusted_write)(value, &mut check), used, "{base}");
                    assert_eq!(&check[..used], input, "{base}");
                    for roundtrip in [codec.checked_roundtrip, codec.trusted_roundtrip] {
                        assert_eq!(roundtrip(input, &mut check), used, "{base}");
                        assert_eq!(&check[..used], input, "{base}");
                    }
                    if let Some((_, write)) = extra {
                        assert_eq!(write(value, &mut check), used, "{base}");
                        assert_eq!(&check[..used], input, "{base}");
                    }
                    if let Some((_, read)) = extra_read {
                        read(input);
                    }
                    frame
                })
                .collect();
            if self.verify {
                let invalid = frames[0].bytes();
                let mut corrupt = invalid.to_vec();
                match (group, profile) {
                    ("fixed", "auto") => corrupt[..4].fill(0xff),
                    ("safety_text", "borrowed") => *corrupt.last_mut().unwrap() = 0xff,
                    ("dyn", _) => corrupt[..4].fill(0xff),
                    _ => corrupt.clear(),
                }
                if !corrupt.is_empty() {
                    assert!(
                        !(codec.valid)(&corrupt),
                        "{base} accepted a malformed frame"
                    );
                }
            }
            self.sizes.push(json!({"group":group,"workload":workload,"profile":profile,
                "object_bytes":std::mem::size_of::<T>(),"align":std::mem::align_of::<T>(),
                "mean_wire_bytes":frames.iter().map(|f|f.len).sum::<usize>() as f64 / COUNT as f64}));
            let mut ops = vec![
                "checked_write",
                "trusted_write",
                "checked_read",
                "trusted_read",
            ];
            if matches!(group, "safety_text" | "dyn" | "domain")
                || matches!(
                    (group, profile),
                    ("fixed", "auto" | "compact" | "compact_group_native")
                        | ("presence", "auto" | "ps_ot" | "bitmap")
                        | ("string_storage", "qsl_lz_inline")
                )
            {
                ops.extend(["validate", "validate_then_trusted", "validate_values"]);
            }
            if let Some((op, _)) = extra {
                ops.push(op);
            }
            if let Some((op, _)) = extra_read {
                ops.push(op);
            }
            if self.reverse {
                ops.reverse();
            }
            for op in ops {
                let id = format!("{base}/{op}");
                if self
                    .selection
                    .as_ref()
                    .is_some_and(|set| !set.contains(&id))
                {
                    continue;
                }
                assert!(self.visited.insert(id.clone()));
                if self.verify {
                    continue;
                }
                let mut index = 0;
                self.criterion.bench_function(&id, |b| match op {
                    "checked_write" | "trusted_write" => {
                        let write = if op == "checked_write" {
                            codec.checked_write
                        } else {
                            codec.trusted_write
                        };
                        b.iter(|| {
                            index = (index + 1) & (COUNT - 1);
                            let n = write(black_box(&values[index]), black_box(&mut output));
                            black_box(&output[..n]);
                        })
                    }
                    "checked_read" | "trusted_read" | "validate" => {
                        let read = match op {
                            "checked_read" => codec.checked_read,
                            "trusted_read" => codec.trusted_read,
                            _ => codec.validate,
                        };
                        b.iter(|| {
                            index = (index + 1) & (COUNT - 1);
                            read(black_box(frames[index].bytes()));
                        })
                    }
                    "validate_then_trusted" => b.iter(|| {
                        index = (index + 1) & (COUNT - 1);
                        let input = black_box(frames[index].bytes());
                        (codec.validate)(input);
                        (codec.trusted_read)(input);
                    }),
                    "validate_values" => b.iter(|| {
                        index = (index + 1) & (COUNT - 1);
                        (codec.validate_values)(black_box(&values[index]));
                    }),
                    _ if extra.is_some_and(|(name, _)| name == op) => {
                        let (_, write) = extra.unwrap();
                        b.iter(|| {
                            index = (index + 1) & (COUNT - 1);
                            let n = write(black_box(&values[index]), black_box(&mut output));
                            black_box(&output[..n]);
                        })
                    }
                    _ => {
                        let (_, read) = extra_read.unwrap();
                        b.iter(|| {
                            index = (index + 1) & (COUNT - 1);
                            read(black_box(frames[index].bytes()));
                        })
                    }
                });
            }
        }
    }
}

macro_rules! extra_write {
    ($ty:ident,cache) => {
        Some((
            "cached_write",
            (|v: &$ty, dst: &mut [u8]| {
                let mut frame = qsl::storage::CachedFrame::<32>::new();
                let bytes = v.serialize_cached(&mut frame).unwrap();
                dst[..bytes.len()].copy_from_slice(bytes);
                bytes.len()
            }) as Write<$ty>,
        ))
    };
    ($ty:ident,slots) => {
        Some((
            "rt_lease_write",
            (|v: &$ty, dst: &mut [u8]| {
                let frame = v.serialize().unwrap();
                let bytes = frame.as_slice();
                dst[..bytes.len()].copy_from_slice(bytes);
                bytes.len()
            }) as Write<$ty>,
        ))
    };
    ($ty:ident,$other:ident) => {
        None
    };
}
macro_rules! extra_read {
    ($ty:ident,overlay) => {
        Some((
            "overlay_read",
            (|bytes| {
                black_box($ty::deserialize_overlay(bytes).unwrap());
            }) as Read,
        ))
    };
    ($ty:ident,copy) => {
        Some((
            "transmute_copy_read",
            (|bytes| {
                black_box($ty::deserialize_transmutate_copy(bytes).unwrap());
            }) as Read,
        ))
    };
    ($ty:ident,$other:ident) => {
        None
    };
}
macro_rules! standard {
    ($ty:ident,$stored:ty,$bench:expr,$group:expr,$profile:expr,$make:expr,$extra:ident,$checked:expr,$trusted:expr,$encoder:ident) => {{
        $bench.regular(
            $group,
            $profile,
            $make,
            Codec::<$stored> {
                checked_write: |v, dst| v.serialize_checked_to(dst).unwrap(),
                trusted_write: |v, dst| unsafe {
                    $encoder::serialize_into_unsafe(v, dst.as_mut_ptr())
                },
                checked_read: |input| {
                    black_box(($checked)(input));
                },
                trusted_read: |input| {
                    black_box(($trusted)(input));
                },
                validate: |input| $ty::validate(input).unwrap(),
                validate_values: |v| v.validate_values().unwrap(),
                checked_roundtrip: |input, dst| {
                    ($checked)(input).serialize_checked_to(dst).unwrap()
                },
                trusted_roundtrip: |input, dst| {
                    ($trusted)(input).serialize_checked_to(dst).unwrap()
                },
                valid: |input| $ty::validate(input).is_ok(),
            },
            extra_write!($ty, $extra),
            extra_read!($ty, $extra),
        );
    }};
}
macro_rules! owned {
    ($ty:ident,$b:expr,$g:expr,$p:expr,$m:expr,$e:ident) => {standard!($ty,$ty,$b,$g,$p,$m,$e,
        |input:&[u8]|$ty::deserialize_checked(input).unwrap(),
        |input:&[u8]|unsafe{$ty::deserialize_from_raw_parts(input.as_ptr(),input.len())},
        Serializable)};
}
macro_rules! borrowed {
    ($ty:ident,$b:expr,$g:expr,$p:expr,$m:expr,$e:ident) => {
        standard!(
            $ty,
            $ty<'static>,
            $b,
            $g,
            $p,
            $m,
            $e,
            $ty::bench_checked,
            $ty::bench_trusted,
            BorrowSerializable
        )
    };
}
macro_rules! dynamic {
    ($ty:ident,$b:expr,$g:expr,$p:expr,$m:expr,$e:ident) => {{
        fn checked(input: &[u8]) -> $ty {
            $ty::deserialize_checked(input, &mut [0; 4]).unwrap()
        }
        fn trusted(input: &[u8]) -> $ty {
            unsafe { $ty::deserialize_dyn_unchecked(input.as_ptr(), input.len()) }
        }
        $b.regular(
            $g,
            $p,
            $m,
            Codec::<$ty> {
                checked_write: |v, dst| v.serialize_checked_to(dst).unwrap(),
                trusted_write: |v, dst| unsafe { v.serialize_dyn_unchecked(dst.as_mut_ptr()) },
                checked_read: |input| {
                    black_box(checked(input));
                },
                trusted_read: |input| {
                    black_box(trusted(input));
                },
                validate: |input| $ty::validate(input, &mut [0; 4]).unwrap(),
                validate_values: |v| v.validate_values().unwrap(),
                checked_roundtrip: |input, dst| checked(input).serialize_checked_to(dst).unwrap(),
                trusted_roundtrip: |input, dst| trusted(input).serialize_checked_to(dst).unwrap(),
                valid: |input| $ty::validate(input, &mut [0; 4]).is_ok(),
            },
            None,
            None,
        );
    }};
}
macro_rules! quantized {
    ($ty:ident,$b:expr,$g:expr,$p:expr,$m:expr,$e:ident) => {standard!($ty,$ty,$b,$g,$p,$m,$e,
        |input:&[u8]|$ty::deserialize_checked(input).unwrap(),
        |input:&[u8]|unsafe{$ty::deserialize_from_raw_parts(input.as_ptr(),input.len())},
        Serializable)};
}
macro_rules! overlay {
    ($ty:ident,$b:expr,$g:expr,$p:expr,$m:expr,$e:ident) => {
        standard!(
            $ty,
            $ty,
            $b,
            $g,
            $p,
            $m,
            none,
            $ty::bench_checked,
            $ty::bench_trusted,
            Serializable
        )
    };
}
include!(concat!(env!("OUT_DIR"), "/matrix.rs"));

fn main() {
    let selection = env::var_os("BENCH_SELECTION")
        .map(|p| serde_json::from_slice(&fs::read(p).unwrap()).unwrap());
    let output = PathBuf::from(env::var_os("BENCH_OUTPUT").expect("BENCH_OUTPUT is required"));
    fs::create_dir_all(&output).unwrap();
    let criterion = Criterion::default()
        .confidence_level(0.99)
        .nresamples(if selection.is_some() { 4000 } else { 1000 })
        .output_directory(&output.join("criterion"))
        .configure_from_args();
    let mut runner = Runner {
        criterion,
        selection,
        sizes: Vec::new(),
        visited: HashSet::new(),
        output,
        verify: env::var_os("BENCH_VERIFY").is_some(),
        reverse: env::var_os("BENCH_REVERSE").is_some(),
    };
    register(&mut runner);
    if let Some(expected) = runner.selection {
        assert_eq!(expected, runner.visited);
    }
    fs::write(runner.output.join("profiles.json"), PROFILES).unwrap();
    fs::write(runner.output.join("schemas.qsl"), SCHEMAS).unwrap();
    fs::write(
        runner.output.join("sizes.json"),
        serde_json::to_vec_pretty(&runner.sizes).unwrap(),
    )
    .unwrap();
    runner.criterion.final_summary();
}
