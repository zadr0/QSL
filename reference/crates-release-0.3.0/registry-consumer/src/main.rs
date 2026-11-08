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
    println!("{{\"version\":\"0.3.0\",\"bitmapFrames\":{:?}}}", frames);
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

#[cfg(test)]
mod core_string_contracts {
    use super::*;
    use qsl::Serializable;
    #[test]
    fn four_strings_mixed_ownership_and_reuse() {
        let text = "Точный UTF-8";
        let bytes = OwnedText { text: text.into() }.serialize_checked().unwrap();
        let stack = StackText::deserialize_checked(&bytes).unwrap();
        let _: &qsl_core::StackString<64> = &stack.text;
        let view = ViewText::deserialize_checked(&bytes).unwrap();
        let _: &str = view.text;
        assert_eq!(view.text.as_ptr(), unsafe { bytes.as_ptr().add(4) });
        let inline = InlineText {
            text: unsafe { qsl_core::InlineString::from_str_unchecked(text) },
        };
        let frame = inline.serialize_checked().unwrap();
        assert_eq!(
            InlineText::deserialize_checked(&frame)
                .unwrap()
                .text
                .as_str(),
            text
        );
        let mixed = MixedText {
            owned: text.into(),
            view: text,
        };
        let frame = mixed.serialize_checked().unwrap();
        let d = MixedText::deserialize_checked(&frame).unwrap();
        assert_eq!(d.owned, d.view);
        let mut out = OwnedText {
            text: String::with_capacity(64),
        };
        let ptr = out.text.as_ptr();
        unsafe { OwnedText::deserialize_reuse(&bytes, &mut out) };
        assert_eq!(out.text, text);
        assert_eq!(ptr, out.text.as_ptr());
    }
    #[test]
    fn shared_error_identity_and_untrusted_strings() {
        let result: qsl_core::QslResult<()> = Err(qsl::QslError::InvalidValue);
        assert!(result.is_err());
        let _: qsl_lexer::QslError = qsl_core::QslError::InvalidValue;
        for mode in ["guard", "true", "bytes"] {
            assert!(qsl_lexer::QslSchemaBuilder::from_source(&format!(
                "&Bad{{str(0..64) text|binary_opt={mode}|;}}"
            ))
            .is_err());
        }
        let frame = OwnedText {
            text: "valid".into(),
        }
        .serialize_checked()
        .unwrap();
        for n in 0..frame.len() {
            assert!(OwnedText::deserialize_checked(&frame[..n]).is_err());
            assert!(StackText::deserialize_checked(&frame[..n]).is_err());
            assert!(ViewText::deserialize_checked(&frame[..n]).is_err());
        }
        let mut forged = frame.clone();
        forged[..4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(OwnedText::deserialize_checked(&forged).is_err());
        assert!(StackText::deserialize_checked(&forged).is_err());
        assert!(ViewText::deserialize_checked(&forged).is_err());
        let mut invalid = frame;
        invalid[4] = 255;
        assert!(OwnedText::deserialize_checked(&invalid).is_err());
        assert!(ViewText::deserialize_checked(&invalid).is_err());
    }
}
