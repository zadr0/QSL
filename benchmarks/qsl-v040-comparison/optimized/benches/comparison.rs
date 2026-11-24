use criterion::Criterion;
use qsl_optimized_bench::{
    cases::{registry, selection, COUNT, VALIDATION_OFFSET},
    MAX_FRAME,
};
use std::{hint::black_box, path::PathBuf};

fn main() {
    let selected = selection().unwrap();
    let offset = if selected.is_some() {
        VALIDATION_OFFSET
    } else {
        0
    };
    let mut cases = registry();
    if let Some(selected) = selected {
        cases.retain(|case| selected.contains(&case.id()));
        assert_eq!(cases.len(), selected.len());
    }
    if std::env::var_os("BENCH_REVERSE").is_some() {
        cases.reverse();
    }
    let output = std::env::var_os("BENCH_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| "results/criterion".into());
    let resamples = if offset == 0 { 2000 } else { 10000 };
    let mut criterion = Criterion::default()
        .confidence_level(0.99)
        .nresamples(resamples)
        .output_directory(&output)
        .configure_from_args();
    for case in cases {
        let mut destination = Box::new([0u8; MAX_FRAME]);
        let entries = case.warm(offset, &mut destination).unwrap();
        let mut index = 0;
        criterion.bench_function(&case.id(), |b| {
            b.iter(|| {
                index = (index + 1) % COUNT;
                case.run(black_box(&entries[index]), black_box(&mut destination))
                    .unwrap();
            })
        });
    }
    criterion.final_summary();
}
