mod primitive {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/primitive.rs"));
}
use qsl::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut source=String::from("&Plain[safety]{u64 id;str(1..100) text;u16? code;} &Bitmap[safety,optionals=bitmap,opt_table,predict_seq]{u8 id;u16? a;u8? b;} &Parent[safety]{#region(aggressive_transmutate=\"Meta\");u64 id;u32 count;str(4) tag;#end_region;u8 tail;} &Wrapper[safety,compact,optionals=bitmap,opt_table]{DeviceId id;DeviceId? extra;} &Quant[safety,qps=Q16]{u64(0..15) a;u64(0..15) b;} &Wide[safety,optionals=bitmap,opt_table]{u8 id;");
    for i in 0..65 {
        source.push_str(&format!("u16? f{i};"));
    }
    source.push('}');
    let b = QslSchemaBuilder::new()
        .with_primitive::<primitive::DeviceId>("DeviceId")?
        .with_source(&source)?;
    let rust = b.use_interpreter()?;
    std::fs::write(
        std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("qsl_gen.rs"),
        rust,
    )?;
    Ok(())
}
