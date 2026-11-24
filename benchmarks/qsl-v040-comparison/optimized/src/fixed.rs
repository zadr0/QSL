use crate::{
    generated::{FixedAuto, FixedCompact, FixedGroup, FixedPoint},
    small_capnp::{fixed_record, fixed_u16},
    Result, MAX_FRAME,
};
use prost::Message;
use qsl::Serializable;

#[derive(Clone, PartialEq, Message)]
struct ProtoVar {
    #[prost(uint32, tag = "1")]
    f0: u32,
    #[prost(uint32, tag = "2")]
    f1: u32,
    #[prost(uint32, tag = "3")]
    f2: u32,
    #[prost(uint32, tag = "4")]
    f3: u32,
}
#[derive(Clone, PartialEq, Message)]
struct ProtoFixed {
    #[prost(fixed32, tag = "1")]
    f0: u32,
    #[prost(fixed32, tag = "2")]
    f1: u32,
    #[prost(fixed32, tag = "3")]
    f2: u32,
    #[prost(fixed32, tag = "4")]
    f3: u32,
}
pub const PROFILES: [&str; 12] = [
    "qsl_auto",
    "qsl_auto_pointing",
    "qsl_auto_compact",
    "qsl_auto_compact_group",
    "protobuf",
    "protobuf_fixed",
    "bincode_varint",
    "bincode_fixint",
    "bincode_u16_fixint",
    "capnp",
    "capnp_packed",
    "capnp_u16",
];
enum Prepared {
    Auto(FixedAuto),
    Point(FixedPoint),
    Compact(FixedCompact),
    Group(FixedGroup),
    Proto(ProtoVar),
    ProtoFixed(ProtoFixed),
    Bin([u32; 4]),
    Narrow([u16; 4]),
    Capnp(capnp::message::Builder<capnp::message::HeapAllocator>),
}
pub struct Fixed {
    profile: &'static str,
    prepared: Prepared,
    frame: Vec<u8>,
}

