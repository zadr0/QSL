fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    qsl::build_qsl_files(&["schemas/layout.qsl"], &[] as &[&str], &out)?;
    Ok(())
}
