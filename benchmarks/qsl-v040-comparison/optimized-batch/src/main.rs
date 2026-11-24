use criterion::Criterion;
use qsl_optimized_bench::{
    cases::{registry, selection, COUNT, VALIDATION_OFFSET},
    MAX_FRAME,
};
use std::{env, hint::black_box, path::PathBuf};

fn main() {
    let selected = selection().unwrap();
    let offset = if selected.is_some() {
        VALIDATION_OFFSET
    } else {
        0
    };
    let mut cases = Vec::new();
    for count in [1, 8, 64, 256] {
        for case in registry() {
            let id = format!("{}/{count}", case.id());
            if selected
                .as_ref()
                .is_none_or(|entries| entries.contains(&id))
            {
                cases.push((case, count, id));
            }
        }
    }
    if let Some(entries) = selected {
        assert_eq!(entries.len(), cases.len());
    }
    if env::var_os("BENCH_REVERSE").is_some() {
        cases.reverse();
    }
    let output = PathBuf::from(env::var_os("BENCH_OUTPUT").expect("BENCH_OUTPUT must be set"));
    let mut criterion = Criterion::default()
        .confidence_level(0.99)
        .nresamples(if offset == 0 { 1000 } else { 5000 })
        .output_directory(&output)
        .configure_from_args();
    for (case, count, id) in cases {
        let mut destination = Box::new([0u8; MAX_FRAME]);
        let entries = case.warm(offset, &mut destination).unwrap();
        let mut index = 0;
        criterion.bench_function(&id, |b| {
            b.iter(|| {
                for record in &entries[index..index + count] {
                    case.run(black_box(record), black_box(&mut destination))
                        .unwrap();
                }
                index = (index + count) % COUNT;
            })
        });
    }
    criterion.final_summary();
}
