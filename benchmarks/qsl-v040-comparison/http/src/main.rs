//! Local HTTP/1.1 transport harness. No protocol-specific HTTP framework or RPC layer.
use qsl_v040_comparison::*;
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    sync::Arc,
    thread,
    time::Duration,
};

trait Endpoint: Send + Sync {
    fn id(&self) -> &str;
    fn decode(&self, body: &[u8]) -> Result<u64>;
    fn encode(&self, index: usize, buffers: &mut Buffers<'_>) -> Result<usize>;
    fn frame(&self, index: usize) -> &[u8];
    fn digest(&self, index: usize) -> u64;
}
struct Handler<C: Codec> {
    id: String,
    inputs: Vec<C::Input<'static>>,
    frames: Vec<Vec<u8>>,
    digests: Vec<u64>,
}
impl<C: Codec> Endpoint for Handler<C>
where
    C::Input<'static>: Send + Sync,
{
    fn id(&self) -> &str {
        &self.id
    }
    fn decode(&self, body: &[u8]) -> Result<u64> {
        C::read(body, |v| v.digest())
    }
    fn encode(&self, index: usize, buffers: &mut Buffers<'_>) -> Result<usize> {
        C::encode(&self.inputs[index], buffers)
    }
    fn frame(&self, index: usize) -> &[u8] {
        &self.frames[index]
    }
    fn digest(&self, index: usize) -> u64 {
        self.digests[index]
    }
}
fn register<C: Codec + 'static>(all: &mut Vec<Arc<dyn Endpoint>>, len: usize)
where
    C::Input<'static>: Send + Sync,
{
    // Fixtures and native inputs are prepared once, outside request handling.
    let fixtures: &'static [Fixture] = Box::leak(
        (0..FIXTURES)
            .map(|i| Fixture::new(i, len))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
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
    let digests: Vec<_> = fixtures.iter().map(|f| f.view(C::KIND).digest()).collect();
    for (i, frame) in frames.iter().enumerate() {
        C::read(frame, |v| assert_eq!(v, fixtures[i].view(C::KIND))).unwrap();
    }
    all.push(Arc::new(Handler::<C> {
        id: format!("{}/{}", C::KIND.name(len), C::NAME),
        inputs,
        frames,
        digests,
    }));
}
fn endpoints() -> Vec<Arc<dyn Endpoint>> {
    let mut all: Vec<Arc<dyn Endpoint>> = Vec::new();
    register::<QNumDefault>(&mut all, 0);
    register::<PNumVar>(&mut all, 0);
    register::<BNumFixed>(&mut all, 0);
    register::<CNum>(&mut all, 0);
    register::<QOptDefault>(&mut all, 0);
    register::<POptionals>(&mut all, 0);
    register::<BOptFixed>(&mut all, 0);
    register::<COptionals>(&mut all, 0);
    register::<QTextHeap>(&mut all, 1024);
    register::<PText>(&mut all, 1024);
    register::<BTextFixed>(&mut all, 1024);
    register::<CText>(&mut all, 1024);
    macro_rules! dyn_mode {
        ($num:ident, $opt:ident, $text:ident) => {
            register::<qsl_dyn_bench::$num>(&mut all, 0);
            register::<qsl_dyn_bench::$opt>(&mut all, 0);
            register::<qsl_dyn_bench::$text>(&mut all, 1024);
        };
    }
    dyn_mode!(NumLine16, OptLine16, TextLine16);
    dyn_mode!(NumLine32, OptLine32, TextLine32);
    dyn_mode!(NumLine64, OptLine64, TextLine64);
    dyn_mode!(NumDynamic32, OptDynamic32, TextDynamic32);
    dyn_mode!(NumDynamic64, OptDynamic64, TextDynamic64);
    dyn_mode!(NumStrict32, OptStrict32, TextStrict32);
    dyn_mode!(NumStrict64, OptStrict64, TextStrict64);
    all
}
fn export(all: &[Arc<dyn Endpoint>], folder: &Path) -> Result<()> {
    fs::create_dir_all(folder)?;
    let mut profiles = Vec::new();
    for endpoint in all {
        let filename = endpoint.id().replace('/', "-") + ".bin";
        let mut blob = Vec::new();
        let mut records = Vec::new();
        for i in 0..FIXTURES {
            let frame = endpoint.frame(i);
            records.push(serde_json::json!({"offset":blob.len(),"length":frame.len(),
                "digest":format!("{:016x}",endpoint.digest(i))}));
            blob.extend_from_slice(frame);
        }
        fs::write(folder.join(&filename), blob)?;
        profiles
            .push(serde_json::json!({"id":endpoint.id(),"filename":filename,"records":records}));
    }
    fs::write(
        folder.join("manifest.json"),
        serde_json::to_vec_pretty(&profiles)?,
    )?;
    Ok(())
}
fn respond(stream: &mut TcpStream, status: &str, body: &[u8], binary: bool) -> std::io::Result<()> {
    let header = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: keep-alive\r\n\r\n",
        body.len(), if binary { "application/octet-stream" } else { "text/plain" });
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}
fn connection(stream: TcpStream, all: Arc<HashMap<String, Arc<dyn Endpoint>>>) -> Result<()> {
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    let mut reader = BufReader::with_capacity(16384, stream);
    let mut scratch = vec![0; CAPACITY];
    let mut buffers = Buffers::new(&mut scratch);
    let mut body = vec![0; CAPACITY];
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        if line.len() > 2048 {
            return Err("request line too long".into());
        }
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err("invalid request line".into());
        }
        let method = parts[0].to_owned();
        let route = parts[1].to_owned();
        let mut length = 0usize;
        let mut close = false;
        let mut header_bytes = 0;
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                return Err("incomplete headers".into());
            }
            header_bytes += line.len();
            if header_bytes > 8192 {
                return Err("headers too long".into());
            }
            if line == "\r\n" {
                break;
            }
            if let Some((key, value)) = line.trim().split_once(':') {
                if key.eq_ignore_ascii_case("content-length") {
                    length = value.trim().parse()?;
                }
                if key.eq_ignore_ascii_case("transfer-encoding") {
                    return Err("chunked input unsupported".into());
                }
                if key.eq_ignore_ascii_case("connection")
                    && value.trim().eq_ignore_ascii_case("close")
                {
                    close = true;
                }
            }
        }
        if length > CAPACITY {
            return Err("request body too large".into());
        }
        reader.read_exact(&mut body[..length])?;
        if route == "/health" {
            respond(reader.get_mut(), "200 OK", b"ok", false)?;
        } else {
            // /decode|encode|control_decode|control_encode/workload/codec/fixture_index
            let route: Vec<_> = route.trim_start_matches('/').split('/').collect();
            let id = if route.len() == 4 {
                format!("{}/{}", route[1], route[2])
            } else {
                String::new()
            };
            let endpoint = if route.len() == 4 { all.get(&id) } else { None };
            if let Some(endpoint) = endpoint {
                let index = route[3].parse::<usize>().ok().filter(|i| *i < FIXTURES);
                if let Some(index) = index {
                    match (method.as_str(), route[0]) {
                        ("POST", "decode") => match endpoint.decode(&body[..length]) {
                            Ok(digest) => respond(
                                reader.get_mut(),
                                "200 OK",
                                format!("{digest:016x}").as_bytes(),
                                false,
                            )?,
                            Err(_) => respond(
                                reader.get_mut(),
                                "400 Bad Request",
                                b"invalid frame",
                                false,
                            )?,
                        },
                        ("GET", "encode") => {
                            let n = endpoint.encode(index, &mut buffers)?;
                            respond(reader.get_mut(), "200 OK", &buffers.out[..n], true)?;
                        }
                        ("POST", "control_decode") => respond(
                            reader.get_mut(),
                            "200 OK",
                            format!("{:016x}", endpoint.digest(index)).as_bytes(),
                            false,
                        )?,
                        ("GET", "control_encode") => {
                            respond(reader.get_mut(), "200 OK", endpoint.frame(index), true)?
                        }
                        _ => respond(reader.get_mut(), "405 Method Not Allowed", b"method", false)?,
                    }
                } else {
                    respond(reader.get_mut(), "400 Bad Request", b"index", false)?;
                }
            } else {
                respond(reader.get_mut(), "404 Not Found", b"route", false)?;
            }
        }
        if close {
            return Ok(());
        }
    }
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let all = endpoints();
    if args.get(1).is_some_and(|a| a == "--export") {
        export(&all, Path::new(args.get(2).ok_or("export path required")?))?;
        return Ok(());
    }
    let all = Arc::new(
        all.into_iter()
            .map(|h| (h.id().to_owned(), h))
            .collect::<HashMap<_, _>>(),
    );
    let address = args.get(1).map(String::as_str).unwrap_or("127.0.0.1:7340");
    if !address.starts_with("127.0.0.1:") {
        return Err("benchmark server must bind loopback".into());
    }
    let listener = TcpListener::bind(address)?;
    println!(
        "READY {address}; {} codec profiles; checked native decode; prepared encode",
        all.len()
    );
    for stream in listener.incoming() {
        let stream = stream?;
        let all = Arc::clone(&all);
        thread::spawn(move || {
            let _ = connection(stream, all);
        });
    }
    Ok(())
}
