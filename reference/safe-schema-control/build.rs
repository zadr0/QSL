fn main() {
    println!("cargo:rerun-if-changed=schema.qsl");
    let schemas = qsl::parse_qsl(&std::fs::read_to_string("schema.qsl").unwrap()).unwrap();
    for (name, limit) in [("normal", 65535), ("small", 64), ("tiny", 8)] {
        let mut generator = qsl::QslInterpreter::new();
        generator.config.max_data_size = limit;
        generator.generate(&schemas).unwrap();
        std::fs::write(
            std::path::Path::new(&std::env::var_os("OUT_DIR").unwrap()).join(format!("{name}.rs")),
            generator.output,
        )
        .unwrap();
    }
}
