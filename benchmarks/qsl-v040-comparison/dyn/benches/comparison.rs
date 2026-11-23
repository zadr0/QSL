use criterion::Criterion;
use qsl_dyn_bench::*;
use std::{hint::black_box, path::PathBuf, time::Duration};

fn case<C: Codec>(c: &mut Criterion, len: usize) {
    let fixtures: Vec<_> = (0..FIXTURES).map(|i| Fixture::new(i, len)).collect();
    let inputs: Vec<_> = fixtures.iter().map(C::prepare).collect();
    let mut scratch = vec![0; CAPACITY];
    let mut buffers = Buffers::new(&mut scratch);
    let frames: Vec<_> = inputs
        .iter()
        .map(|v| {
            let n = C::encode(v, &mut buffers).unwrap();
            buffers.out[..n].to_vec()
        })
        .collect();
    for (fixture, frame) in fixtures.iter().zip(&frames) {
        C::read(frame, |v| assert_eq!(v, fixture.view(C::KIND))).unwrap();
    }
    let kind = C::KIND.name(len);
    let mode = if C::BORROWED {
        "read_borrowed"
    } else {
        "read_owned"
    };
    let reverse = std::env::var_os("BENCH_REVERSE").is_some();
    let mut phases = vec![false];
    if C::WRITE {
        phases.push(true)
    }
    if !reverse {
        phases.reverse()
    }
    for write in phases {
        let name = format!(
            "{kind}/{}/{name}",
            if write { "write_prepared" } else { mode },
            name = C::NAME
        );
        let mut cursor = 0usize;
        c.bench_function(&name, |b| {
            b.iter(|| {
                cursor = (cursor + 1) & (FIXTURES - 1);
                if write {
                    let n = C::encode(black_box(&inputs[cursor]), &mut buffers).unwrap();
                    black_box(&buffers.out[..n]);
                } else {
                    black_box(
                        C::read(black_box(frames[cursor].as_slice()), |v| v.digest()).unwrap(),
                    );
                }
            })
        });
    }
}
type Case = Box<dyn Fn(&mut Criterion)>;
fn register<C: Codec>(cases: &mut Vec<Case>, len: usize) {
    cases.push(Box::new(move |c| case::<C>(c, len)));
}
fn control(c: &mut Criterion, kind: Kind, len: usize) {
    let fixtures: Vec<_> = (0..FIXTURES).map(|i| Fixture::new(i, len)).collect();
    let mut cursor = 0;
    c.bench_function(&format!("{}/consume_control", kind.name(len)), |b| {
        b.iter(|| {
            cursor = (cursor + 1) & (FIXTURES - 1);
            black_box(black_box(&fixtures[cursor]).view(kind).digest())
        })
    });
}
fn main() {
    #[cfg(target_arch = "x86_64")]
    assert!(
        std::is_x86_feature_detected!("bmi2"),
        "This profile requires BMI2; use the documented portable profile on another CPU."
    );
    let output = std::env::var_os("BENCH_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("results/criterion"));
    let mut criterion = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2))
        .confidence_level(0.99)
        .nresamples(10000)
        .output_directory(&output)
        .configure_from_args();
    let mut cases: Vec<Case> = Vec::new();
    visit_dyn_cases!(register, &mut cases);
    for (kind, len) in [(Kind::Numeric, 0), (Kind::Optionals, 0), (Kind::Text, 1024)] {
        cases.push(Box::new(move |c| control(c, kind, len)));
    }
    if std::env::var_os("BENCH_REVERSE").is_some() {
        cases.reverse();
    }
    for run in cases {
        run(&mut criterion)
    }
    criterion.final_summary();
}
