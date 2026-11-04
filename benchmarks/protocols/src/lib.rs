use prost::Message;
use std::error::Error;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
pub const MAX_RECORD: usize = 65535;
pub const MAX_BATCH: usize = 256 * 1024;
pub const BATCHES: [usize; 4] = [1, 4, 16, 64];
pub const FIXTURES: usize = 1024;
pub mod records_capnp {
    include!(concat!(env!("OUT_DIR"), "/records_capnp.rs"));
}
#[allow(
    unused_imports,
    dead_code,
    clippy::missing_safety_doc,
    clippy::extra_unused_lifetimes,
    clippy::derivable_impls
)]
pub mod flat {
    include!(concat!(env!("OUT_DIR"), "/records_generated.rs"));
}
pub mod proto {
    tonic::include_proto!("comparison");
}
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/qsl_records.rs"));
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Numeric,
    Mixed,
}
impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Numeric => "numeric",
            Self::Mixed => "mixed",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codec {
    QslDefault,
    QslOptimized,
    Capnp,
    Flatbuffers,
    Protobuf,
    Bincode,
}
impl Codec {
    pub const ALL: [Self; 6] = [
        Self::QslDefault,
        Self::QslOptimized,
        Self::Capnp,
        Self::Flatbuffers,
        Self::Protobuf,
        Self::Bincode,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::QslDefault => "qsl_default",
            Self::QslOptimized => "qsl_optimized",
            Self::Capnp => "capnp",
            Self::Flatbuffers => "flatbuffers",
            Self::Protobuf => "protobuf",
            Self::Bincode => "bincode",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Record {
    pub id: u64,
    pub v: [u64; 8],
    pub text: String,
    pub data: Vec<u8>,
    pub a: Option<u8>,
    pub b: Option<u16>,
}
pub struct Fixture {
    pub record: Record,
    pub numeric: proto::Numeric,
    pub mixed: proto::Mixed,
}
impl Fixture {
    pub fn new(i: usize) -> Self {
        Self::new_kind(i, Kind::Mixed)
    }
    pub fn new_kind(i: usize, kind: Kind) -> Self {
        let text: String = if kind == Kind::Numeric {
            String::new()
        } else {
            "Рад, так Рад! "
                .chars()
                .cycle()
                .take(1024)
                .enumerate()
                .map(|(j, c)| {
                    if j < 3 {
                        char::from_u32(0x410 + ((i >> (j * 5)) & 31) as u32).unwrap()
                    } else {
                        c
                    }
                })
                .collect()
        };
        let record = Record {
            id: i as u64,
            v: std::array::from_fn(|j| ((i * 37 + j * 101) & 4095) as u64),
            text,
            data: if kind == Kind::Numeric {
                Vec::new()
            } else {
                (0..128).map(|j| ((i + j * 73) & 255) as u8).collect()
            },
            a: (i & 1 != 0).then_some(i as u8),
            b: (i & 2 != 0).then_some((i * 13) as u16),
        };
        let numeric = numeric_proto(&record);
        let mixed = mixed_proto(&record);
        Self {
            record,
            numeric,
            mixed,
        }
    }
}
pub fn numeric_proto(r: &Record) -> proto::Numeric {
    proto::Numeric {
        id: r.id,
        values: r.v.map(|v| v as u32).to_vec(),
        a: r.a.map(u32::from),
        b: r.b.map(u32::from),
    }
}
pub fn mixed_proto(r: &Record) -> proto::Mixed {
    proto::Mixed {
        id: r.id,
        values: r.v.map(|v| v as u32).to_vec(),
        text: r.text.clone(),
        data: r.data.clone(),
        a: r.a.map(u32::from),
        b: r.b.map(u32::from),
    }
}
#[derive(bincode::Encode, bincode::BorrowDecode)]
struct Num {
    id: u64,
    v: [u16; 8],
    a: Option<u8>,
    b: Option<u16>,
}
#[derive(bincode::Encode, bincode::BorrowDecode)]
struct Mix<'a> {
    id: u64,
    v: [u16; 8],
    text: &'a str,
    data: &'a [u8],
    a: Option<u8>,
    b: Option<u16>,
}
fn invalid<T>() -> Result<T> {
    Err("invalid record".into())
}
pub fn validate_values(kind: Kind, id: u64, v: &[u64; 8], text: &str, data: &[u8]) -> Result<()> {
    let _ = id;
    if v.iter().any(|&n| n > 4095) {
        return invalid();
    }
    if kind == Kind::Mixed && (text.is_empty() || text.len() > 4096 || data.len() > 256) {
        return invalid();
    }
    Ok(())
}
#[inline(never)]
pub fn checksum(
    id: u64,
    v: &[u64; 8],
    text: &str,
    data: &[u8],
    a: Option<u8>,
    b: Option<u16>,
) -> u64 {
    let mut sum = id;
    for &n in v {
        sum = sum.rotate_left(7) ^ n;
    }
    sum ^= xxhash_rust::xxh3::xxh3_64(text.as_bytes());
    sum = sum.rotate_left(13) ^ xxhash_rust::xxh3::xxh3_64(data);
    sum ^ a.map_or(0, |x| 0x10000 | u64::from(x)) ^ b.map_or(0, |x| 0x100000000 | u64::from(x))
}
pub fn expected(kind: Kind, r: &Record) -> u64 {
    checksum(
        r.id,
        &r.v,
        if kind == Kind::Mixed { &r.text } else { "" },
        if kind == Kind::Mixed { &r.data } else { &[] },
        r.a,
        r.b,
    )
}
pub fn useful(kind: Kind, r: &Record) -> usize {
    72 + if kind == Kind::Mixed {
        r.text.len() + r.data.len()
    } else {
        0
    } + usize::from(r.a.is_some())
        + 2 * usize::from(r.b.is_some())
}
pub struct Encoder<'a> {
    buf: Vec<u8>,
    flat: flatbuffers::FlatBufferBuilder<'static>,
    cap: capnp::message::ScratchSpaceHeapAllocator<'a>,
}
impl<'a> Encoder<'a> {
    pub fn new(scratch: &'a mut [u8]) -> Self {
        Self {
            buf: vec![0; 8192],
            flat: flatbuffers::FlatBufferBuilder::with_capacity(8192),
            cap: capnp::message::ScratchSpaceHeapAllocator::new(scratch),
        }
    }
    #[inline(always)]
    pub fn encode(&mut self, codec: Codec, kind: Kind, f: &Fixture) -> Result<&[u8]> {
        let r = &f.record;
        if !matches!(codec, Codec::QslDefault | Codec::QslOptimized)
            || (codec == Codec::QslOptimized && kind == Kind::Mixed)
        {
            validate_values(kind, r.id, &r.v, &r.text, &r.data)?;
        }
        match (codec, kind) {
            (Codec::QslDefault, Kind::Numeric) => {
                self.buf.resize(8192, 0);
                let value = generated::NumericDefault {
                    id: r.id,
                    v0: r.v[0],
                    v1: r.v[1],
                    v2: r.v[2],
                    v3: r.v[3],
                    v4: r.v[4],
                    v5: r.v[5],
                    v6: r.v[6],
                    v7: r.v[7],
                    a: r.a,
                    b: r.b,
                };
                let used = value.serialize_checked_to(&mut self.buf)?;
                Ok(&self.buf[..used])
            }
            (Codec::QslOptimized, Kind::Numeric) => {
                self.buf.resize(8192, 0);
                let value = generated::NumericOptimized {
                    id: r.id,
                    v0: r.v[0],
                    v1: r.v[1],
                    v2: r.v[2],
                    v3: r.v[3],
                    v4: r.v[4],
                    v5: r.v[5],
                    v6: r.v[6],
                    v7: r.v[7],
                    a: r.a,
                    b: r.b,
                };
                let used = value
                    .serialize_quantized_to(&mut self.buf)?
                    .as_slice()
                    .len();
                Ok(&self.buf[..used])
            }
            (Codec::Protobuf, Kind::Numeric) => {
                self.buf.clear();
                f.numeric.encode(&mut self.buf)?;
                Ok(&self.buf)
            }
            (Codec::Bincode, Kind::Numeric) => {
                self.buf.resize(8192, 0);
                let value = Num {
                    id: r.id,
                    v: r.v.map(|v| v as u16),
                    a: r.a,
                    b: r.b,
                };
                let used = bincode::encode_into_slice(
                    value,
                    &mut self.buf,
                    bincode::config::standard()
                        .with_fixed_int_encoding()
                        .with_limit::<65535>(),
                )?;
                Ok(&self.buf[..used])
            }
            (Codec::Capnp, Kind::Numeric) => {
                self.buf.clear();
                let mut message = capnp::message::Builder::new(&mut self.cap);
                {
                    let mut root = message.init_root::<records_capnp::numeric::Builder>();
                    root.set_id(r.id);
                    root.set_v0(r.v[0] as u16);
                    root.set_v1(r.v[1] as u16);
                    root.set_v2(r.v[2] as u16);
                    root.set_v3(r.v[3] as u16);
                    root.set_v4(r.v[4] as u16);
                    root.set_v5(r.v[5] as u16);
                    root.set_v6(r.v[6] as u16);
                    root.set_v7(r.v[7] as u16);
                    root.set_has_a(r.a.is_some());
                    root.set_a(r.a.unwrap_or(0));
                    root.set_has_b(r.b.is_some());
                    root.set_b(r.b.unwrap_or(0));
                }
                capnp::serialize::write_message(&mut self.buf, &message)?;
                Ok(&self.buf)
            }
            (Codec::Flatbuffers, Kind::Numeric) => {
                self.flat.reset();
                let values = flat::bench::Values::new(
                    r.v[0] as u16,
                    r.v[1] as u16,
                    r.v[2] as u16,
                    r.v[3] as u16,
                    r.v[4] as u16,
                    r.v[5] as u16,
                    r.v[6] as u16,
                    r.v[7] as u16,
                );
                let root = flat::bench::Numeric::create(
                    &mut self.flat,
                    &flat::bench::NumericArgs {
                        id: r.id,
                        values: Some(&values),
                        a: r.a,
                        b: r.b,
                    },
                );
                self.flat.finish(root, None);
                Ok(self.flat.finished_data())
            }
            (Codec::QslDefault, Kind::Mixed) => {
                self.buf.resize(8192, 0);
                let value = generated::MixedDefault {
                    id: r.id,
                    v0: r.v[0],
                    v1: r.v[1],
                    v2: r.v[2],
                    v3: r.v[3],
                    v4: r.v[4],
                    v5: r.v[5],
                    v6: r.v[6],
                    v7: r.v[7],
                    text: &r.text,
                    data: &r.data,
                    a: r.a,
                    b: r.b,
                };
                let used = value.serialize_checked_to(&mut self.buf)?;
                Ok(&self.buf[..used])
            }
            (Codec::QslOptimized, Kind::Mixed) => {
                self.buf.resize(8192, 0);
                let value = generated::MixedOptimized {
                    id: r.id,
                    v0: r.v[0] as u16,
                    v1: r.v[1] as u16,
                    v2: r.v[2] as u16,
                    v3: r.v[3] as u16,
                    v4: r.v[4] as u16,
                    v5: r.v[5] as u16,
                    v6: r.v[6] as u16,
                    v7: r.v[7] as u16,
                    text: &r.text,
                    data: &r.data,
                    a: r.a,
                    b: r.b,
                };
                let used = value.serialize_checked_to(&mut self.buf)?;
                Ok(&self.buf[..used])
            }
            (Codec::Protobuf, Kind::Mixed) => {
                self.buf.clear();
                f.mixed.encode(&mut self.buf)?;
                Ok(&self.buf)
            }
            (Codec::Bincode, Kind::Mixed) => {
                self.buf.resize(8192, 0);
                let value = Mix {
                    id: r.id,
                    v: r.v.map(|v| v as u16),
                    text: &r.text,
                    data: &r.data,
                    a: r.a,
                    b: r.b,
                };
                let used = bincode::encode_into_slice(
                    value,
                    &mut self.buf,
                    bincode::config::standard()
                        .with_fixed_int_encoding()
                        .with_limit::<65535>(),
                )?;
                Ok(&self.buf[..used])
            }
            (Codec::Capnp, Kind::Mixed) => {
                self.buf.clear();
                let mut message = capnp::message::Builder::new(&mut self.cap);
                {
                    let mut root = message.init_root::<records_capnp::mixed::Builder>();
                    root.set_id(r.id);
                    root.set_v0(r.v[0] as u16);
                    root.set_v1(r.v[1] as u16);
                    root.set_v2(r.v[2] as u16);
                    root.set_v3(r.v[3] as u16);
                    root.set_v4(r.v[4] as u16);
                    root.set_v5(r.v[5] as u16);
                    root.set_v6(r.v[6] as u16);
                    root.set_v7(r.v[7] as u16);
                    root.set_text(r.text.as_str());
                    root.set_data(&r.data);
                    root.set_has_a(r.a.is_some());
                    root.set_a(r.a.unwrap_or(0));
                    root.set_has_b(r.b.is_some());
                    root.set_b(r.b.unwrap_or(0));
                }
                capnp::serialize::write_message(&mut self.buf, &message)?;
                Ok(&self.buf)
            }
            (Codec::Flatbuffers, Kind::Mixed) => {
                self.flat.reset();
                let text = self.flat.create_string(&r.text);
                let data = self.flat.create_vector(&r.data);
                let values = flat::bench::Values::new(
                    r.v[0] as u16,
                    r.v[1] as u16,
                    r.v[2] as u16,
                    r.v[3] as u16,
                    r.v[4] as u16,
                    r.v[5] as u16,
                    r.v[6] as u16,
                    r.v[7] as u16,
                );
                let root = flat::bench::Mixed::create(
                    &mut self.flat,
                    &flat::bench::MixedArgs {
                        id: r.id,
                        values: Some(&values),
                        text: Some(text),
                        data: Some(data),
                        a: r.a,
                        b: r.b,
                    },
                );
                self.flat.finish(root, None);
                Ok(self.flat.finished_data())
            }
        }
    }
}

