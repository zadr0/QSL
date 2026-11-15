#![no_std]
#![doc=include_str!("../README.md")]
#[cfg(feature = "compiler")]
extern crate std;
#[cfg(feature = "alloc")]
pub use qsl_core::alloc;
pub use qsl_core::payload_quant;
#[cfg(feature = "alloc")]
pub use qsl_core::payload_quant::QuantizedOut;
pub use qsl_core::payload_quant::{QuantizedFrame, QuantizedRecord};
pub use qsl_core::socket::{ByteTransport, RawSocket, SocketError};
pub use qsl_core::storage as modernization;
#[cfg(feature = "alloc")]
pub use qsl_core::SerOut;
pub use qsl_core::{
    bit_io, buffer, dynamic, io, length_graph, optional, serialize, socket, storage, strings,
    validation,
};
pub use qsl_core::{bit_io::*, io::*, optional::OptionalOffsets, strings::*, types::*, wire::*};
pub use qsl_core::{qsl_native_primitive, qsl_primitive};
pub use qsl_core::{
    BorrowSerializable, DeserializeDestination, InlineArray, InlineString, PrimitiveInteger,
    PrimitiveRepr, QslCustom, QslError, QslPrimitive, QslResult, QslSizer, SchemaHashAlgorithm,
    SchemaHashKind, SchemaHashes, SchemaIdentity, Serializable, SliceOut, StackString,
    StringHeader, SCHEMA_HASH_VERSION,
};
pub use qsl_rt as rt;
pub type QuantizedRegions<T, const CAP: usize, const N: usize> =
    qsl_core::payload_quant::QuantizedRegions<T, qsl_rt::StaticRegions<CAP, N>>;
pub type QuantizedLease<T, const CAP: usize, const N: usize> =
    qsl_core::payload_quant::QuantizedLease<T, qsl_rt::RegionLease<'static, CAP, N>>;
pub use qsl_core::payload_quant::QPS_CODEC_REVISION;
pub use qsl_core::qps_bytes;
pub use qsl_core::qps_bytes as qps;
#[cfg(not(feature = "compiler"))]
pub use qsl_core::{domain, optimizer, schema_hash, types};
#[cfg(feature = "compiler")]
pub use qsl_interpreter::build_qsl_files;
#[cfg(feature = "rust-macros")]
pub use qsl_interpreter::build_qsl_files_with_rust;
#[cfg(feature = "compiler")]
pub use qsl_interpreter::interpreter::*;
#[cfg(feature = "compiler")]
pub use qsl_interpreter::{
    interpreter, prepare_schemas, InterpreterBuilderExt, PreparedModule, PreparedSchema,
};
#[cfg(feature = "compiler")]
pub use qsl_lexer::deq;
#[cfg(feature = "compiler")]
pub use qsl_lexer::deq::{parse_deq, DeqConfig, DeqConstant};
#[cfg(feature = "rust-macros")]
pub use qsl_lexer::rust_macro;
#[cfg(feature = "compiler")]
pub use qsl_lexer::{
    analysis, builder, compat, config, domain, hybrid, layout, lexer, optimizer, parser,
    parser_util, regions, schema_hash, schema_macro, static_index, static_optional, types,
};
#[cfg(feature = "compiler")]
pub use qsl_lexer::{
    analysis::*,
    compat::*,
    config::GenerationConfig,
    frontend::{analyze_schemas, AnalyzedModule},
    lexer::{Lexer, Token, TokenKind},
    parser::{parse_qsl, parse_qsl_with_constants, ParseError},
};
#[cfg(feature = "compiler")]
pub use qsl_lexer::{
    build_qsl_schem, parse_qsl_with_wide_constants, DeqBuilder, QslBuilder, QslFieldType, QslMacro,
    QslSchemBuilder, QslSchemaBuilder, SchemaBackend, WideNumber,
};
#[cfg(feature = "macros")]
pub use qsl_macros_impl::{qsl_macro, schem_macro};
#[cfg(feature = "compiler")]
pub mod prelude {
    pub use crate::{
        build_qsl_schem, DeqBuilder, InterpreterBuilderExt, QslBuilder, QslSchemaBuilder,
        SchemaBackend,
    };
}
#[cfg(feature = "compiler")]
pub use qsl_interpreter::frontend;

pub use qsl_core::with_alloc;
#[cfg(feature = "compiler")]
pub use qsl_lexer::types::*;

pub use qsl_core::{FramePool, FrameStorage, PreparedFramePool};

#[cfg(feature = "compiler")]
pub use qsl_lexer::payload_quant::{PayloadQuantPlan, QuantField};

pub use qsl_core::{OutputBuffer, SerializeOutput, StackOut};
