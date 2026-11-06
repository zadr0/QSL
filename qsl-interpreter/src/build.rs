//! Compatible build-script wrappers around the shared schema builder.
use crate::{InterpreterBuilderExt, QslSchemaBuilder};
use std::path::{Path, PathBuf};

/// Load the declared inputs, choose Rust generation and write qsl_gen.rs.
pub fn build_qsl_files(
    qsl_files: &[&str],
    deq_files: &[&str],
    out_dir: impl AsRef<Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let builder = QslSchemaBuilder::from_files(qsl_files, deq_files)?;
    let path = out_dir.as_ref().join("qsl_gen.rs");
    std::fs::write(&path, builder.use_interpreter()?)?;
    Ok(path)
}
#[cfg(feature = "rust-macros")]
pub fn build_qsl_files_with_rust(
    qsl_files: &[&str],
    deq_files: &[&str],
    rust_files: &[&str],
    out_dir: impl AsRef<Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let builder = QslSchemaBuilder::from_files_with_rust(qsl_files, deq_files, rust_files)?;
    let path = out_dir.as_ref().join("qsl_gen.rs");
    std::fs::write(&path, builder.use_interpreter()?)?;
    Ok(path)
}
