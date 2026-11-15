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
    let output_schemas = qsl::parse_qsl(
        r#"
&OutputDomain[safety,compact]{u128(0..15) f0;u128(0..15) f1;u128(0..15) f2;u128(0..15) f3;u128(0..15) f4;u128(0..15) f5;u128(0..15) f6;u128(0..15) f7;u128(0..15) f8;u128(0..15) f9;u128(0..15) f10;u128(0..15) f11;u128(0..15) f12;u128(0..15) f13;u128(0..15) f14;u128(0..15) f15;}
&Output64[safety]{array<u8>(64) bytes;}
&Output252[safety]{str(252) text;}
&Output253[safety]{str(253) text;}
&Output256[safety]{str(256) text;}
&Output257[safety]{str(257) text;}
&OutputLarge[safety]{str(0..4096) first;str(0..4096) second;}
&OutputUnknown[safety]{str text;}
&OutputBorrowed[safety,lifetime]{str(0..64) text;}
&OutputLz[safety]{str(0..100) text|decompress=true|;}
&OutputQuant[safety,qps]{u64(0..15) a;u64(0..15) b;}
&OutputDyn[safety,dyn[type=line,width=16]]{u8 id=1;str?(0..8) text=2;}
"#,
    )
    .unwrap();
    let mut ordinary = qsl::QslInterpreter::new();
    ordinary.generate(&output_schemas).unwrap();
    g.output.extend(ordinary.output);
    use qsl::InterpreterBuilderExt;
    let limited = qsl::QslBuilder::new()
        .config(|c| {
            c.stack_limit(64);
        })
        .schem(|s| {
            s.name("OutputLimit64")
                .safety()
                .field("bytes")
                .farray::<u8>(64, false);
        })
        .build()
        .unwrap();
    g.output.extend(limited.use_interpreter().unwrap());
    let deq = qsl::parse_deq("!config STACK_LIMIT = 64;").unwrap();
    let limited = qsl::QslSchemaBuilder::from_source_with_config(
        "&OutputLimit65[safety]{str(64) bytes;u8 tail;}",
        deq.generation,
    )
    .unwrap();
    g.output.extend(limited.use_interpreter().unwrap());
    let zero = qsl::QslBuilder::new()
        .config(|c| {
            c.stack_limit(0);
        })
        .schem(|s| {
            s.name("OutputZero")
                .safety()
                .field("bytes")
                .farray::<u8>(0, false);
        })
        .build()
        .unwrap();
    g.output.extend(zero.use_interpreter().unwrap());
    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("qsl_gen.rs"),
        g.output,
    )
    .unwrap();
}
