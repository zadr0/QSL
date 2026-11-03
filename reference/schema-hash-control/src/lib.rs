#![allow(dead_code)]
mod h64 {
    include!(concat!(env!("OUT_DIR"), "/hash64.rs"));
}
mod h128 {
    include!(concat!(env!("OUT_DIR"), "/hash128.rs"));
}
mod rt {
    include!(concat!(env!("OUT_DIR"), "/rt.rs"));
}
#[test]
fn generated_hashes_cover_owned_borrowed_packed_and_rt_and_preserve_wire() {
    use qsl::{BorrowSerializable, SchemaIdentity, Serializable};
    assert_eq!(
        h64::Plain::SCHEMA_HASHES,
        <h64::Plain as SchemaIdentity>::SCHEMA_HASHES
    );
    assert_eq!(h64::Plain::SCHEMA_HASH_LIGHT, h64::Guard::SCHEMA_HASH_LIGHT);
    assert_eq!(
        h64::Plain::SCHEMA_HASH_LIGHT,
        h64::Borrowed::SCHEMA_HASH_LIGHT
    );
    assert_eq!(h128::Pool::SCHEMA_HASHES, rt::Pool::SCHEMA_HASHES);
    assert_eq!(h128::Plain::SCHEMA_HASHES, rt::Plain::SCHEMA_HASHES);
    assert_eq!(
        h64::Packed::SCHEMA_HASHES.algorithm,
        qsl::SchemaHashAlgorithm::Xxh3_64
    );
    assert_eq!(
        h128::Packed::SCHEMA_HASHES.algorithm,
        qsl::SchemaHashAlgorithm::Xxh3_128
    );
    let a = h64::Plain {
        id: 42,
        name: "hello".into(),
        flag: Some(7),
    };
    let b = h128::Plain {
        id: 42,
        name: "hello".into(),
        flag: Some(7),
    };
    let borrowed = h64::Borrowed {
        id: 42,
        name: "hello",
        flag: Some(7),
    };
    let bytes = a.serialize();
    assert_eq!(bytes, b.serialize());
    assert_eq!(bytes, borrowed.serialize());
    let decoded = h64::Plain::deserialize_checked(&bytes).unwrap();
    assert_eq!(decoded.name, "hello");
    let a = h64::Packed { a: 3, b: 15 };
    let b = h128::Packed { a: 3, b: 15 };
    assert_eq!(
        <h64::Packed as Serializable>::serialize(&a),
        <h128::Packed as Serializable>::serialize(&b)
    );
}
