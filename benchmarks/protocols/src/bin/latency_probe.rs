//! A separate request-level latency probe. Criterion has no per-request clocks.
use qsl_protocol_comparison::*;
fn main() -> Result<()> {
    metrics::gate();
    metrics::pin(2)?;
    let fixtures = (0..FIXTURES).map(Fixture::new).collect::<Vec<_>>();
    let mut rows = Vec::new();
    let timer = (0..2000)
        .map(|_| {
            let t = std::time::Instant::now();
            t.elapsed().as_nanos() as u64
        })
        .collect::<Vec<_>>();
    for batch in BATCHES {
        for codec in Codec::ALL {
            let mut scratch = vec![0; 8192];
            let mut encoder = Encoder::new(&mut scratch);
            let mut client = transport::TcpClient::start(codec)?;
            for i in 0..128 {
                std::hint::black_box(client.call(
                    codec,
                    &mut encoder,
                    &fixtures,
                    (i * batch) & (FIXTURES - 1),
                    batch,
                )?);
            }
            let mut times = Vec::with_capacity(512);
            for i in 0..512 {
                let t = std::time::Instant::now();
                std::hint::black_box(client.call(
                    codec,
                    &mut encoder,
                    &fixtures,
                    (i * batch) & (FIXTURES - 1),
                    batch,
                )?);
                times.push(t.elapsed().as_nanos() as u64);
            }
            rows.push(serde_json::json!({"codec":format!("{}_tcp",codec.name()),"batch":batch,"times_ns":times}));
        }
        for streaming in [false, true] {
            let mut client = if streaming {
                transport::GrpcClient::start_stream(&fixtures, batch)
            } else {
                transport::GrpcClient::start(&fixtures, batch)
            }?;
            for i in 0..128 {
                std::hint::black_box(client.call(i)?);
            }
            let mut times = Vec::with_capacity(512);
            for i in 0..512 {
                let t = std::time::Instant::now();
                std::hint::black_box(client.call(i)?);
                times.push(t.elapsed().as_nanos() as u64);
            }
            rows.push(serde_json::json!({"codec":if streaming{"grpc_stream"}else{"grpc"},"batch":batch,"times_ns":times}));
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({"timer_floor_ns":timer,"rows":rows}))?
    );
    Ok(())
}