impl Fixed {
    pub fn new(
        profile: &'static str,
        values: [u32; 4],
        dest: &mut [u8; MAX_FRAME],
    ) -> Result<Self> {
        assert!(values.iter().all(|&v| v <= 4095));
        let [f0, f1, f2, f3] = values;
        let prepared = match profile {
            "qsl_auto" => Prepared::Auto(FixedAuto { f0, f1, f2, f3 }),
            "qsl_auto_pointing" => Prepared::Point(FixedPoint { f0, f1, f2, f3 }),
            "qsl_auto_compact" => Prepared::Compact(FixedCompact { f0, f1, f2, f3 }),
            "qsl_auto_compact_group" => Prepared::Group(FixedGroup { f0, f1, f2, f3 }),
            "protobuf" => Prepared::Proto(ProtoVar { f0, f1, f2, f3 }),
            "protobuf_fixed" => Prepared::ProtoFixed(ProtoFixed { f0, f1, f2, f3 }),
            "bincode_varint" | "bincode_fixint" => Prepared::Bin(values),
            "bincode_u16_fixint" => Prepared::Narrow(values.map(|value| value as u16)),
            "capnp" | "capnp_packed" => {
                let mut message = capnp::message::Builder::new_default();
                let mut record = message.init_root::<fixed_record::Builder<'_>>();
                record.set_f0(f0);
                record.set_f1(f1);
                record.set_f2(f2);
                record.set_f3(f3);
                Prepared::Capnp(message)
            }
            "capnp_u16" => {
                let mut message = capnp::message::Builder::new_default();
                let mut record = message.init_root::<fixed_u16::Builder<'_>>();
                record.set_f0(f0 as u16);
                record.set_f1(f1 as u16);
                record.set_f2(f2 as u16);
                record.set_f3(f3 as u16);
                Prepared::Capnp(message)
            }
            _ => return Err("unknown fixed profile".into()),
        };
        let mut item = Self {
            profile,
            prepared,
            frame: Vec::new(),
        };
        let n = item.write(dest, false)?;
        item.frame = dest[..n].to_vec();
        assert_eq!(item.read(false)?, values);
        if profile.starts_with("qsl") {
            let n = item.write(dest, true)?;
            assert_eq!(&dest[..n], item.frame());
            assert_eq!(item.read(true)?, values);
            assert_eq!(item.allocate(true)?, item.frame);
        }
        assert_eq!(item.allocate(false)?, item.frame);
        Ok(item)
    }
    pub fn frame(&self) -> &[u8] {
        &self.frame
    }
    pub fn write(&self, target: &mut [u8; MAX_FRAME], trusted: bool) -> Result<usize> {
        // Values were checked against the fixed domain in new. The output is
        // a disjoint 32 KiB buffer for records of at most 16 bytes in QSL.
        macro_rules! put {
            ($v:expr) => {
                if trusted {
                    unsafe { $v.serialize_into_unsafe(target.as_mut_ptr()) }
                } else {
                    $v.serialize_checked_to(target)?
                }
            };
        }
        Ok(match &self.prepared {
            Prepared::Auto(v) => put!(v),
            Prepared::Point(v) => put!(v),
            Prepared::Compact(v) => put!(v),
            Prepared::Group(v) => put!(v),
            Prepared::Proto(v) => {
                let mut dst = target.as_mut_slice();
                v.encode(&mut dst)?;
                MAX_FRAME - dst.len()
            }
            Prepared::ProtoFixed(v) => {
                let mut dst = target.as_mut_slice();
                v.encode(&mut dst)?;
                MAX_FRAME - dst.len()
            }
            Prepared::Bin(v) => {
                if self.profile == "bincode_fixint" {
                    bincode::encode_into_slice(
                        v,
                        target,
                        bincode::config::standard().with_fixed_int_encoding(),
                    )?
                } else {
                    bincode::encode_into_slice(v, target, bincode::config::standard())?
                }
            }
            Prepared::Narrow(v) => bincode::encode_into_slice(
                v,
                target,
                bincode::config::standard().with_fixed_int_encoding(),
            )?,
            Prepared::Capnp(v) => {
                let mut dst = target.as_mut_slice();
                if self.profile == "capnp_packed" {
                    capnp::serialize_packed::write_message(&mut dst, v)?;
                } else {
                    capnp::serialize::write_message(&mut dst, v)?;
                }
                MAX_FRAME - dst.len()
            }
        })
    }
    pub fn allocate(&self, trusted: bool) -> Result<Vec<u8>> {
        macro_rules! pack{($v:expr)=>{{
            if trusted {
                let mut output:Vec<u8>=Vec::with_capacity($v.serialized_size());
                // The value is verified by new and capacity matches the generated writer's requirement.
                unsafe{ let n=$v.serialize_into_unsafe(output.as_mut_ptr());output.set_len(n); }
                output
            }else{
                let mut output=vec![0;$v.serialized_size()];
                let n=$v.serialize_checked_to(&mut output)?;
                output.truncate(n);
                output
            }
        }}}
        Ok(match &self.prepared {
            Prepared::Auto(v) => pack!(v),
            Prepared::Point(v) => pack!(v),
            Prepared::Compact(v) => pack!(v),
            Prepared::Group(v) => pack!(v),
            Prepared::Proto(v) => v.encode_to_vec(),
            Prepared::ProtoFixed(v) => v.encode_to_vec(),
            Prepared::Bin(v) => {
                if self.profile == "bincode_fixint" {
                    bincode::encode_to_vec(
                        v,
                        bincode::config::standard().with_fixed_int_encoding(),
                    )?
                } else {
                    bincode::encode_to_vec(v, bincode::config::standard())?
                }
            }
            Prepared::Narrow(v) => {
                bincode::encode_to_vec(v, bincode::config::standard().with_fixed_int_encoding())?
            }
            Prepared::Capnp(v) => {
                if self.profile == "capnp_packed" {
                    let mut output = Vec::new();
                    capnp::serialize_packed::write_message(&mut output, v)?;
                    output
                } else {
                    capnp::serialize::write_message_to_words(v)
                }
            }
        })
    }
    pub fn read(&self, trusted: bool) -> Result<[u32; 4]> {
        let frame = self.frame();
        macro_rules! fields {
            ($v:expr) => {{
                let v = $v;
                [v.f0, v.f1, v.f2, v.f3]
            }};
        }
        macro_rules! get {
            ($t:ident) => {
                // Only the exact immutable frame established by new reaches the trusted reader.
                fields!(if trusted {
                    unsafe { $t::deserialize_from_raw_parts(frame.as_ptr(), frame.len()) }
                } else {
                    $t::deserialize_checked(frame)?
                })
            };
        }
        Ok(match &self.prepared {
            Prepared::Auto(_) => get!(FixedAuto),
            Prepared::Point(_) => get!(FixedPoint),
            Prepared::Compact(_) => get!(FixedCompact),
            Prepared::Group(_) => get!(FixedGroup),
            Prepared::Proto(_) => fields!(ProtoVar::decode(frame)?),
            Prepared::ProtoFixed(_) => fields!(ProtoFixed::decode(frame)?),
            Prepared::Bin(_) => {
                if self.profile == "bincode_fixint" {
                    bincode::decode_from_slice(
                        frame,
                        bincode::config::standard().with_fixed_int_encoding(),
                    )?
                    .0
                } else {
                    bincode::decode_from_slice(frame, bincode::config::standard())?.0
                }
            }
            Prepared::Narrow(_) => {
                let (values, _): ([u16; 4], _) = bincode::decode_from_slice(
                    frame,
                    bincode::config::standard().with_fixed_int_encoding(),
                )?;
                values.map(u32::from)
            }
            Prepared::Capnp(_) => {
                let options = capnp::message::ReaderOptions::new();
                let mut frame = frame;
                if self.profile == "capnp_packed" {
                    let msg = capnp::serialize_packed::read_message(&mut frame, options)?;
                    read_capnp(msg.get_root::<fixed_record::Reader<'_>>()?)
                } else {
                    let msg = capnp::serialize::read_message_from_flat_slice_no_alloc(
                        &mut frame, options,
                    )?;
                    if self.profile == "capnp_u16" {
                        let r = msg.get_root::<fixed_u16::Reader<'_>>()?;
                        [r.get_f0(), r.get_f1(), r.get_f2(), r.get_f3()].map(u32::from)
                    } else {
                        read_capnp(msg.get_root::<fixed_record::Reader<'_>>()?)
                    }
                }
            }
        })
    }
}
fn read_capnp(r: fixed_record::Reader<'_>) -> [u32; 4] {
    [r.get_f0(), r.get_f1(), r.get_f2(), r.get_f3()]
}
pub fn values(index: usize) -> [u32; 4] {
    let mut state = (index as u32 + 89).wrapping_mul(0x9e37_79b9);
    std::array::from_fn(|_| {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state & 4095
    })
}
