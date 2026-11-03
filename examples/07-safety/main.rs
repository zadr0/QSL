use qsl_examples::Bounded;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = Bounded {
        kind: 2,
        name: "QSL".into(),
    };
    let bytes = value.serialize_checked()?;
    assert_eq!(Bounded::deserialize_checked(&bytes)?.name, "QSL");
    println!("MAX_DATA_SIZE={} B", Bounded::MAX_DATA_SIZE);
    for (name, input) in [
        ("invalid numeric domain", vec![9, 1, 0, 0, 0, b'a']),
        ("forged u32::MAX string length", vec![1, 255, 255, 255, 255]),
        ("invalid UTF-8", vec![1, 1, 0, 0, 0, 255]),
    ] {
        let error = Bounded::validate(&input).unwrap_err();
        println!("{name}: {error}");
    }
    Ok(())
}
