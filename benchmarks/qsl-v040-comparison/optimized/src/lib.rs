use model::Envelope;
pub mod cases;
pub mod fixed;
pub mod zero_copy;
use prost::Message as _;
use qsl_realistic_bench::telemetry_capnp;
pub use qsl_realistic_bench::{fixture, model, normalize, Result, MAX_FRAME};

mod flat {
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/flat_adapter.rs"));
}
pub mod small_capnp {
    include!(concat!(env!("OUT_DIR"), "/small_capnp.rs"));
}
#[allow(dead_code)]
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Auto,
    Bitmap,
    Compact,
    Bounded,
    Aligned,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codec {
    Qsl(Layout),
    Protobuf,
    Bincode,
    BincodeFixed,
    Capnp,
    CapnpPacked,
}
pub const CODECS: [Codec; 10] = [
    Codec::Qsl(Layout::Auto),
    Codec::Qsl(Layout::Bitmap),
    Codec::Qsl(Layout::Compact),
    Codec::Qsl(Layout::Bounded),
    Codec::Qsl(Layout::Aligned),
    Codec::Protobuf,
    Codec::Bincode,
    Codec::BincodeFixed,
    Codec::Capnp,
    Codec::CapnpPacked,
];

type CapMessage = capnp::message::Builder<capnp::message::HeapAllocator>;
mod cap_codec {
    #![allow(dead_code)]
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../realistic/src/cap_codec.rs"
    ));
    pub(super) fn prepare(value: &Envelope) -> super::CapMessage {
        let mut message = capnp::message::Builder::new_default();
        write_envelope(message.init_root::<schema::envelope::Builder<'_>>(), value);
        message
    }
}
enum Prepared {
    Flat(flat::Flat),
    Rust(Envelope),
    Capnp(CapMessage),
}
impl Codec {
    pub fn name(self) -> &'static str {
        match self {
            Self::Qsl(Layout::Auto) => "qsl_auto",
            Self::Qsl(Layout::Bitmap) => "qsl_auto_bitmap",
            Self::Qsl(Layout::Compact) => "qsl_auto_compact_bitmap",
            Self::Qsl(Layout::Bounded) => "qsl_auto_bounded_compact_bitmap",
            Self::Qsl(Layout::Aligned) => "qsl_auto_bounded_aligned_bitmap",
            Self::Protobuf => "protobuf",
            Self::Bincode => "bincode_varint",
            Self::BincodeFixed => "bincode_fixint",
            Self::Capnp => "capnp",
            Self::CapnpPacked => "capnp_packed",
        }
    }
    fn prepare(self, value: Envelope) -> Result<Prepared> {
        if value.body.is_none() {
            return Err("missing body".into());
        }
        Ok(match self {
            Self::Qsl(layout) => Prepared::Flat(flat::Flat::prepare(value, layout)?),
            Self::Protobuf | Self::Bincode | Self::BincodeFixed => Prepared::Rust(value),
            Self::Capnp | Self::CapnpPacked => Prepared::Capnp(cap_codec::prepare(&value)),
        })
    }
    fn encode(
        self,
        prepared: &Prepared,
        output: &mut [u8; MAX_FRAME],
        checked: bool,
    ) -> Result<usize> {
        match (self, prepared) {
            (Self::Qsl(_), Prepared::Flat(value)) => {
                // Unchecked calls are confined to Verified, constructed from a successful checked write.
                unsafe { value.write(output, checked) }
            }
            (Self::Protobuf, Prepared::Rust(value)) => {
                let mut dst = output.as_mut_slice();
                value.encode(&mut dst)?;
                Ok(MAX_FRAME - dst.len())
            }
            (Self::Bincode, Prepared::Rust(value)) => Ok(bincode::encode_into_slice(
                value,
                output,
                bincode::config::standard().with_limit::<MAX_FRAME>(),
            )?),
            (Self::BincodeFixed, Prepared::Rust(value)) => Ok(bincode::encode_into_slice(
                value,
                output,
                bincode::config::standard()
                    .with_fixed_int_encoding()
                    .with_limit::<MAX_FRAME>(),
            )?),
            (Self::Capnp | Self::CapnpPacked, Prepared::Capnp(value)) => {
                let mut dst = output.as_mut_slice();
                if self == Self::CapnpPacked {
                    capnp::serialize_packed::write_message(&mut dst, value)?;
                } else {
                    capnp::serialize::write_message(&mut dst, value)?;
                }
                Ok(MAX_FRAME - dst.len())
            }
            _ => Err("mismatched prepared representation".into()),
        }
    }
    pub fn decode_checked(self, input: &[u8]) -> Result<Envelope> {
        if input.len() > MAX_FRAME {
            return Err("frame limit".into());
        }
        let value = match self {
            Self::Qsl(layout) => unsafe { flat::read(input, layout, true)? },
            Self::Protobuf => Envelope::decode(input)?,
            Self::Bincode => {
                let (value, used) = bincode::decode_from_slice(
                    input,
                    bincode::config::standard().with_limit::<MAX_FRAME>(),
                )?;
                if input.len() != used {
                    return Err("trailing bytes".into());
                }
                value
            }
            Self::BincodeFixed => {
                let (value, used) = bincode::decode_from_slice(
                    input,
                    bincode::config::standard()
                        .with_fixed_int_encoding()
                        .with_limit::<MAX_FRAME>(),
                )?;
                if used != input.len() {
                    return Err("trailing bytes".into());
                }
                value
            }
            Self::Capnp | Self::CapnpPacked => cap_codec::decode(input, self == Self::CapnpPacked)?,
        };
        if value.body.is_none() {
            return Err("missing body".into());
        }
        Ok(value)
    }
    pub fn encode_checked_into(
        self,
        value: Envelope,
        output: &mut [u8; MAX_FRAME],
    ) -> Result<usize> {
        self.encode(&self.prepare(value)?, output, true)
    }
    pub fn encode_checked(self, value: Envelope) -> Result<Vec<u8>> {
        let mut buffer = [0u8; MAX_FRAME];
        let used = self.encode_checked_into(value, &mut buffer)?;
        Ok(buffer[..used].to_vec())
    }
}
impl std::fmt::Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
impl std::str::FromStr for Codec {
    type Err = &'static str;
    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        CODECS
            .into_iter()
            .find(|codec| codec.name() == value)
            .ok_or("unknown codec")
    }
}

