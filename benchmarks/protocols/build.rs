fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let p = qsl::build_qsl_files(&["schemas/records.qsl"], &["schemas/config.deq"], &out)?;
    std::fs::copy(p, out.join("qsl_records.rs"))?;
    let compiler = std::env::var_os("CAPNP")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "tools/capnp/capnproto-tools-win32-1.5.0/capnp.exe".into());
    capnpc::CompilerCommand::new()
        .capnp_executable(compiler)
        .src_prefix("schemas")
        .file("schemas/records.capnp")
        .run()?;
    let flatc = std::env::var_os("FLATC").unwrap_or_else(|| "tools/flatc/flatc.exe".into());
    let status = std::process::Command::new(flatc)
        .args(["--rust", "-o"])
        .arg(&out)
        .arg("schemas/records.fbs")
        .status()?;
    if !status.success() {
        return Err("flatc failed".into());
    }
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", protoc);
    tonic_prost_build::configure().compile_protos(&["schemas/records.proto"], &["schemas"])?;
    for s in ["records.capnp", "records.fbs", "records.proto"] {
        println!("cargo:rerun-if-changed=schemas/{s}");
    }
    Ok(())
}
