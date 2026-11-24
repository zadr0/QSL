use crate::{
    generated::{BlobAligned, BlobAuto},
    small_capnp::borrowed_text,
    Result, MAX_FRAME,
};
use prost::{bytes::Bytes, Message};
use qsl::BorrowSerializable;

#[derive(Clone, PartialEq, Message)]
struct ProtoRef {
    #[prost(uint32, tag = "1")]
    id: u32,
    #[prost(bytes = "bytes", tag = "2")]
    text: Bytes,
}
#[derive(bincode::Encode, bincode::BorrowDecode)]
struct BinRef<'a> {
    id: u32,
    text: &'a str,
}
pub const PROFILES: [&str; 6] = [
    "qsl_auto_lifetime",
    "qsl_auto_lifetime_aligned",
    "protobuf",
    "bincode_varint",
    "bincode_fixint",
    "capnp",
];
pub struct Borrowed {
    profile: &'static str,
    frame: Bytes,
}
fn within(frame: &[u8], text: &str) -> bool {
    let start = frame.as_ptr() as usize;
    let ptr = text.as_ptr() as usize;
    ptr >= start
        && ptr
            .checked_add(text.len())
            .is_some_and(|end| end <= start + frame.len())
}
impl Borrowed {
    pub fn new(
        profile: &'static str,
        id: u32,
        text: &str,
        dst: &mut [u8; MAX_FRAME],
    ) -> Result<Self> {
        let frame = match profile {
            "qsl_auto_lifetime" => {
                let n = BlobAuto { id, text }.serialize_checked_to(dst)?;
                Vec::from(&dst[..n])
            }
            "qsl_auto_lifetime_aligned" => {
                let n = BlobAligned { id, text }.serialize_checked_to(dst)?;
                Vec::from(&dst[..n])
            }
            "protobuf" => ProtoRef {
                id,
                text: Bytes::copy_from_slice(text.as_bytes()),
            }
            .encode_to_vec(),
            "bincode_varint" => {
                bincode::encode_to_vec(BinRef { id, text }, bincode::config::standard())?
            }
            "bincode_fixint" => bincode::encode_to_vec(
                BinRef { id, text },
                bincode::config::standard().with_fixed_int_encoding(),
            )?,
            "capnp" => {
                let mut msg = capnp::message::Builder::new_default();
                let mut record = msg.init_root::<borrowed_text::Builder<'_>>();
                record.set_id(id);
                record.set_text(text);
                capnp::serialize::write_message_to_words(&msg)
            }
            _ => return Err("unknown borrowed profile".into()),
        };
        let sample = Self {
            profile,
            frame: Bytes::from(frame),
        };
        sample.with(false, |actual, view| {
            assert_eq!(actual, id);
            assert_eq!(view, text);
            assert!(within(&sample.frame, view));
        })?;
        if profile.starts_with("qsl") {
            sample.with(true, |actual, view| {
                assert_eq!(actual, id);
                assert_eq!(view, text);
                assert!(within(&sample.frame, view));
            })?;
        }
        Ok(sample)
    }
    pub fn len(&self) -> usize {
        self.frame.len()
    }
    pub fn with<T>(&self, trusted: bool, visit: impl FnOnce(u32, &str) -> T) -> Result<T> {
        let bytes = &self.frame[..];
        Ok(match self.profile {
            "qsl_auto_lifetime" => {
                // The trusted decoder sees an immutable frame that passed Self::new.
                let v = if trusted {
                    unsafe { BlobAuto::deserialize_borrowed_unsafe(bytes.as_ptr()) }
                } else {
                    BlobAuto::deserialize_checked(bytes)?
                };
                visit(v.id, v.text)
            }
            "qsl_auto_lifetime_aligned" => {
                let v = if trusted {
                    unsafe { BlobAligned::deserialize_borrowed_unsafe(bytes.as_ptr()) }
                } else {
                    BlobAligned::deserialize_checked(bytes)?
                };
                visit(v.id, v.text)
            }
            "protobuf" => {
                let v = ProtoRef::decode(self.frame.clone())?;
                visit(v.id, std::str::from_utf8(&v.text)?)
            }
            "bincode_varint" => {
                let (v, used): (BinRef<'_>, _) =
                    bincode::borrow_decode_from_slice(bytes, bincode::config::standard())?;
                if used != bytes.len() {
                    return Err("trailing bytes".into());
                }
                visit(v.id, v.text)
            }
            "bincode_fixint" => {
                let (v, used): (BinRef<'_>, _) = bincode::borrow_decode_from_slice(
                    bytes,
                    bincode::config::standard().with_fixed_int_encoding(),
                )?;
                if used != bytes.len() {
                    return Err("trailing bytes".into());
                }
                visit(v.id, v.text)
            }
            "capnp" => {
                let mut rest = bytes;
                let msg = capnp::serialize::read_message_from_flat_slice_no_alloc(
                    &mut rest,
                    capnp::message::ReaderOptions::new(),
                )?;
                if !rest.is_empty() {
                    return Err("trailing bytes".into());
                }
                let r = msg.get_root::<borrowed_text::Reader<'_>>()?;
                visit(r.get_id(), r.get_text()?.to_str()?)
            }
            _ => return Err("unknown borrowed profile".into()),
        })
    }
}
pub fn text(index: usize, length: usize) -> String {
    let mut output = format!("sensor {index:07} · ");
    let mut state = (index as u32 + 1).wrapping_mul(0x9e37_79b9);
    let alphabet = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    while output.len() < length {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        output.push(alphabet[(state as usize) % alphabet.len()] as char);
    }
    output
}
pub fn sample(id: u32, text: &str) -> u64 {
    let b = text.as_bytes();
    u64::from(id)
        ^ (b.len() as u64).rotate_left(24)
        ^ (u64::from(b[0]) << 48)
        ^ (u64::from(b[b.len() / 2]) << 40)
        ^ (u64::from(b[b.len() - 1]) << 56)
}
