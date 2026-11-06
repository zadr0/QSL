use qsl_interpreter::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = build_qsl_schem(".")?;
    let rust = builder.use_interpreter()?;
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("qsl_gen.rs");
    std::fs::write(output, rust)?;
    Ok(())
}
