use qsl_realistic_bench::{
    fixture::{make, RECORDS},
    normalize,
    transport::configure,
    Codec, Result, CODECS, MAX_FRAME,
};
use serde_json::json;
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
};

fn export(output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    let mut profiles = Vec::new();
    for codec in CODECS {
        let mut fixture_file = Vec::new();
        let mut records = Vec::new();
        for i in 0..RECORDS {
            let value = make("api", i);
            let request = codec.encode(value.clone())?;
            let response = codec.encode(normalize(value)?)?;
            let offset = fixture_file.len();
            fixture_file.extend_from_slice(&request);
            fixture_file.extend_from_slice(&response);
            records.push(
                json!({"offset": offset, "request":request.len(), "response":response.len()}),
            );
        }
        let file = format!("{codec}.bin");
        fs::write(output.join(&file), fixture_file)?;
        profiles.push(json!({"codec":codec.name(), "file":file, "records":records}));
    }
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&profiles)?,
    )?;
    Ok(())
}

fn bounded_line(input: &mut impl BufRead, line: &mut String) -> Result<usize> {
    line.clear();
    let read = input.take(2049).read_line(line)?;
    if read > 2048 {
        return Err("oversized HTTP header line".into());
    }
    Ok(read)
}

fn respond(stream: &mut TcpStream, status: &str, payload: &[u8]) -> Result<()> {
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n", payload.len())?;
    stream.write_all(payload)?;
    Ok(())
}

fn serve(stream: TcpStream) -> Result<()> {
    configure(&stream)?;
    let mut input = BufReader::new(stream);
    let mut line = String::new();
    let mut body = Vec::new();
    loop {
        if bounded_line(&mut input, &mut line)? == 0 {
            return Ok(());
        }
        let mut words = line.split_whitespace();
        let method = words.next().ok_or("missing method")?.to_owned();
        let path = words.next().ok_or("missing path")?.to_owned();
        if words.next() != Some("HTTP/1.1") || words.next().is_some() {
            return Err("invalid request line".into());
        }
        let mut length = None;
        let mut close = false;
        for count in 0..32 {
            if bounded_line(&mut input, &mut line)? == 0 {
                return Err("early end of headers".into());
            }
            if line == "\r\n" {
                break;
            }
            if count == 31 {
                return Err("too many headers".into());
            }
            let (key, value) = line.split_once(':').ok_or("invalid header")?;
            if key.eq_ignore_ascii_case("content-length") {
                if length.replace(value.trim().parse::<usize>()?).is_some() {
                    return Err("duplicate length".into());
                }
            } else if key.eq_ignore_ascii_case("transfer-encoding") {
                return Err("unsupported transfer encoding".into());
            } else if key.eq_ignore_ascii_case("connection") {
                close = value.trim().eq_ignore_ascii_case("close");
            }
        }
        let length = length.unwrap_or(0);
        if length > MAX_FRAME {
            return Err("oversized request".into());
        }
        body.resize(length, 0);
        input.read_exact(&mut body)?;
        if method == "GET" && path == "/health" {
            respond(input.get_mut(), "200 OK", b"ok")?;
        } else if let Some(codec) = path
            .strip_prefix("/v1/normalize/")
            .and_then(|name| name.parse::<Codec>().ok())
            .filter(|_| method == "POST")
        {
            let response = codec
                .decode(&body)
                .and_then(normalize)
                .and_then(|value| codec.encode(value));
            match response {
                Ok(payload) => respond(input.get_mut(), "200 OK", &payload)?,
                Err(_) => respond(input.get_mut(), "400 Bad Request", b"invalid record")?,
            }
        } else {
            respond(input.get_mut(), "404 Not Found", b"not found")?;
        }
        if close {
            return Ok(());
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("export") => {
            let output = args.next().ok_or("missing output directory")?;
            export(Path::new(&output))
        }
        Some("serve") => {
            let port = args.next().ok_or("missing port")?.parse::<u16>()?;
            let listener = TcpListener::bind(("127.0.0.1", port))?;
            eprintln!("API benchmark listening on {}", listener.local_addr()?);
            for stream in listener.incoming() {
                let stream = stream?;
                thread::spawn(move || {
                    let _ = serve(stream);
                });
            }
            Ok(())
        }
        _ => Err("expected export DIR or serve PORT".into()),
    }
}
