mod primitive {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/primitive.rs"));
}
use primitive::DeviceId;
qsl::qsl_native_primitive!(DeviceId, u32);
include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
fn main() {
    let mut frames = Vec::new();
    for mask in 0..4 {
        let p = Bitmap::try_new(
            9,
            (mask & 1 != 0).then_some(0x1234),
            (mask & 2 != 0).then_some(7),
        )
        .unwrap();
        frames.push(p.serialize_checked().unwrap());
    }
    println!("{{\"version\":\"0.2.0\",\"bitmapFrames\":{:?}}}", frames);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_owned_and_into() {
        let value = Plain::try_new(42, "QSL".into(), Some(7)).unwrap();
        let frame = value.serialize_checked().unwrap();
        let d = Plain::deserialize_checked(&frame).unwrap();
        assert_eq!(d.text, "QSL");
        assert_eq!(d.code(), Some(7));
        let mut out = std::mem::MaybeUninit::uninit();
        assert_eq!(
            Plain::deserialize_checked_into(&frame, &mut out)
                .unwrap()
                .id,
            42
        );
        assert!(Plain::deserialize_checked(&frame[..3]).is_err());
        let raw = unsafe { <Plain as qsl::Serializable>::deserialize(&frame) };
        assert_eq!(raw.text, "QSL");
    }
    #[test]
    fn bitmap_order_and_errors() {
        for mask in 0..4 {
            let p = Bitmap::try_new(
                9,
                (mask & 1 != 0).then_some(0x1234),
                (mask & 2 != 0).then_some(7),
            )
            .unwrap();
            let bytes = p.serialize_checked().unwrap();
            let mut expected = vec![((mask & 1) * 128 + (mask >> 1) * 64) as u8, 9];
            if mask & 2 != 0 {
                expected.push(7)
            }
            if mask & 1 != 0 {
                expected.extend_from_slice(&[52, 18])
            }
            assert_eq!(bytes, expected);
            let d = Bitmap::deserialize_checked(&bytes).unwrap();
            assert_eq!(d.a(), p.a());
            assert_eq!(d.b(), p.b());
            let mut invalid = bytes;
            invalid[0] |= 1;
            assert!(Bitmap::deserialize_checked(&invalid).is_err());
        }
    }
    #[test]
    fn nested_region_type() {
        let value = Parent {
            Meta: Meta {
                id: 7,
                count: 8,
                tag: *b"abcd",
            },
            tail: 9,
        };
        let bytes = value.serialize_checked().unwrap();
        assert_eq!(bytes.len(), 17);
        let d = Parent::deserialize_checked(&bytes).unwrap();
        assert_eq!(d.Meta.id, 7);
        assert_eq!(d.Meta.tag, *b"abcd");
    }
    #[test]
    fn validated_primitive_and_original_budget() {
        let value = Wrapper {
            id: DeviceId(42),
            extra: Some(DeviceId(7)),
        };
        let bytes = value.serialize_checked().unwrap();
        assert_eq!(bytes, [128, 41, 6]);
        let d = Wrapper::deserialize_checked(&bytes).unwrap();
        assert_eq!(d.extra, Some(DeviceId(7)));
        assert!(Wrapper::deserialize_checked(&[128, 255, 6]).is_err());
        assert!(Wrapper {
            id: DeviceId(0),
            extra: None
        }
        .serialize_checked()
        .is_err());
    }
    #[test]
    fn qps_exact_values() {
        let value = Quant { a: 3, b: 15 };
        let bytes = value.serialize_checked().unwrap();
        let d = Quant::deserialize_checked(&bytes).unwrap();
        assert_eq!((d.a, d.b), (3, 15));
        assert_eq!(Quant::QUANT_PADDING_BYTES, 0);
    }
    #[test]
    fn wide_bitmap_header_uses_u128() {
        let mut bytes = vec![0u8; 17];
        bytes[16] = 9;
        let d = Wide::deserialize_checked(&bytes).unwrap();
        assert_eq!(d.id, 9);
        assert!(d.f64().is_none());
        assert_eq!(d.serialize_checked().unwrap(), bytes);
        bytes[0] = 1;
        assert!(Wide::deserialize_checked(&bytes).is_err());
    }
}
