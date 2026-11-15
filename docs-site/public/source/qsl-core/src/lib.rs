#![no_std]
#![doc = include_str!("../README.md")]
#[cfg(feature = "alloc")]
pub extern crate alloc;
#[cfg(test)]
extern crate std;
pub mod bit_io;
pub mod bitmap;
pub mod buffer;
pub mod domain;
pub mod dynamic;
pub mod io;
pub mod length_graph;
pub mod optimizer;
pub mod optional;
pub mod payload_quant;
pub mod primitive;
mod primitive_macro;
pub mod qps_bytes;
pub mod schema_hash;
pub mod serialize;
pub mod socket;
pub mod storage;
pub mod strings;
pub mod types;
pub mod validation;
pub mod wire;
pub use bit_io::*;
pub use io::*;
pub use optional::OptionalOffsets;
pub use schema_hash::{
    SchemaHashAlgorithm, SchemaHashKind, SchemaHashes, SchemaIdentity, SCHEMA_HASH_VERSION,
};
pub use serialize::*;
pub use storage::{InlineArray, StackString};
pub use strings::*;
pub use types::*;
pub use wire::*;

pub use primitive::{PrimitiveRepr, QslPrimitive};

pub use buffer::{FramePool, FrameStorage, PreparedFramePool};

#[cfg(feature = "alloc")]
pub use payload_quant::QuantizedOut;
pub use payload_quant::{CheckedQuantizedRecord, QuantizedFrame, QuantizedRecord};

pub use socket::{ByteTransport, RawSocket, SocketError};

/// Include allocator-backed conveniences only when the runtime enables alloc.
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! with_alloc {($($item:tt)*)=>{$($item)*};}
/// Omit allocator-backed conveniences in caller-buffer-only builds.
#[cfg(not(feature = "alloc"))]
#[macro_export]
macro_rules! with_alloc {
    ($($item:tt)*) => {};
}
