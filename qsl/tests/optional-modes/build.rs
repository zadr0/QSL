fn main() {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let p = qsl::build_qsl_files(&["modes.qsl"], &[], &out).unwrap();
    std::fs::copy(p, out.join("qsl_generated.rs")).unwrap();
}
