use crate::*;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::mpsc,
    thread,
};
use tonic::{Request, Response, Status};
pub fn mixed_proto_checksum(x: &proto::Mixed) -> Result<u64> {
    let id = x.id;
    let values: &[u32; 8] = x
        .values
        .as_slice()
        .try_into()
        .map_err(|_| "numeric count")?;
    let v = values.map(u64::from);
    let text = x.text.as_str();
    let data = x.data.as_slice();
    validate_values(Kind::Mixed, id, &v, text, data)?;
    Ok(checksum(
        id,
        &v,
        text,
        data,
        x.a.map(u8::try_from).transpose()?,
        x.b.map(u16::try_from).transpose()?,
    ))
}
fn combine(sum: u64, next: u64) -> u64 {
    sum.rotate_left(5) ^ next
}
pub struct TcpClient {
    stream: TcpStream,
    thread: Option<thread::JoinHandle<Result<()>>>,
    frame: Vec<u8>,
}
impl TcpClient {
    pub fn start(codec: Codec) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        let server = thread::spawn(move || -> Result<()> {
            crate::metrics::pin(1)?;
            let (mut stream, _) = listener.accept()?;
            stream.set_nodelay(true)?;
            stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
            let mut handshake = [0u8; 8];
            stream.read_exact(&mut handshake)?;
            if handshake != [81, 83, 66, 49, codec as u8, 2, 0, 0] {
                return Err("handshake mismatch".into());
            }
            stream.write_all(&[1])?;
            let mut buf = vec![0u8; MAX_BATCH];
            let mut decoder = crate::Decoder::default();
            loop {
                let mut header = [0u8; 4];
                match stream.read_exact(&mut header) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e.into()),
                }
                let len = u32::from_le_bytes(header) as usize;
                if !(2..=MAX_BATCH).contains(&len) {
                    return Err("invalid batch length".into());
                }
                stream.read_exact(&mut buf[..len])?;
                let count = u16::from_le_bytes(buf[..2].try_into()?) as usize;
                if !(1..=64).contains(&count) {
                    return Err("invalid batch count".into());
                }
                let mut at = 2usize;
                let mut sum = 0;
                for _ in 0..count {
                    let end = at.checked_add(2).ok_or("length overflow")?;
                    let size = u16::from_le_bytes(
                        buf.get(at..end).ok_or("short record header")?.try_into()?,
                    ) as usize;
                    at = end;
                    let end = at.checked_add(size).ok_or("length overflow")?;
                    let record = buf.get(at..end).ok_or("short record")?;
                    let value = if codec == Codec::Protobuf {
                        decoder.protobuf(Kind::Mixed, record)?
                    } else {
                        read(codec, Kind::Mixed, record)?
                    };
                    sum = combine(sum, value);
                    at = end;
                }
                if at != len {
                    return Err("trailing batch bytes".into());
                }
                stream.write_all(&sum.to_le_bytes())?;
            }
            Ok(())
        });
        let mut stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;
        stream.write_all(&[81, 83, 66, 49, codec as u8, 2, 0, 0])?;
        let mut ack = [0];
        stream.read_exact(&mut ack)?;
        if ack != [1] {
            return Err("handshake rejected".into());
        }
        Ok(Self {
            stream,
            thread: Some(server),
            frame: Vec::with_capacity(MAX_BATCH + 4),
        })
    }
    pub fn call(
        &mut self,
        codec: Codec,
        encoder: &mut Encoder<'_>,
        fixtures: &[Fixture],
        start: usize,
        batch: usize,
    ) -> Result<u64> {
        self.frame.clear();
        self.frame.extend_from_slice(&[0; 4]);
        self.frame.extend_from_slice(&(batch as u16).to_le_bytes());
        for j in 0..batch {
            let wire =
                encoder.encode(codec, Kind::Mixed, &fixtures[(start + j) & (FIXTURES - 1)])?;
            let len = u16::try_from(wire.len())?;
            self.frame.extend_from_slice(&len.to_le_bytes());
            self.frame.extend_from_slice(wire);
        }
        let len = self.frame.len() - 4;
        if len > MAX_BATCH {
            return Err("batch too large".into());
        }
        self.frame[..4].copy_from_slice(&(len as u32).to_le_bytes());
        self.stream.write_all(&self.frame)?;
        let mut ack = [0; 8];
        self.stream.read_exact(&mut ack)?;
        Ok(u64::from_le_bytes(ack))
    }
}
impl Drop for TcpClient {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
#[derive(Default)]
struct Service;
#[tonic::async_trait]
impl proto::exchange_server::Exchange for Service {
    type FlowStream =
        tokio_stream::wrappers::ReceiverStream<std::result::Result<proto::Ack, Status>>;
    async fn flow(
        &self,
        request: Request<tonic::Streaming<proto::MixedBatch>>,
    ) -> std::result::Result<Response<Self::FlowStream>, Status> {
        let mut input = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            loop {
                match input.message().await {
                    Ok(Some(batch)) => {
                        let result = (|| {
                            if !(1..=64).contains(&batch.records.len()) {
                                return Err(Status::invalid_argument("batch count"));
                            }
                            let mut sum = 0;
                            for record in &batch.records {
                                sum = combine(
                                    sum,
                                    mixed_proto_checksum(record)
                                        .map_err(|_| Status::invalid_argument("record domain"))?,
                                );
                            }
                            Ok(proto::Ack { checksum: sum })
                        })();
                        let failed = result.is_err();
                        if tx.send(result).await.is_err() || failed {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    async fn submit(
        &self,
        request: Request<proto::MixedBatch>,
    ) -> std::result::Result<Response<proto::Ack>, Status> {
        let batch = request.into_inner();
        if !(1..=64).contains(&batch.records.len()) {
            return Err(Status::invalid_argument("batch count"));
        }
        let mut sum = 0;
        for record in &batch.records {
            sum = combine(
                sum,
                mixed_proto_checksum(record)
                    .map_err(|_| Status::invalid_argument("record domain"))?,
            );
        }
        Ok(Response::new(proto::Ack { checksum: sum }))
    }
}
pub struct GrpcClient {
    stream: Option<(
        tokio::sync::mpsc::Sender<proto::MixedBatch>,
        tonic::Streaming<proto::Ack>,
    )>,
    runtime: tokio::runtime::Runtime,
    client: proto::exchange_client::ExchangeClient<tonic::transport::Channel>,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<thread::JoinHandle<Result<()>>>,
    batches: Vec<proto::MixedBatch>,
}
impl GrpcClient {
    pub fn start(fixtures: &[Fixture], batch: usize) -> Result<Self> {
        let (ready, receive) = mpsc::channel();
        let (stop, shutdown) = tokio::sync::oneshot::channel();
        let server = thread::spawn(move || -> Result<()> {
            crate::metrics::pin(1)?;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            rt.block_on(async move{
 let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await?;ready.send(listener.local_addr()?)?;
 let router=tonic::transport::Server::builder().tcp_nodelay(true).initial_connection_window_size(1024*1024).initial_stream_window_size(1024*1024).http2_adaptive_window(Some(false)).add_service(proto::exchange_server::ExchangeServer::new(Service).max_decoding_message_size(MAX_BATCH).max_encoding_message_size(MAX_BATCH));
 tokio::select!{result=router.serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))=>{result?;Ok(())},_=shutdown=>Ok(())}

 })
        });
        let addr = receive.recv()?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let channel = runtime.block_on(
            tonic::transport::Endpoint::from_shared(format!("http://{addr}"))?
                .tcp_nodelay(true)
                .buffer_size(1)
                .concurrency_limit(1)
                .initial_connection_window_size(1024 * 1024)
                .initial_stream_window_size(1024 * 1024)
                .http2_adaptive_window(false)
                .connect(),
        )?;
        let client = proto::exchange_client::ExchangeClient::new(channel)
            .max_decoding_message_size(MAX_BATCH)
            .max_encoding_message_size(MAX_BATCH);
        let batches = (0..FIXTURES / batch)
            .map(|i| proto::MixedBatch {
                records: (0..batch)
                    .map(|j| fixtures[(i * batch + j) & (FIXTURES - 1)].mixed.clone())
                    .collect(),
            })
            .collect();
        Ok(Self {
            stream: None,
            runtime,
            client,
            stop: Some(stop),
            thread: Some(server),
            batches,
        })
    }
    pub fn start_stream(fixtures: &[Fixture], batch: usize) -> Result<Self> {
        let mut result = Self::start(fixtures, batch)?;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let stream = result
            .runtime
            .block_on(
                result
                    .client
                    .flow(tokio_stream::wrappers::ReceiverStream::new(rx)),
            )?
            .into_inner();
        result.stream = Some((tx, stream));
        Ok(result)
    }
    pub fn call(&mut self, index: usize) -> Result<u64> {
        let request = self.batches[index % self.batches.len()].clone();
        if let Some((tx, rx)) = self.stream.as_mut() {
            return self.runtime.block_on(async {
                tx.send(request).await?;
                Ok(rx.message().await?.ok_or("gRPC stream ended")?.checksum)
            });
        }
        Ok(self
            .runtime
            .block_on(self.client.submit(request))?
            .into_inner()
            .checksum)
    }
}
impl Drop for GrpcClient {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
