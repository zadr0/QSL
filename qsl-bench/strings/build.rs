use qsl_interpreter::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = qsl_interpreter::GenerationConfig {
        stack_limit: 2048,
        ..Default::default()
    };
    let builder=QslSchemaBuilder::from_source_with_config("&Owned[safety]{str(1..2048) text;} &Stack[safety,aggressive]{str(1..2048) text;} &Inline[safety]{str(1..2048) text|binary_opt=inline|;} &View[safety]{str(1..2048) text|binary_opt=view|;} &Mixed[safety]{str(1..2048) owned;str(1..2048) view|binary_opt=view|;}",config)?;
    let rust = builder.use_interpreter()?;
    std::fs::write(
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("strings.rs"),
        rust,
    )?;
    Ok(())
}
