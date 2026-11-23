use crate::{
    fixture::{make, RECORDS},
    model::Body,
    Codec, Result, MAX_FRAME,
};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

pub const EVENT_BATCH: usize = 32;
pub const TIMEOUT: Duration = Duration::from_secs(5);

pub fn configure(stream: &TcpStream) -> Result<()> {
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    Ok(())
}

pub fn append_frame(out: &mut Vec<u8>, bytes: &[u8]) {
    assert!(bytes.len() <= MAX_FRAME);
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

pub fn read_frame(stream: &mut TcpStream, buffer: &mut Vec<u8>) -> Result<()> {
    let mut prefix = [0u8; 4];
    stream.read_exact(&mut prefix)?;
    let length = u32::from_le_bytes(prefix) as usize;
    if length > MAX_FRAME {
        return Err("TCP frame limit".into());
    }
    buffer.resize(length, 0);
    stream.read_exact(buffer)?;
    Ok(())
}

pub fn heartbeat_reply(codec: Codec, bytes: &[u8]) -> Result<Vec<u8>> {
    let mut value = codec.decode(bytes)?;
    if !matches!(value.body, Some(Body::Heartbeat(_))) {
        return Err("expected heartbeat".into());
    }
    value.revision = value.revision.checked_add(1).ok_or("revision overflow")?;
    codec.encode(value)
}

pub fn event_digest(value: &crate::model::Envelope) -> Result<u64> {
    let Some(Body::Event(event)) = value.body.as_ref() else {
        return Err("expected event".into());
    };
    let seed = u64::from(value.request_id).rotate_left(23)
        ^ u64::from(event.sequence)
        ^ u64::from(event.topic).rotate_left(37);
    Ok(event.affected.iter().fold(seed, |state, &value| {
        state.rotate_left(7) ^ u64::from(value)
    }))
}

pub fn reply_events(
    stream: &mut TcpStream,
    codec: Codec,
    input: &mut Vec<u8>,
    output: &mut Vec<u8>,
) -> Result<()> {
    let mut digest = 0u64;
    for _ in 0..EVENT_BATCH {
        read_frame(stream, input)?;
        digest = digest.wrapping_add(event_digest(&codec.decode(input)?)?);
    }
    let mut payload = [0; 12];
    payload[..4].copy_from_slice(&(EVENT_BATCH as u32).to_le_bytes());
    payload[4..].copy_from_slice(&digest.to_le_bytes());
    output.clear();
    append_frame(output, &payload);
    stream.write_all(output)?;
    Ok(())
}

pub struct TcpFixtures {
    pub heartbeat: Vec<Vec<u8>>,
    pub heartbeat_response: Vec<Vec<u8>>,
    pub event_batches: Vec<Vec<u8>>,
    pub event_ack: Vec<[u8; 12]>,
}
pub fn tcp_fixtures(codec: Codec) -> Result<TcpFixtures> {
    let mut heartbeat = Vec::new();
    let mut heartbeat_response = Vec::new();
    let mut event_batches = Vec::new();
    let mut event_ack = Vec::new();
    for i in 0..RECORDS {
        let value = make("heartbeat", i);
        let request = codec.encode(value.clone())?;
        let mut reply = value;
        reply.revision += 1;
        heartbeat_response.push(codec.encode(reply)?);
        let mut framed = Vec::new();
        append_frame(&mut framed, &request);
        heartbeat.push(framed);
        let mut batch = Vec::new();
        let mut digest = 0u64;
        for j in 0..EVENT_BATCH {
            let value = make("event", (i * 13 + j) % RECORDS);
            digest = digest.wrapping_add(event_digest(&value)?);
            append_frame(&mut batch, &codec.encode(value)?);
        }
        let mut ack = [0; 12];
        ack[..4].copy_from_slice(&(EVENT_BATCH as u32).to_le_bytes());
        ack[4..].copy_from_slice(&digest.to_le_bytes());
        event_batches.push(batch);
        event_ack.push(ack);
    }
    Ok(TcpFixtures {
        heartbeat,
        heartbeat_response,
        event_batches,
        event_ack,
    })
}