#[inline(always)]
pub fn read(codec: Codec, kind: Kind, input: &[u8]) -> Result<u64> {
    if input.len() > MAX_RECORD {
        return invalid();
    }
    let mut bounded = input;
    match (codec, kind) {
        (Codec::QslDefault, Kind::Numeric) => {
            let x = generated::NumericDefault::deserialize_checked(input)?;
            Ok(checksum(
                x.id,
                &[x.v0, x.v1, x.v2, x.v3, x.v4, x.v5, x.v6, x.v7],
                "",
                &[],
                x.a,
                x.b,
            ))
        }
        (Codec::QslOptimized, Kind::Numeric) => {
            let x = generated::NumericOptimized::deserialize_checked(input)?;
            Ok(checksum(
                x.id,
                &[x.v0, x.v1, x.v2, x.v3, x.v4, x.v5, x.v6, x.v7],
                "",
                &[],
                x.a,
                x.b,
            ))
        }
        (Codec::Protobuf, Kind::Numeric) => {
            let x = proto::Numeric::decode(input)?;
            let id = x.id;
            let values: &[u32; 8] = x
                .values
                .as_slice()
                .try_into()
                .map_err(|_| "numeric count")?;
            let v = values.map(u64::from);
            let a = x.a.map(u8::try_from).transpose()?;
            let b = x.b.map(u16::try_from).transpose()?;
            let text = "";
            let data = &[];
            validate_values(kind, id, &v, text, data)?;
            Ok(checksum(id, &v, text, data, a, b))
        }
        (Codec::Bincode, Kind::Numeric) => {
            let (x, used): (Num, usize) = bincode::borrow_decode_from_slice(
                input,
                bincode::config::standard()
                    .with_fixed_int_encoding()
                    .with_limit::<65535>(),
            )?;
            if used != input.len() {
                return invalid();
            }
            let id = x.id;
            let v = x.v.map(u64::from);
            let a = x.a;
            let b = x.b;
            let text = "";
            let data = &[];
            validate_values(kind, id, &v, text, data)?;
            Ok(checksum(id, &v, text, data, a, b))
        }
        (Codec::Capnp, Kind::Numeric) => {
            let message = capnp::serialize::read_message_from_flat_slice_no_alloc(
                &mut bounded,
                capnp::message::ReaderOptions {
                    traversal_limit_in_words: Some(8192),
                    nesting_limit: 16,
                },
            )?;
            if !bounded.is_empty() {
                return invalid();
            }
            let x = message.get_root::<records_capnp::numeric::Reader>()?;
            let id = x.get_id();
            let v = [
                u64::from(x.get_v0()),
                u64::from(x.get_v1()),
                u64::from(x.get_v2()),
                u64::from(x.get_v3()),
                u64::from(x.get_v4()),
                u64::from(x.get_v5()),
                u64::from(x.get_v6()),
                u64::from(x.get_v7()),
            ];
            let a = x.get_has_a().then_some(x.get_a());
            let b = x.get_has_b().then_some(x.get_b());
            let text = "";
            let data = &[];
            validate_values(kind, id, &v, text, data)?;
            Ok(checksum(id, &v, text, data, a, b))
        }
        (Codec::Flatbuffers, Kind::Numeric) => {
            let options = flatbuffers::VerifierOptions {
                max_depth: 16,
                max_tables: 16,
                max_apparent_size: MAX_RECORD,
                ignore_missing_null_terminator: false,
            };
            let x = flatbuffers::root_with_opts::<flat::bench::Numeric>(&options, input)?;
            let id = x.id();
            let v = [
                u64::from(x.values().v0()),
                u64::from(x.values().v1()),
                u64::from(x.values().v2()),
                u64::from(x.values().v3()),
                u64::from(x.values().v4()),
                u64::from(x.values().v5()),
                u64::from(x.values().v6()),
                u64::from(x.values().v7()),
            ];
            let a = x.a();
            let b = x.b();
            let text = "";
            let data = &[];
            validate_values(kind, id, &v, text, data)?;
            Ok(checksum(id, &v, text, data, a, b))
        }
        (Codec::QslDefault, Kind::Mixed) => {
            let x = generated::MixedDefault::deserialize_checked(input)?;
            Ok(checksum(
                x.id,
                &[x.v0, x.v1, x.v2, x.v3, x.v4, x.v5, x.v6, x.v7],
                x.text,
                x.data,
                x.a,
                x.b,
            ))
        }
        (Codec::QslOptimized, Kind::Mixed) => {
            let x = generated::MixedOptimized::deserialize_checked(input)?;
            Ok(checksum(
                x.id,
                &[
                    u64::from(x.v0),
                    u64::from(x.v1),
                    u64::from(x.v2),
                    u64::from(x.v3),
                    u64::from(x.v4),
                    u64::from(x.v5),
                    u64::from(x.v6),
                    u64::from(x.v7),
                ],
                x.text,
                x.data,
                x.a,
                x.b,
            ))
        }
        (Codec::Protobuf, Kind::Mixed) => {
            let x = proto::Mixed::decode(input)?;
            let id = x.id;
            let values: &[u32; 8] = x
                .values
                .as_slice()
                .try_into()
                .map_err(|_| "numeric count")?;
            let v = values.map(u64::from);
            let a = x.a.map(u8::try_from).transpose()?;
            let b = x.b.map(u16::try_from).transpose()?;
            let text = x.text.as_str();
            let data = x.data.as_slice();
            validate_values(kind, id, &v, text, data)?;
            Ok(checksum(id, &v, text, data, a, b))
        }
        (Codec::Bincode, Kind::Mixed) => {
            let (x, used): (Mix, usize) = bincode::borrow_decode_from_slice(
                input,
                bincode::config::standard()
                    .with_fixed_int_encoding()
                    .with_limit::<65535>(),
            )?;
            if used != input.len() {
                return invalid();
            }
            let id = x.id;
            let v = x.v.map(u64::from);
            let a = x.a;
            let b = x.b;
            let text = x.text;
            let data = x.data;
            validate_values(kind, id, &v, text, data)?;
            Ok(checksum(id, &v, text, data, a, b))
        }
        (Codec::Capnp, Kind::Mixed) => {
            let message = capnp::serialize::read_message_from_flat_slice_no_alloc(
                &mut bounded,
                capnp::message::ReaderOptions {
                    traversal_limit_in_words: Some(8192),
                    nesting_limit: 16,
                },
            )?;
            if !bounded.is_empty() {
                return invalid();
            }
            let x = message.get_root::<records_capnp::mixed::Reader>()?;
            let id = x.get_id();
            let v = [
                u64::from(x.get_v0()),
                u64::from(x.get_v1()),
                u64::from(x.get_v2()),
                u64::from(x.get_v3()),
                u64::from(x.get_v4()),
                u64::from(x.get_v5()),
                u64::from(x.get_v6()),
                u64::from(x.get_v7()),
            ];
            let a = x.get_has_a().then_some(x.get_a());
            let b = x.get_has_b().then_some(x.get_b());
            let text = x.get_text()?.to_str()?;
            let data = x.get_data()?;
            validate_values(kind, id, &v, text, data)?;
            Ok(checksum(id, &v, text, data, a, b))
        }
        (Codec::Flatbuffers, Kind::Mixed) => {
            let options = flatbuffers::VerifierOptions {
                max_depth: 16,
                max_tables: 16,
                max_apparent_size: MAX_RECORD,
                ignore_missing_null_terminator: false,
            };
            let x = flatbuffers::root_with_opts::<flat::bench::Mixed>(&options, input)?;
            let id = x.id();
            let v = [
                u64::from(x.values().v0()),
                u64::from(x.values().v1()),
                u64::from(x.values().v2()),
                u64::from(x.values().v3()),
                u64::from(x.values().v4()),
                u64::from(x.values().v5()),
                u64::from(x.values().v6()),
                u64::from(x.values().v7()),
            ];
            let a = x.a();
            let b = x.b();
            let text = x.text();
            let data = x.data().bytes();
            validate_values(kind, id, &v, text, data)?;
            Ok(checksum(id, &v, text, data, a, b))
        }
    }
}
pub mod metrics;
pub mod transport;

