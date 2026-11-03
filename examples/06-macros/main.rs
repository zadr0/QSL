use qsl_examples::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = models::Trace {
        header: models::Header { id: 42 },
        kind: 7,
    };
    let flat = MacroPacket {
        trace_header_id: original.header.id,
        trace_kind: original.kind,
        status: Some(1),
    };
    let frame = flat.serialize_checked()?;
    let h = MacroPacket::SCHEMA_HASHES;
    println!("Rust Trace stays nested; generated fields: trace_header_id, trace_kind, status");
    println!(
        "{} B; {} strict={}, light={}; computed during analysis",
        frame.len(),
        h.algorithm.name(),
        h.strict_hex(),
        h.light_hex()
    );
    Ok(())
}
