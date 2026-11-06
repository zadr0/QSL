fn main() {
    println!("cargo:rerun-if-changed=schemas.qsl");
    println!("cargo:rerun-if-changed=models.rs");
    let source = std::fs::read_to_string("schemas.qsl").unwrap();
    let rust = std::fs::read_to_string("models.rs").unwrap();
    let schemas =
        qsl::rust_macro::parse_qsl_with_rust(&source, &Default::default(), &[&rust]).unwrap();
    let mut g = qsl::QslInterpreter::new();
    g.config.qsl_rt = true;
    g.generate(&schemas).unwrap();
    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("qsl_gen.rs"),
        g.output,
    )
    .unwrap();
}
