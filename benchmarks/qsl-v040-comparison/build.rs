fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    qsl::build_qsl_files(&["schemas/records.qsl"], &["schemas/settings.deq"], &out)?;
    let compiler = std::env::var_os("CAPNP").unwrap_or_else(|| "capnp".into());
    capnpc::CompilerCommand::new()
        .capnp_executable(compiler)
        .src_prefix("schemas")
        .file("schemas/records.capnp")
        .run()?;
    println!("cargo:rerun-if-changed=schemas/records.capnp");
    println!("cargo:rerun-if-env-changed=CAPNP");
    Ok(())
}
