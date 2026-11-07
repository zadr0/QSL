use qsl_interpreter::prelude::*;
#[path = "primitive.rs"]
mod primitive;
fn main() {
    let mut source = String::new();
    let mut tests = String::new();
    for n in [0, 1, 8, 9, 16, 17, 32, 33, 64, 65, 128] {
        for ot in [false, true] {
            let name = format!("Bm{n}{}", if ot { "Table" } else { "Match" });
            source.push_str(&format!(
                "&{name}[safety,optionals=bitmap{}]{{u16 id;",
                if ot { ",opt_table" } else { "" }
            ));
            for i in 0..n {
                source.push_str(&format!("u16? f{i};"));
            }
            source.push('}');
            let args = (0..n)
                .map(|i| {
                    let offset = if i == 0 {
                        "seed".into()
                    } else {
                        format!("seed+{i}")
                    };
                    format!("if ({offset})%3==0{{Some({}u16)}}else{{None}}", i + 10)
                })
                .collect::<Vec<_>>()
                .join(",");
            let asserts = (0..n)
                .map(|i| format!("assert_eq!(decoded.f{i}(),value.f{i}());"))
                .collect::<String>();
            let width = match n {
                0 => 0,
                1..=8 => 8,
                9..=16 => 16,
                17..=32 => 32,
                33..=64 => 64,
                _ => 128,
            };
            let mut expected = (width / 8 + 2).to_string();
            for i in 0..n {
                expected.push_str(&format!("+2*usize::from(value.f{i}().is_some())"));
            }
            tests.push_str(&format!("#[test]fn roundtrip_{name}(){{for seed in 0..3{{let _=seed;let value={name}::try_new(0x1234,{args}).unwrap();let bytes=value.serialize_checked().unwrap();let decoded={name}::deserialize_checked(&bytes).unwrap();{asserts}assert_eq!(decoded.id,0x1234);let raw=unsafe{{<{name} as qsl::Serializable>::deserialize(&bytes)}};assert_eq!(raw.id,decoded.id);assert_eq!(bytes.len(),{expected});for len in 0..bytes.len(){{assert!({name}::deserialize_checked(&bytes[..len]).is_err());}}}}}}"));
        }
    }
    source.push_str("&Bits[safety,compact,optionals=bitmap,opt_table]{u8(0..1) id;u16?(0..15) a;str?(1..8) b;} &PredictBitmap[safety,predict_seq,optionals=bitmap,opt_table]{u8 id;u16? a;u8? b;} &Borrow[safety,lifetime,optionals=bitmap,opt_table]{u8 id;str?(1..10) a;str?(0..10) b;} &Raw[safety]{u16 id;u8? a;str?(1..10) b;u16? c;} &Table[safety,opt_table]{u16 id;u8? a;str?(1..10) b;u16? c;}");
    source.push_str("&Parent[safety]{#region(aggresive_transmutate=\"Meta\");u64 id;u32 value;str(4) tag;#end_region;str(3..10) name;} &NativeBorrow[safety,lifetime]{#region(aggressive_transmutate=\"ViewMeta\");u64 id;u32 value;str(4) tag;#end_region;str(1..10) name;} &NestedParent[safety]{#region(aggressive_transmutate=\"Outer\");u64 lead;#region(aggressive_transmutate=\"Inner\");u32 a;u32 b;#end_region;u64 tail;#end_region;u8 marker;}");
    source.push_str("&Poly[safety,optionals=bitmap,opt_table]{DeviceId id;DeviceId? a;u16? tail;} &PolyPS[safety,predict_seq,opt_table]{u8 id;DeviceId? a;u16? tail;} &PolyNative[safety]{#region(aggressive_transmutate=\"PrimitiveRegion\");DeviceId id;u32 count;#end_region;u8 code;} &Child[safety,optionals=bitmap,opt_table]{u8 id;u16? a;u16? b;} &Scoped[safety,optionals=bitmap,opt_table]{u8 id;Envelope? nested;u8? marker;}");
    source.push_str("&MacroParent[safety]{SharedMeta data;u8 marker;}");
    source.push_str("&Arrays[safety,compact,optionals=bitmap,opt_table]{u8(0..1) id;array<u64>?(0..2) values;u8?(0..3) tail;} &Lz[safety,optionals=bitmap,opt_table]{u8 id;str?(1..128) text|decompress=true|;u8? tail;} &ArrayParent[safety]{#region(aggressive_transmutate=\"ArrayRegion\");array<u8>(100) bytes;#end_region;u8 tail;}");
    source.push_str("&PolyCompact[safety,compact]{DeviceId id;DeviceId? a;} &PolyArray[safety,compact,optionals=bitmap,opt_table]{array<DeviceId>(2) ids;array<DeviceId>?(1..3) extras;}");
    let builder = QslSchemaBuilder::new()
        .with_primitive::<primitive::DeviceId>("DeviceId")
        .unwrap()
        .with_source(&source)
        .unwrap()
        .with_rust_templates(&[include_str!("native_macro.rs")])
        .unwrap();
    let generated = builder.use_interpreter().unwrap();
    let mut budget = builder
        .schemas()
        .iter()
        .find(|s| s.name == "PolyCompact")
        .unwrap()
        .clone();
    budget.name = "BudgetCompact".into();
    budget.fields.truncate(1);
    budget.max_tag = budget.fields[0].tag;
    let mut generator = qsl_interpreter::RustGenerator::new();
    generator.config.max_data_size = 1;
    generator.generate(&[budget]).unwrap();
    std::fs::write(
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("budget_types.rs"),
        generator.output,
    )
    .unwrap();
    let dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(dir.join("qsl_gen.rs"), generated).unwrap();
    // Empty optional lists use a zero size term.
    tests = tests.replace("+2+);", "+2+0);");
    tests = tests
        .replace("Bm0Match::try_new(0x1234,).unwrap()", "Bm0Match{id:0x1234}")
        .replace("Bm0Table::try_new(0x1234,).unwrap()", "Bm0Table{id:0x1234}");
    std::fs::write(dir.join("boundary_tests.rs"), tests).unwrap();
    let mut benchmark = String::new();
    for (name, options) in [
        ("BitmapMatch", "optionals=bitmap"),
        ("BitmapTable", "optionals=bitmap,opt_table"),
        ("TaggedMatch", ""),
        ("TaggedTable", "opt_table"),
    ] {
        benchmark.push_str(&format!(
            "&{name}[safety,inline{}]{{u64 id;",
            if options.is_empty() {
                String::new()
            } else {
                format!(",{options}")
            }
        ));
        for i in 0..10 {
            benchmark.push_str(&format!("u64? f{i};"));
        }
        benchmark.push('}');
    }
    let codecs = QslSchemaBuilder::from_source(&benchmark)
        .unwrap()
        .use_interpreter()
        .unwrap();
    std::fs::write(dir.join("benchmark_types.rs"), codecs).unwrap();
}
