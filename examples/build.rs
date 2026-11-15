use qsl::{InterpreterBuilderExt, QslBuilder, QslSchemaBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let generated = qsl::build_qsl_files_with_rust(
        &[
            "schemas/01-basic.qsl",
            "schemas/02-strings.qsl",
            "schemas/03-optionals.qsl",
            "schemas/04-qps.qsl",
            "schemas/06-macros.qsl",
            "schemas/07-safety.qsl",
        ],
        &["schemas/config.deq"],
        &["models.rs"],
        &out,
    )?;
    std::fs::copy(&generated, out.join("normal.rs"))?;
    let generated = qsl::build_qsl_files(
        &["schemas/05-rt.qsl"],
        &["schemas/config.deq", "schemas/rt.deq"],
        &out,
    )?;
    std::fs::copy(&generated, out.join("rt.rs"))?;
    build_builder_example(&out)?;
    let bmi2 = QslSchemaBuilder::from_files(
        &["schemas/09-bmi2.qsl"],
        &["schemas/config.deq", "schemas/bmi2.deq"],
    )?;
    std::fs::write(out.join("bmi2.rs"), bmi2.use_interpreter()?)?;
    Ok(())
}

fn build_builder_example(out: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // QslBuilder collects schemas. The config closure receives a DeqBuilder;
    // each schem closure receives a QslSchemaBuilder.
    let typed = QslBuilder::new()
        .config(|config| {
            config.stack_limit(64).max_data_size(65535);
        })
        .schem(|schema| {
            schema.name("Packet").safety().aggresive(false, false);
            // u32(min, max, default, optional); str(min, max, decompress, optional).
            schema.field("id").u32(0, 1_000_000, 0, false);
            schema.field("name").str(1, 32, false, false);
            schema.field("status").u8(0, 15, 0, true);
        })
        .build()?;
    // Equivalent text input uses the same configuration and Rust backend.
    let text = QslSchemaBuilder::from_source_with_config(
        "&Packet[aggressive,safety] {u32(0..1000000) id|default=0|;str(1..32) name;u8?(0..15) status|default=0|;}",
        typed.config(),
    )?;
    std::fs::write(out.join("builder_typed.rs"), typed.use_interpreter()?)?;
    std::fs::write(out.join("builder_text.rs"), text.use_interpreter()?)?;
    Ok(())
}
