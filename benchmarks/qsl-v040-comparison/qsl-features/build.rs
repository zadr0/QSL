fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    qsl::build_qsl_files(
        &["schemas/features.qsl"],
        &["schemas/settings.deq"],
        &output,
    )?;
    Ok(())
}
