#![doc = include_str!("../README.md")]
pub use qsl_interpreter::interpreter;
pub use qsl_interpreter::InterpreterBuilderExt;
pub use qsl_lexer::{build_qsl_schem, QslSchemaBuilder, SchemaBackend};
pub mod prelude {
    pub use crate::{build_qsl_schem, InterpreterBuilderExt, QslSchemaBuilder, SchemaBackend};
}
pub mod frontend {
    pub use qsl_interpreter::frontend::{prepare_schemas, PreparedModule, PreparedSchema};
    pub use qsl_lexer::frontend::*;
}
pub use qsl_interpreter::{prepare_schemas, PreparedModule, PreparedSchema};
#[cfg(feature = "rust-macros")]
pub use qsl_lexer::rust_macro;
pub use qsl_lexer::{
    analysis, builder, compat, config, hybrid, length_graph, lexer, parser, parser_util,
    schema_hash, schema_macro, static_index, static_optional, types,
};
pub mod bit_io;
pub mod deq;
pub mod io;
pub mod modernization;
pub mod optional;
pub mod payload_quant;
pub mod qps;
pub mod qps_bytes;
pub mod rt;
pub mod serialize;
pub mod socket;
pub mod strings;
pub mod validation;
pub use bit_io::*;
#[cfg(feature = "rust-macros")]
pub use deq::build_qsl_files_with_rust;
pub use deq::{build_qsl_files, parse_deq, DeqConfig, DeqConstant};
pub use interpreter::*;
pub use io::*;
pub use modernization::{InlineArray, StackString};
pub use optional::OptionalOffsets;
pub use payload_quant::{QuantizedFrame, QuantizedLease, QuantizedOut, QuantizedRegions};
pub use qsl_lexer::{
    analysis::*,
    compat::*,
    config::GenerationConfig,
    frontend::{analyze_schemas, AnalyzedModule},
    lexer::{Lexer, Token, TokenKind},
    parser::{parse_qsl, parse_qsl_with_constants, ParseError},
    schema_hash::{
        SchemaHashAlgorithm, SchemaHashKind, SchemaHashes, SchemaIdentity, SCHEMA_HASH_VERSION,
    },
    types::*,
};
pub use qsl_lexer::{qsl_native_primitive, qsl_primitive};
pub use qsl_macros_impl::{qsl_macro, schem_macro};
pub use serialize::*;
pub use socket::RawSocket;
pub use strings::*;
