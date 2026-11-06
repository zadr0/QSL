#![doc = include_str!("../README.md")]
pub mod compat;
pub mod deq;
pub mod interpreter;
pub mod io;
pub mod parser;
pub mod parser_util;
pub mod serialize;
pub mod strings;
pub mod types;

pub use compat::*;
pub use deq::{build_qsl_files, parse_deq, DeqConfig, DeqConstant};
pub use interpreter::*;
pub use io::*;
pub use parser::{parse_qsl, parse_qsl_with_constants, ParseError};
pub use serialize::*;
pub use strings::*;
pub use types::*;

pub mod hybrid;
pub mod modernization;
pub use modernization::{InlineArray, StackString};
pub mod optional;
pub use optional::OptionalOffsets;

pub mod socket;
pub use socket::RawSocket;

pub mod analysis;
pub mod bit_io;
pub mod payload_quant;
pub mod qps;
pub mod qps_bytes;
pub mod static_index;
pub use payload_quant::{QuantizedFrame, QuantizedLease, QuantizedOut, QuantizedRegions};
pub mod rt;
#[cfg(feature = "rust-macros")]
pub mod rust_macro;
pub mod schema_macro;
#[cfg(feature = "rust-macros")]
pub use deq::build_qsl_files_with_rust;
pub use qsl_macros_impl::{qsl_macro, schem_macro};
pub mod length_graph;
pub mod static_optional;
pub mod validation;
pub use analysis::*;
pub use bit_io::*;

/// Native Windows memory regions and direct blocking TCP.
#[cfg(all(feature = "native", windows, target_has_atomic = "64"))]
pub mod native;

pub mod schema_hash;
pub use schema_hash::{
    SchemaHashAlgorithm, SchemaHashKind, SchemaHashes, SchemaIdentity, SCHEMA_HASH_VERSION,
};
