include!("../models.rs");
include!(concat!(env!("OUT_DIR"), "/schemas.rs"));
#[cfg(test)]
mod tests {
    #[test]
    fn native_macro_is_defined_once_and_roundtrips_exact_wire() {
        let value = super::Packet {
            ArrayMeta: super::ArrayMeta {
                values: [1, 2, 65535],
            },
            tail: 7,
        };
        let bytes = value.serialize_checked().unwrap();
        assert_eq!(bytes, [1, 0, 2, 0, 255, 255, 7]);
        let back = super::Packet::deserialize_checked(&bytes).unwrap();
        assert_eq!(back.ArrayMeta.values, [1, 2, 65535]);
        assert_eq!(back.tail, 7);
    }
}
