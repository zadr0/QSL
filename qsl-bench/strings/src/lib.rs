include!(concat!(env!("OUT_DIR"), "/strings.rs"));
#[cfg(test)]
mod contracts {
    use super::*;
    use qsl::Serializable;
    #[test]
    fn string_models_and_mixed_ownership_are_real_generated_types() {
        let text = "Я".repeat(1024);
        let bytes = Owned { text: text.clone() }.serialize_checked().unwrap();
        let stack = Stack::deserialize_checked(&bytes).unwrap();
        let _: &qsl::StackString<2048> = &stack.text;
        let view = View::deserialize_checked(&bytes).unwrap();
        let _: &str = view.text;
        assert_eq!(view.text.as_ptr(), unsafe { bytes.as_ptr().add(4) });
        let inline = Inline {
            text: unsafe { qsl::InlineString::from_str_unchecked(&text) },
        };
        let f = inline.serialize_checked().unwrap();
        assert_eq!(Inline::deserialize_checked(&f).unwrap().text.as_str(), text);
        let mixed = Mixed {
            owned: text.clone(),
            view: &text,
        };
        let f = mixed.serialize_checked().unwrap();
        let restored = Mixed::deserialize_checked(&f).unwrap();
        assert_eq!(restored.owned, restored.view);
        let mut owned = Owned {
            text: String::with_capacity(2048),
        };
        let p = owned.text.as_ptr();
        unsafe { Owned::deserialize_reuse(&bytes, &mut owned) };
        assert_eq!(owned.text, text);
        assert_eq!(p, owned.text.as_ptr());
    }
    #[test]
    fn checked_models_reject_forged_lengths_invalid_utf8_and_truncation() {
        let bytes = Owned {
            text: "valid".into(),
        }
        .serialize_checked()
        .unwrap();
        for n in 0..bytes.len() {
            assert!(Owned::deserialize_checked(&bytes[..n]).is_err());
            assert!(View::deserialize_checked(&bytes[..n]).is_err());
            assert!(Stack::deserialize_checked(&bytes[..n]).is_err());
        }
        let mut f = bytes.clone();
        f[..4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Owned::deserialize_checked(&f).is_err());
        assert!(Stack::deserialize_checked(&f).is_err());
        assert!(View::deserialize_checked(&f).is_err());
        let mut f = bytes;
        f[4] = 0xff;
        assert!(View::deserialize_checked(&f).is_err());
        assert!(Stack::deserialize_checked(&f).is_err());
    }
}