/// Owns one prepared value and its immutable, checked and round-tripped frame.
pub struct Verified {
    codec: Codec,
    prepared: Prepared,
    frame: Vec<u8>,
}
impl Verified {
    pub fn new(codec: Codec, value: &Envelope, buffer: &mut [u8; MAX_FRAME]) -> Result<Self> {
        let prepared = codec.prepare(value.clone())?;
        let used = codec.encode(&prepared, buffer, true)?;
        let frame = buffer[..used].to_vec();
        if codec.decode_checked(&frame)? != *value {
            return Err("checked value differs".into());
        }
        let checked = Self {
            codec,
            prepared,
            frame,
        };
        if matches!(codec, Codec::Qsl(_)) {
            let trusted = checked.write(buffer, true)?;
            if &buffer[..trusted] != checked.frame() || checked.read(true)? != *value {
                return Err("trusted and checked must match for prevalidated frames".into());
            }
        }
        Ok(checked)
    }
    pub fn frame(&self) -> &[u8] {
        &self.frame
    }
    pub fn allocate(&self, trusted: bool) -> Result<Vec<u8>> {
        match (&self.prepared, self.codec) {
            (Prepared::Flat(value), Codec::Qsl(_)) => unsafe { value.allocate(!trusted) },
            (Prepared::Rust(value), Codec::Protobuf) => Ok(value.encode_to_vec()),
            (Prepared::Rust(value), Codec::Bincode) => Ok(bincode::encode_to_vec(
                value,
                bincode::config::standard().with_limit::<MAX_FRAME>(),
            )?),
            (Prepared::Rust(value), Codec::BincodeFixed) => Ok(bincode::encode_to_vec(
                value,
                bincode::config::standard()
                    .with_fixed_int_encoding()
                    .with_limit::<MAX_FRAME>(),
            )?),
            (Prepared::Capnp(value), Codec::Capnp) => {
                Ok(capnp::serialize::write_message_to_words(value))
            }
            (Prepared::Capnp(value), Codec::CapnpPacked) => {
                let mut output = Vec::new();
                capnp::serialize_packed::write_message(&mut output, value)?;
                Ok(output)
            }
            _ => Err("mismatched prepared representation".into()),
        }
    }
    pub fn write(&self, buffer: &mut [u8; MAX_FRAME], trusted: bool) -> Result<usize> {
        self.codec.encode(&self.prepared, buffer, !trusted)
    }
    pub fn read(&self, trusted: bool) -> Result<Envelope> {
        if let (Codec::Qsl(layout), true) = (self.codec, trusted) {
            // frame is immutable, exact and verified by the checked decoder in Self::new.
            unsafe { flat::read(&self.frame, layout, false) }
        } else {
            self.codec.decode_checked(&self.frame)
        }
    }
    pub fn native_size(&self) -> Option<usize> {
        match &self.prepared {
            Prepared::Flat(value) => Some(value.size()),
            _ => None,
        }
    }
}
