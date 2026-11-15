#[allow(dead_code)]
#[qsl::qsl_macro]
struct HostStorage {
    name: qsl::StackString<32, core::num::NonZeroU8>,
    values: qsl::InlineArray<u16, 8, core::num::NonZeroU8>,
    extra: Option<qsl::InlineArray<u8, 4, core::num::NonZeroU8>>,
}
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
        println!("cargo:rustc-link-lib=vcruntime");
        println!("cargo:rustc-link-lib=ucrt");
        println!("cargo:rustc-link-arg=/SUBSYSTEM:CONSOLE");
    }
    let source = r#"
&NativeRecord[safety]{#region(aggressive_transmutate="NativeFields");u32 id;u32 value;#end_region;u8 tail;}
&Basic[safety,compact]{u8(0..7) kind;u16(0..1023) value;}
&StackRecord[safety,aggressive[shift=true]]{str(0..32) name;array<u16>(0..8) values;}
&LzRecord[safety]{str(0..96) text|decompress=true|;}
&BorrowedRecord[safety,lifetime]{str(0..96) name;}
&LargeBorrowedRecord[safety,lifetime]{str(0..4096) name;}
&Predicted[safety,predict_seq,opt_table]{u32 id;u8? a;u16? b;}
&OptionalArrays[safety,aggressive[shift=true],optionals=bitmap,opt_table]{u8 id;array<u16>?(0..8) values;}
&QuantRecord[safety,qps=Q16]{u64(0..15) a;u64(0..15) b;u64(0..15) c;u64(0..15) d;}
"#;
    let builder = qsl_interpreter::QslSchemaBuilder::from_source_with_config(
        source,
        qsl_interpreter::GenerationConfig {
            qsl_rt: true,
            qsl_rt_default: 2,
            ..Default::default()
        },
    )
    .unwrap();
    use qsl_interpreter::InterpreterBuilderExt;
    let mut typed = qsl::QslBuilder::new();
    typed.schem(|s| {
        s.name("TypedStorage")
            .safety()
            .aggresive(true, false)
            .r#macro::<HostStorage>();
    });
    let typed = typed.build().unwrap();
    let mut schemas = builder.schemas().to_vec();
    schemas.extend_from_slice(typed.schemas());
    let builder = qsl::QslSchemaBuilder::from_schemas(&schemas, builder.config()).unwrap();
    let generated = builder.use_interpreter().unwrap();
    std::fs::write(
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("schemas.rs"),
        generated,
    )
    .unwrap();
}