/// Owned initialized storage for prepared readers; padding is never transmitted.
pub struct Frame {
    pub storage: Vec<u8>,
    pub wire_len: usize,
}
impl Frame {
    pub fn new(bytes: &[u8]) -> Self {
        let wire_len = bytes.len();
        let mut storage = Vec::with_capacity(wire_len + 8);
        storage.extend_from_slice(bytes);
        storage.resize(wire_len + 8, 0);
        Self { storage, wire_len }
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.storage[..self.wire_len]
    }
}
#[inline(always)]
pub fn read_frame<const C: usize, const K: usize>(frame: &Frame) -> Result<u64> {
    if C == 1 && K == 0 {
        let frame = qsl::payload_quant::QuantizedFrame::<generated::NumericOptimized>::try_new(
            &frame.storage,
            frame.wire_len,
        )?;
        let mut out = std::mem::MaybeUninit::uninit();
        let x = frame.decode_into(&mut out);
        Ok(checksum(
            x.id,
            &[x.v0, x.v1, x.v2, x.v3, x.v4, x.v5, x.v6, x.v7],
            "",
            &[],
            x.a,
            x.b,
        ))
    } else {
        read(
            Codec::ALL[C],
            if K == 0 { Kind::Numeric } else { Kind::Mixed },
            frame.as_slice(),
        )
    }
}

/// Reuse native Prost-owned buffers. Decoding errors leave this private scratch
/// partially changed; it is cleared before reuse and never published on error.
#[derive(Default)]
pub struct Decoder {
    numeric: proto::Numeric,
    mixed: proto::Mixed,
}
impl Decoder {
    #[inline(always)]
    pub fn protobuf(&mut self, kind: Kind, input: &[u8]) -> Result<u64> {
        if input.len() > MAX_RECORD {
            return invalid();
        }
        match kind {
            Kind::Numeric => {
                self.numeric.clear();
                self.numeric.merge(input)?;
                let x = &self.numeric;
                let values: &[u32; 8] = x
                    .values
                    .as_slice()
                    .try_into()
                    .map_err(|_| "numeric count")?;
                let v = values.map(u64::from);
                validate_values(kind, x.id, &v, "", &[])?;
                Ok(checksum(
                    x.id,
                    &v,
                    "",
                    &[],
                    x.a.map(u8::try_from).transpose()?,
                    x.b.map(u16::try_from).transpose()?,
                ))
            }
            Kind::Mixed => {
                self.mixed.clear();
                self.mixed.merge(input)?;
                transport::mixed_proto_checksum(&self.mixed)
            }
        }
    }
}
