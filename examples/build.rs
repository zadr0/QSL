fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let generated = qsl::build_qsl_files_with_rust(
        &[
            "schemas/01-basic.qsl",
            "schemas/02-strings.qsl",
            "schemas/03-optionals.qsl",
            "schemas/04-qps.qsl",
            "schemas/06-macros.qsl",
            "schemas/07-safety.qsl",
        ],
        &["schemas/config.deq"],
        &["models.rs"],
        &out,
    )?;
    std::fs::copy(&generated, out.join("normal.rs"))?;
    let generated = qsl::build_qsl_files(
        &["schemas/05-rt.qsl"],
        &["schemas/config.deq", "schemas/rt.deq"],
        &out,
    )?;
    std::fs::copy(&generated, out.join("rt.rs"))?;
    Ok(())
}
