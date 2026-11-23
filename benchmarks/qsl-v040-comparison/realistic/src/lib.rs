use prost::Message;
use std::fmt;

pub mod cap_codec;
pub mod fixture;
pub mod model;
mod qsl_codec;
pub mod transport;
pub mod telemetry_capnp {
    include!(concat!(env!("OUT_DIR"), "/telemetry_capnp.rs"));
}
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub const MAX_FRAME: usize = 32_768;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codec {
    Qsl,
    QslBitmap,
    Protobuf,
    Bincode,
    Capnp,
    CapnpPacked,
}
pub const CODECS: [Codec; 6] = [
    Codec::Qsl,
    Codec::QslBitmap,
    Codec::Protobuf,
    Codec::Bincode,
    Codec::Capnp,
    Codec::CapnpPacked,
];
impl Codec {
    pub fn name(self) -> &'static str {
        match self {
            Self::Qsl => "qsl",
            Self::QslBitmap => "qsl_bitmap",
            Self::Protobuf => "protobuf",
            Self::Bincode => "bincode_varint",
            Self::Capnp => "capnp",
            Self::CapnpPacked => "capnp_packed",
        }
    }
    pub fn encode(self, value: model::Envelope) -> Result<Vec<u8>> {
        if value.body.is_none() {
            return Err("missing body".into());
        }
        let bytes = match self {
            Self::Qsl | Self::QslBitmap => qsl_codec::encode(value, self == Self::QslBitmap)?,
            Self::Protobuf => value.encode_to_vec(),
            Self::Bincode => bincode::encode_to_vec(
                value,
                bincode::config::standard().with_limit::<MAX_FRAME>(),
            )?,
            Self::Capnp | Self::CapnpPacked => {
                cap_codec::encode(&value, self == Self::CapnpPacked)?
            }
        };
        if bytes.len() > MAX_FRAME {
            return Err("frame limit".into());
        }
        Ok(bytes)
    }
    pub fn decode(self, bytes: &[u8]) -> Result<model::Envelope> {
        if bytes.len() > MAX_FRAME {
            return Err("frame limit".into());
        }
        let value = match self {
            Self::Qsl | Self::QslBitmap => qsl_codec::decode(bytes, self == Self::QslBitmap)?,
            Self::Protobuf => model::Envelope::decode(bytes)?,
            Self::Bincode => {
                let (value, used) = bincode::decode_from_slice(
                    bytes,
                    bincode::config::standard().with_limit::<MAX_FRAME>(),
                )?;
                if used != bytes.len() {
                    return Err("trailing Bincode bytes".into());
                }
                value
            }
            Self::Capnp | Self::CapnpPacked => cap_codec::decode(bytes, self == Self::CapnpPacked)?,
        };
        if value.body.is_none() {
            return Err("missing body".into());
        }
        Ok(value)
    }
}
impl fmt::Display for Codec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
impl std::str::FromStr for Codec {
    type Err = &'static str;
    fn from_str(name: &str) -> std::result::Result<Self, Self::Err> {
        CODECS
            .into_iter()
            .find(|codec| codec.name() == name)
            .ok_or("unknown codec")
    }
}
pub fn normalize(mut envelope: model::Envelope) -> Result<model::Envelope> {
    use model::Body::*;
    match envelope.body.as_ref() {
        Some(Reading(_) | Update(_) | Notice(_)) => {}
        _ => return Err("API accepts reading, update or notice".into()),
    }
    envelope.revision = envelope
        .revision
        .checked_add(1)
        .ok_or("revision overflow")?;
    Ok(envelope)
}
