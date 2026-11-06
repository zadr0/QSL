//! Compatibility re-exports for DEQ parsing and build-script loading.
pub use qsl_interpreter::build_qsl_files;
#[cfg(feature = "rust-macros")]
pub use qsl_interpreter::build_qsl_files_with_rust;
pub use qsl_lexer::deq::*;
