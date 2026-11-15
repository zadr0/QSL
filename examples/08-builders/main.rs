pub mod typed {
    include!(concat!(env!("OUT_DIR"), "/builder_typed.rs"));
}
pub mod text {
    include!(concat!(env!("OUT_DIR"), "/builder_text.rs"));
}

fn main() -> qsl::QslResult<()> {
    for status in [None, Some(7)] {
        let a = typed::Packet::try_new(42, qsl::StackString::try_from_str("QSL")?, status)?;
        let b = text::Packet::try_new(42, qsl::StackString::try_from_str("QSL")?, status)?;
        let bytes: qsl::StackOut<42> = a.serialize_checked()?;
        assert_eq!(bytes.as_ref(), b.serialize_checked()?.as_ref());
        let decoded = text::Packet::deserialize_checked(&bytes)?;
        assert_eq!(
            (decoded.id, decoded.name.as_str(), decoded.status()),
            (42, "QSL", status)
        );
        assert!(text::Packet::deserialize_checked(&bytes[..3]).is_err());
        println!(
            "Builder / QSL text: status={status:?}, {} identical wire B",
            bytes.len()
        );
    }
    assert_eq!(typed::Packet::SCHEMA_HASHES, text::Packet::SCHEMA_HASHES);
    assert_eq!(typed::Packet::MAX_DATA_SIZE, 65535);
    println!(
        "STACK_LIMIT=64: StackString<32>, stack output; required fields precede optional status"
    );
    Ok(())
}
