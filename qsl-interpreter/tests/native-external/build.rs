use qsl::InterpreterBuilderExt;
fn main() {
    let builder=qsl::QslSchemaBuilder::from_source("&Packet[safety]{#region(aggressive_transmutate=\"ArrayMeta\");array<u16>(3) values;#end_region;u8 tail;}").unwrap().with_rust_templates(&[include_str!("models.rs")]).unwrap();
    let rust = builder.use_interpreter().unwrap();
    assert!(!String::from_utf8_lossy(&rust).contains("pub struct ArrayMeta"));
    std::fs::write(
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("schemas.rs"),
        rust,
    )
    .unwrap();
}
