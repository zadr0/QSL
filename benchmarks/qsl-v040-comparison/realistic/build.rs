fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    qsl::build_qsl_files(
        &["schemas/telemetry.qsl"],
        &["schemas/settings.deq"],
        &output,
    )?;
    capnpc::CompilerCommand::new()
        .capnp_executable(std::env::var_os("CAPNP").unwrap_or_else(|| "capnp".into()))
        .src_prefix("schemas")
        .file("schemas/telemetry.capnp")
        .run()?;
    println!("cargo:rerun-if-changed=schemas/telemetry.capnp");
    println!("cargo:rerun-if-env-changed=CAPNP");
    Ok(())
}
