include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(Packet::MAX_DATA_SIZE, 1024);
    let value = Packet::try_new(42, "QSL".into(), Some(7))?;
    let bytes = value.serialize_checked()?;
    let restored = Packet::deserialize_checked(&bytes)?;
    assert_eq!(restored.id, value.id);
    assert_eq!(restored.text, value.text);
    assert_eq!(restored.status(), value.status());
    assert!(Packet::deserialize_checked(&bytes[..3]).is_err());
    println!(
        "Separate compiler crates: {} wire bytes, checked round-trip passed",
        bytes.len()
    );
    Ok(())
}
