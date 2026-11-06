include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));

fn main() -> qsl::QslResult<()> {
    let user = User {
        id: 42,
        name: "Ada".into(),
    };
    let bytes = user.serialize_checked()?;
    let restored = User::deserialize_checked(&bytes)?;
    assert_eq!((restored.id, restored.name.as_str()), (42, "Ada"));
    println!("{}: {} ({} bytes)", restored.id, restored.name, bytes.len());
    Ok(())
}
