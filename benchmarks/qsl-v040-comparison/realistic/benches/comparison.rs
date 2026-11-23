use criterion::{BatchSize, Criterion};
use qsl_realistic_bench::{
    fixture::{make, RECORDS, WORKLOADS},
    Codec, CODECS,
};
use std::{hint::black_box, path::PathBuf};

fn benchmark(criterion: &mut Criterion, codec: Codec, workload: &str, reverse: bool) {
    let values: Vec<_> = (0..RECORDS).map(|i| make(workload, i)).collect();
    let frames: Vec<_> = values
        .iter()
        .map(|value| codec.encode(value.clone()).unwrap())
        .collect();
    for (frame, original) in frames.iter().zip(&values) {
        assert_eq!(codec.decode(frame).unwrap(), *original);
    }
    let mut operations = ["encode", "decode"];
    if reverse {
        operations.reverse();
    }
    for operation in operations {
        criterion.bench_function(&format!("{workload}/{operation}/{codec}"), |bench| {
            let mut i = 0;
            if operation == "encode" {
                bench.iter_batched(
                    || {
                        i = (i + 1) % RECORDS;
                        values[i].clone()
                    },
                    |value| black_box(codec.encode(black_box(value)).unwrap()),
                    BatchSize::SmallInput,
                );
            } else {
                bench.iter(|| {
                    i = (i + 1) % RECORDS;
                    black_box(codec.decode(black_box(&frames[i])).unwrap());
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
    let mut cases: Vec<_> = WORKLOADS
        .into_iter()
        .flat_map(|workload| CODECS.into_iter().map(move |codec| (workload, codec)))
        .collect();
    if reverse {
        cases.reverse();
    }
    for (workload, codec) in cases {
        benchmark(&mut criterion, codec, workload, reverse);
    }
    criterion.final_summary();
}
