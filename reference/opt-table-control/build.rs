fn main() -> Result<(), Box<dyn std::error::Error>> {
    qsl::build_qsl_files(
        &["schema.qsl"],
        &[],
        std::path::Path::new(&std::env::var_os("OUT_DIR").unwrap()),
    )?;
    Ok(())
}
