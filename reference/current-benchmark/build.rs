fn main() -> Result<(), Box<dyn std::error::Error>> {
    qsl::build_qsl_files_with_rust(
        &["schema.qsl"],
        &[],
        &["models.rs"],
        std::path::Path::new(&std::env::var_os("OUT_DIR").unwrap()),
    )?;
    Ok(())
}
