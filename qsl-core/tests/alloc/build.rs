use qsl_interpreter::InterpreterBuilderExt;
fn main() {
    let b=qsl_interpreter::QslSchemaBuilder::from_source("&Owned[safety]{str(0..512) name;array<u16>(0..16) numbers;} &OwnedLz[safety]{str(0..512) text|decompress=true|;} &WideReuse[safety,compact]{array<u128>(0..8) values;} &ByteReuse[safety,compact]{u8(0..3) prefix;array<u8>(0..8) bytes;u8(0..3) suffix;} &FixedReuse[safety]{str(512) raw;str(512) zipped|decompress=true|;} &FixedArrayReuse[safety,compact]{u8(0..3) prefix;array<i16>(300) values;u8(0..3) suffix;} &ShiftReuse[safety,aggressive[shift=true],compact]{str(0..96) text|decompress=true|;array<u16>(0..8) values;u8(0..3) tail;}").unwrap();
    let mut generated = b.use_interpreter().unwrap();
    for (name, maximum, pair) in [
        ("HeapLimit", 515, false),
        ("HeapPairLimit", 1031, true),
        ("HeapPairAllow", 1032, true),
    ] {
        let fields = if pair {
            "str(0..512) first|decompress=true|;str(0..512) second|decompress=true|;"
        } else {
            "str(0..512) text|decompress=true|;"
        };
        let source = format!("&{name}[safety]{{{fields}}}");
        let schema = qsl_interpreter::QslSchemaBuilder::from_source(&source).unwrap();
        let mut generator = qsl_interpreter::RustGenerator::new();
        generator.config.max_data_size = maximum;
        generator.generate(schema.schemas()).unwrap();
        generated.extend(generator.output);
    }
    std::fs::write(
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("schemas.rs"),
        generated,
    )
    .unwrap();
}
