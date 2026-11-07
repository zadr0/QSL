#![doc = include_str!("../README.md")]
pub mod analysis;
pub mod bitmap;
pub mod builder;
pub mod compat;
pub mod config;
pub mod deq;
pub mod frontend;
pub mod hybrid;
pub mod io;
pub mod length_graph;
pub mod lexer;
pub mod modernization;
pub mod parser;
pub mod parser_util;
pub mod payload_quant;
mod primitive_macro;
pub mod regions;
#[cfg(feature = "rust-macros")]
pub mod rust_macro;
pub mod schema_hash;
pub mod schema_macro;
pub mod static_index;
pub mod static_optional;
pub mod types;
pub mod wire;
pub use analysis::*;
pub use builder::{build_qsl_schem, QslSchemaBuilder, SchemaBackend};
pub use compat::*;
pub use config::GenerationConfig;
pub use deq::{parse_deq, DeqConfig, DeqConstant};
pub use frontend::{analyze_schemas, AnalyzedModule};
pub use io::{QslError, QslResult, QslSizer};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{parse_qsl, parse_qsl_with_constants, ParseError};
pub use schema_hash::{
    SchemaHashAlgorithm, SchemaHashKind, SchemaHashes, SchemaIdentity, SCHEMA_HASH_VERSION,
};
pub use types::*;
pub use wire::*;
