use qsl_examples::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scalar = packed_numbers(3).serialize_checked()?;
    let value = MixedMessage {
        a: 3,
        b: 7,
        text: "A".repeat(1024),
    };
    let bytes = value.serialize_checked()?;
    let restored = MixedMessage::deserialize_checked(&bytes)?;
    assert_eq!(
        (restored.a, restored.b, restored.text.as_str()),
        (3, 7, value.text.as_str())
    );
    assert_eq!(&bytes[1..5], &1024u32.to_le_bytes());
    println!("Scalar Q16: 64 useful B -> {} wire B", scalar.len());
    println!(
        "Mixed Q16: 16 scalar B -> 1 B; string length stays 4 B, string stays 1024 B; frame={} B",
        bytes.len()
    );
    Ok(())
}
