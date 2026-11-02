fn main() {let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());qsl::build_qsl_files(&["lab.qsl"],&[],&out).unwrap();}
