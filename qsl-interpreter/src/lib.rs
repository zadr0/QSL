#![doc = include_str!("../README.md")]
pub use qsl_lexer::*;
mod backend;
mod build;
pub use backend::InterpreterBuilderExt;
pub mod frontend;
pub use frontend::{prepare_schemas, PreparedModule, PreparedSchema};
pub mod interpreter;
pub use build::build_qsl_files;
#[cfg(feature = "rust-macros")]
pub use build::build_qsl_files_with_rust;
pub use interpreter::*;
#[cfg(test)]
mod frontend_tests;

#[cfg(test)]
mod macro_emission_tests;

/// Convenient imports for the shared builder and Rust backend.
pub mod prelude {
    pub use crate::{build_qsl_schem, InterpreterBuilderExt, QslSchemaBuilder, SchemaBackend};
}

#[cfg(test)]
mod optional_policy_tests;
