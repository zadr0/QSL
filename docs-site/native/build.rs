use qsl::InterpreterBuilderExt;
fn main(){
 println!("cargo:rerun-if-changed=schemas.qsl");
 let builder=qsl::QslSchemaBuilder::from_source(include_str!("schemas.qsl")).unwrap();
 let code=builder.use_interpreter().unwrap();
 std::fs::write(std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("schema.rs"),code).unwrap();
}
