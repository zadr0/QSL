fn main() {
    let schemas=qsl::parse_qsl("&Plain[safety]{u32 id;str(1..100) name;u8? flag;} &Guard[safety]{u32 id;str(1..100) name|binary_opt=guard|;u8? flag;} &Borrowed[safety,lifetime]{u32 id;str(1..100) name;u8? flag;} &Packed[qps,safety]{u64(0..15) a;u64(0..15) b;} &Pool{u32 id;}").unwrap();
    for (name, algorithm, rt) in [
        ("hash64", qsl::SchemaHashAlgorithm::Xxh3_64, false),
        ("hash128", qsl::SchemaHashAlgorithm::Xxh3_128, false),
        ("rt", qsl::SchemaHashAlgorithm::Xxh3_128, true),
    ] {
        let mut g = qsl::QslInterpreter::new();
        g.config.use_hash = algorithm;
        g.config.qsl_rt = rt;
        g.generate(&schemas).unwrap();
        std::fs::write(
            std::path::Path::new(&std::env::var_os("OUT_DIR").unwrap()).join(format!("{name}.rs")),
            g.output,
        )
        .unwrap();
    }
}
