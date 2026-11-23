use criterion::Criterion;
use qsl_layout_bench::{generated::*, *};
use std::{hint::black_box, path::PathBuf};

fn cases<T: Layout>(criterion: &mut Criterion, reverse: bool) {
    validate::<T>();
    let hot: Vec<_> = (0..HOT_COUNT)
        .map(|i| T::from_fields(Fields::new(i)))
        .collect();
    let frames: Vec<_> = (0..HOT_COUNT)
        .map(|i| example_frame::<T>(Fields::new(i)))
        .collect();
    let large: Vec<_> = (0..LARGE_COUNT)
        .map(|i| T::from_fields(Fields::new(i)))
        .collect();
    let mut operations = ["write", "read", "scan_hot", "scan_large"];
    if reverse {
        operations.reverse();
    }
    let mut output = [0; FRAME_SIZE];
    for operation in operations {
        let mut cursor = 0usize;
        criterion.bench_function(
            &format!("{}/{}", operation, T::NAME),
            |bench| match operation {
                "write" => bench.iter(|| {
                    cursor = (cursor + 1) & (HOT_COUNT - 1);
                    let n = black_box(&hot[cursor]).serialize(&mut output);
                    black_box(&output[..n]);
                }),
                "read" => bench.iter(|| {
                    cursor = (cursor + 1) & (HOT_COUNT - 1);
                    let record = T::deserialize(black_box(&frames[cursor]));
                    black_box(&record);
                    black_box(record.fields().digest());
                }),
                "scan_hot" => bench.iter(|| scan::<T>(&hot)),
                "scan_large" => bench.iter(|| scan::<T>(&large)),
                _ => unreachable!(),
            },
        );
    }
}
#[inline(never)]
fn scan<T: Layout>(values: &[T]) -> u64 {
    black_box(values).iter().fold(0u64, |sum, record| {
        sum.wrapping_add(record.fields().digest())
    })
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
    let mut cases = [
        cases::<Declared>,
        cases::<Auto>,
        cases::<Canonical>,
        cases::<Manual>,
    ];
    if reverse {
        cases.reverse();
    }
    for run in cases {
        run(&mut criterion, reverse);
    }
    criterion.final_summary();
}
