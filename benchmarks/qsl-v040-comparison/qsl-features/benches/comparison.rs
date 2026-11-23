use criterion::Criterion;
use qsl_feature_bench::{generated::*, *};
use std::{hint::black_box, path::PathBuf};

fn strings<T: TextCodec>(criterion: &mut Criterion, corpus: &str, reverse: bool) {
    let values: Vec<_> = (0..RECORDS)
        .map(|i| T::prepare(i as u32, text(corpus, i)))
        .collect();
    let mut buffer = [0u8; BUFFER];
    let frames: Vec<_> = values
        .iter()
        .map(|record| {
            let used = record.write(&mut buffer);
            assert_eq!(T::read(&buffer[..used]).fields(), record.fields());
            buffer[..used].to_vec()
        })
        .collect();
    let mut operations = ["encode", "decode"];
    if reverse {
        operations.reverse();
    }
    for operation in operations {
        let mut i = 0usize;
        criterion.bench_function(&format!("{corpus}/{operation}/{}", T::NAME), |bench| {
            if operation == "encode" {
                bench.iter(|| {
                    i = (i + 1) % RECORDS;
                    let used = black_box(&values[i]).write(&mut buffer);
                    black_box(&buffer[..used]);
                })
            } else {
                bench.iter(|| {
                    i = (i + 1) % RECORDS;
                    let value = T::read(black_box(&frames[i]));
                    black_box(&value);
                    black_box(value.digest());
                })
            }
        });
    }
}
fn native(criterion: &mut Criterion, reverse: bool) {
    let (aligned, unaligned) = native_frames();
    let mut cases = [
        ("fields", fields as fn(&[u8]) -> u64, false),
        ("copy_aligned", copied, false),
        ("copy_unaligned", copied, true),
        ("overlay_aligned", overlay, false),
    ];
    if reverse {
        cases.reverse();
    }
    for (mode, decode, shifted) in cases {
        let mut i = 0usize;
        criterion.bench_function(&format!("native/read/{mode}"), |bench| {
            if shifted {
                bench.iter(|| {
                    i = (i + 1) % RECORDS;
                    black_box(decode(black_box(&unaligned[i].0[1..])));
                });
            } else {
                bench.iter(|| {
                    i = (i + 1) % RECORDS;
                    black_box(decode(black_box(&aligned[i].0)));
                });
            }
        });
    }
}
fn main() {
    let output = std::env::var_os("BENCH_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| "results/criterion".into());
    let mut criterion = Criterion::default()
        .confidence_level(0.99)
        .nresamples(10000)
        .output_directory(&output)
        .configure_from_args();
    let reverse = std::env::var_os("BENCH_REVERSE").is_some();
    let mut corpora = CORPORA;
    let mut codecs = [
        strings::<HeapText>,
        strings::<StackText>,
        strings::<CompressedText>,
    ];
    if reverse {
        corpora.reverse();
        codecs.reverse();
        native(&mut criterion, reverse);
    }
    for corpus in corpora {
        for codec in codecs {
            codec(&mut criterion, corpus, reverse);
        }
    }
    if !reverse {
        native(&mut criterion, reverse);
    }
    criterion.final_summary();
}
