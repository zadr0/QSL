use criterion::{criterion_group, criterion_main, Criterion};
use qsl_protocol_comparison::*;
use std::{hint::black_box, time::Duration};
fn chosen(name: &str) -> bool {
    std::env::var("QSL_CASE").map_or(true, |x| x == name)
}
fn fixtures(kind: Kind) -> Vec<Fixture> {
    (0..FIXTURES).map(|i| Fixture::new_kind(i, kind)).collect()
}
fn metadata(name: &str, kind: Kind, codec: &str, batch: usize, fixtures: &[Fixture], wire: usize) {
    if let Ok(path) = std::env::var("QSL_CASE_METADATA") {
        let useful = fixtures
            .iter()
            .map(|f| useful(kind, &f.record))
            .sum::<usize>() as f64
            / fixtures.len() as f64;
        std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"case":name,"kind":kind.name(),"codec":codec,"batch":batch,"useful_per_record":useful,"wire_total_for_fixture_ring":wire,"fixture_count":fixtures.len(),"contract":"checked complete traversal"})).unwrap()).unwrap();
    }
}
fn codec<const C: usize, const K: usize>(c: &mut Criterion, batch: usize, write: bool) {
    let codec = Codec::ALL[C];
    let kind = if K == 0 { Kind::Numeric } else { Kind::Mixed };
    let name = format!(
        "codec/{}/{}/b{batch}/{}",
        kind.name(),
        if write { "write" } else { "read" },
        codec.name()
    );
    if !chosen(&name) {
        return;
    }
    let fixtures = fixtures(kind);
    let mut scratch = vec![0; 8192];
    let mut encoder = Encoder::new(&mut scratch);
    let frames: Vec<Frame> = fixtures
        .iter()
        .map(|f| Frame::new(encoder.encode(codec, kind, f).unwrap()))
        .collect();
    for (f, wire) in fixtures.iter().zip(&frames) {
        assert_eq!(read_frame::<C, K>(wire).unwrap(), expected(kind, &f.record));
    }
    metadata(
        &name,
        kind,
        codec.name(),
        batch,
        &fixtures,
        frames.iter().map(|f| f.wire_len).sum(),
    );
    let mut decoder = Decoder::default();
    for frame in &frames {
        if C == 4 {
            decoder.protobuf(kind, frame.as_slice()).unwrap();
        }
    }
    let mut next = 0usize;
    if write {
        c.bench_function(&name, |b| {
            b.iter(|| {
                for j in 0..batch {
                    let wire = encoder
                        .encode(
                            Codec::ALL[C],
                            if K == 0 { Kind::Numeric } else { Kind::Mixed },
                            black_box(&fixtures[(next + j) & (FIXTURES - 1)]),
                        )
                        .unwrap();
                    black_box(wire);
                }
                next = (next + batch) & (FIXTURES - 1);
            })
        });
    } else {
        c.bench_function(&name, |b| {
            b.iter(|| {
                let mut sum = 0u64;
                for j in 0..batch {
                    sum ^= if C == 4 {
                        decoder
                            .protobuf(
                                kind,
                                black_box(frames[(next + j) & (FIXTURES - 1)].as_slice()),
                            )
                            .unwrap()
                    } else {
                        read_frame::<C, K>(black_box(&frames[(next + j) & (FIXTURES - 1)])).unwrap()
                    };
                }
                black_box(sum);
                next = (next + batch) & (FIXTURES - 1);
            })
        });
    }
}
fn tcp<const C: usize>(c: &mut Criterion, batch: usize) {
    let codec = Codec::ALL[C];
    let name = format!("transport/mixed/request_ack/b{batch}/{}_tcp", codec.name());
    if !chosen(&name) {
        return;
    }
    let fixtures = fixtures(Kind::Mixed);
    let mut scratch = vec![0; 8192];
    let mut encoder = Encoder::new(&mut scratch);
    let mut client = transport::TcpClient::start(codec).unwrap();
    let mut next = 0;
    black_box(
        client
            .call(codec, &mut encoder, &fixtures, 0, batch)
            .unwrap(),
    );
    let wire = fixtures
        .iter()
        .map(|f| encoder.encode(codec, Kind::Mixed, f).unwrap().len())
        .sum();
    metadata(&name, Kind::Mixed, codec.name(), batch, &fixtures, wire);
    c.bench_function(&name, |b| {
        b.iter(|| {
            black_box(
                client
                    .call(codec, &mut encoder, &fixtures, next, batch)
                    .unwrap(),
            );
            next = (next + batch) & (FIXTURES - 1);
        })
    });
}
fn grpc(c: &mut Criterion, batch: usize, streaming: bool) {
    let name = format!(
        "transport/mixed/request_ack/b{batch}/{}",
        if streaming { "grpc_stream" } else { "grpc" }
    );
    if !chosen(&name) {
        return;
    }
    let fixtures = fixtures(Kind::Mixed);
    let mut client = if streaming {
        transport::GrpcClient::start_stream(&fixtures, batch)
    } else {
        transport::GrpcClient::start(&fixtures, batch)
    }
    .unwrap();
    black_box(client.call(0).unwrap());
    let wire = fixtures
        .iter()
        .map(|f| prost::Message::encoded_len(&f.mixed))
        .sum();
    metadata(&name, Kind::Mixed, "grpc", batch, &fixtures, wire);
    let mut next = 0;
    c.bench_function(&name, |b| {
        b.iter(|| {
            black_box(client.call(next).unwrap());
            next = (next + 1) % (FIXTURES / batch);
        })
    });
}
fn floor(c: &mut Criterion, batch: usize) {
    let name = format!("control/loop/b{batch}");
    if !chosen(&name) {
        return;
    }
    let mut next = 0usize;
    c.bench_function(&name, |b| {
        b.iter(|| {
            for j in 0..batch {
                black_box((next + j) & (FIXTURES - 1));
            }
            next = (next + batch) & (FIXTURES - 1);
        })
    });
}
fn consume(c: &mut Criterion, batch: usize) {
    let name = format!("control/consume/mixed/b{batch}");
    if !chosen(&name) {
        return;
    }
    let fixtures = fixtures(Kind::Mixed);
    let mut next = 0usize;
    c.bench_function(&name, |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for j in 0..batch {
                sum ^= expected(
                    Kind::Mixed,
                    black_box(&fixtures[(next + j) & (FIXTURES - 1)].record),
                );
            }
            black_box(sum);
            next = (next + batch) & (FIXTURES - 1);
        })
    });
}
fn measure(c: &mut Criterion) {
    metrics::gate();
    metrics::pin(2).unwrap();
    macro_rules! cases {
        ($c:literal) => {{
            for write in [true, false] {
                codec::<$c, 0>(c, 1, write);
                for batch in BATCHES {
                    codec::<$c, 1>(c, batch, write);
                }
            }
            for batch in BATCHES {
                tcp::<$c>(c, batch);
            }
        }};
    }
    cases!(0);
    cases!(1);
    cases!(2);
    cases!(3);
    cases!(4);
    cases!(5);
    for batch in BATCHES {
        grpc(c, batch, false);
        grpc(c, batch, true);
        floor(c, batch);
        consume(c, batch);
    }
}
criterion_group! {name=benches;config=Criterion::default().warm_up_time(Duration::from_secs(15)).measurement_time(Duration::from_secs(10)).sample_size(200).confidence_level(0.99).significance_level(0.01);targets=measure}
criterion_main!(benches);
