fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let normal = qsl::build_qsl_files_with_rust(&["schema.qsl"], &[], &["models.rs"], &out)?;
    std::fs::write(out.join("normal.rs"), std::fs::read(normal)?)?;
    let rt = qsl::build_qsl_files(&["rt.qsl"], &["rt.deq"], &out)?;
    std::fs::write(out.join("rt.rs"), std::fs::read(rt)?)?;
    Ok(())
}
