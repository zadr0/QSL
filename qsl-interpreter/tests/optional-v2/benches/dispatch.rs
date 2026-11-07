use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qsl::Serializable;
use qsl_optional_v2_contracts::benchmark_types::*;
use std::hint::black_box;
use std::time::Duration;
fn run<T: Serializable>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    frames: &[Vec<u8>],
) {
    group.bench_with_input(BenchmarkId::new(name, 10), frames, |b, frames| {
        let mut index = 0usize;
        b.iter(|| {
            let input = black_box(frames[index & 63].as_slice());
            index = index.wrapping_add(1);
            let value = unsafe { T::deserialize(input) };
            black_box(value)
        });
    });
}
fn dispatch(c: &mut Criterion) {
    macro_rules! frames {
        ($t:ty) => {
            (0..64u64)
                .map(|i| {
                    <$t>::try_new(
                        i,
                        Some(i + 1),
                        Some(i + 2),
                        Some(i + 3),
                        Some(i + 4),
                        Some(i + 5),
                        Some(i + 6),
                        Some(i + 7),
                        Some(i + 8),
                        Some(i + 9),
                        Some(i + 10),
                    )
                    .unwrap()
                    .serialize_checked()
                    .unwrap()
                })
                .collect::<Vec<_>>()
        };
    }
    let bm = frames!(BitmapMatch);
    let bt = frames!(BitmapTable);
    let raw = frames!(TaggedMatch);
    let table = frames!(TaggedTable);
    assert_eq!(bm, bt);
    assert_eq!(raw, table);
    for frame in &bm {
        BitmapMatch::validate(frame).unwrap();
    }
    for frame in &table {
        TaggedTable::validate(frame).unwrap();
    }
    let mut group = c.benchmark_group("bitmap");
    group.throughput(Throughput::Bytes(88));
    run::<BitmapMatch>(&mut group, "match", &bm);
    run::<BitmapTable>(&mut group, "opt_table", &bt);
    group.finish();
    let mut group = c.benchmark_group("tagged");
    group.throughput(Throughput::Bytes(88));
    run::<TaggedMatch>(&mut group, "match", &raw);
    run::<TaggedTable>(&mut group, "opt_table", &table);
    group.finish();
}
criterion_group! {name=benches;config=Criterion::default().warm_up_time(Duration::from_secs(5)).measurement_time(Duration::from_secs(8)).sample_size(100).confidence_level(0.99).significance_level(0.01);targets=dispatch}
criterion_main!(benches);
