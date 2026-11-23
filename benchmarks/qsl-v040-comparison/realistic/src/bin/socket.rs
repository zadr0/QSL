use qsl_realistic_bench::{fixture::RECORDS, transport::*, Codec, Result, CODECS};
use serde_json::json;
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
    time::Instant,
};

const WARMUP: usize = 128;
const MEASURED: usize = 2048;

fn trial(codec: Codec, pass: usize) -> Result<serde_json::Value> {
    let fixtures = tcp_fixtures(codec)?;
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let address = listener.local_addr()?;
    let worker = thread::spawn(move || -> Result<()> {
        let (mut stream, _) = listener.accept()?;
        configure(&stream)?;
        let mut input = Vec::new();
        let mut output = Vec::new();
        for _ in 0..WARMUP + MEASURED {
            read_frame(&mut stream, &mut input)?;
            let response = heartbeat_reply(codec, &input)?;
            output.clear();
            append_frame(&mut output, &response);
            stream.write_all(&output)?;
            reply_events(&mut stream, codec, &mut input, &mut output)?;
        }
        Ok(())
    });
    let mut stream = TcpStream::connect(address)?;
    configure(&stream)?;
    let mut input = Vec::new();
    let mut heartbeat_ns = Vec::with_capacity(MEASURED);
    let mut event_batch_ns = Vec::with_capacity(MEASURED);
    let mut payload_bytes = [0usize; 3];
    let result = (|| -> Result<()> {
        for i in 0..WARMUP + MEASURED {
            let index = (i * 17) % RECORDS;
            let start = Instant::now();
            stream.write_all(&fixtures.heartbeat[index])?;
            read_frame(&mut stream, &mut input)?;
            let ping_time = start.elapsed();
            if input != fixtures.heartbeat_response[index] {
                return Err("heartbeat response differs".into());
            }
            let heartbeat_response_len = input.len();
            let start = Instant::now();
            stream.write_all(&fixtures.event_batches[index])?;
            read_frame(&mut stream, &mut input)?;
            let events_time = start.elapsed();
            if input != fixtures.event_ack[index] {
                return Err("event acknowledgement differs".into());
            }
            if i >= WARMUP {
                heartbeat_ns.push(ping_time.as_nanos() as u64);
                event_batch_ns.push(events_time.as_nanos() as u64);
                payload_bytes[0] += fixtures.heartbeat[index].len() + heartbeat_response_len + 4;
                payload_bytes[1] += fixtures.event_batches[index].len();
                payload_bytes[2] += input.len() + 4;
            }
        }
        Ok(())
    })();
    drop(stream);
    let checked = worker.join().map_err(|_| "TCP worker panic")?;
    result?;
    checked?;
    Ok(json!({
        "codec": codec.name(), "pass": pass, "warmup_rounds": WARMUP, "measured_rounds": MEASURED, "batch_size": EVENT_BATCH,
        "heartbeat_rtt_ns": heartbeat_ns, "event_batch_rtt_ns": event_batch_ns,
        "heartbeat_two_way_framed_bytes": payload_bytes[0],
        "events_framed_bytes": payload_bytes[1],
        "ack_framed_bytes": payload_bytes[2],
    }))
}
fn main() -> Result<()> {
    let mut rows = Vec::new();
    for pass in 1..=2 {
        let mut codecs = CODECS;
        if pass == 2 {
            codecs.reverse();
        }
        for codec in codecs {
            rows.push(trial(codec, pass)?);
            eprintln!(
                "TCP pass {pass}: {codec} verified {MEASURED} heartbeats and {} events",
                MEASURED * EVENT_BATCH
            );
        }
    }
    println!("{}", serde_json::to_string_pretty(&rows)?);
    Ok(())
}
