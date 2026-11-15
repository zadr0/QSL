#![no_std]
include!(concat!(env!("OUT_DIR"), "/schemas.rs"));
pub fn owned_roundtrip() -> qsl::QslResult<Owned> {
    let mut numbers = qsl::alloc::vec::Vec::new();
    numbers.extend_from_slice(&[0, 1, 65535]);
    let v = Owned {
        name: qsl::alloc::string::String::from("Яa"),
        numbers,
    };
    let bytes = v.serialize_checked()?;
    Owned::deserialize_checked(&bytes)
}
#[cfg(test)]
mod tests {
    #[test]
    fn allocated_values_are_exact() {
        let v = super::owned_roundtrip().unwrap();
        assert_eq!(v.name, "Яa");
        assert_eq!(v.numbers, [0, 1, 65535]);
    }
    #[test]
    fn heap_inline_preserves_type_codec_and_checked_restoration() {
        for text in [
            "",
            "Яa",
            "abcde".repeat(102).as_str(),
            "a".repeat(512).as_str(),
        ] {
            let text = qsl::InlineString::<512>::try_boxed_from_str(text).unwrap();
            let v = super::OwnedLz { text };
            let bytes = v.serialize_checked().unwrap();
            let back = super::OwnedLz::deserialize_checked(&bytes).unwrap();
            let _: &qsl::alloc::boxed::Box<qsl::InlineString<512>> = &back.text;
            assert_eq!(back.text.as_str(), v.text.as_str());
            let back = unsafe { <super::OwnedLz as qsl::Serializable>::deserialize(&bytes) };
            assert_eq!(back.text.as_str(), v.text.as_str());
        }
        let v = qsl::InlineString::<512>::try_boxed_spaces(32).unwrap();
        assert_eq!(v.as_str(), " ".repeat(32));
    }
    #[test]
    fn forged_lz_never_publishes_an_overflow() {
        let mut c = qsl::validation::ValidationCursor::new(&[0x04, 0x80, 0, b'A', 0xff, 1]);
        assert!(c
            .decode_compressed_boxed::<100, u32>(0, false, false)
            .is_err());
        let mut c = qsl::validation::ValidationCursor::new(&[0x84, 0, b'A', 0xff, 1]);
        assert!(c
            .decode_compressed_boxed::<100, u32>(0, false, false)
            .is_err());
    }
    #[test]
    fn checked_writer_and_reader_agree_on_fixed_heap_capacity_budgets() {
        let value = super::HeapLimit {
            text: qsl::InlineString::try_boxed_from_str("a").unwrap(),
        };
        assert!(matches!(
            value.serialize_checked(),
            Err(qsl::QslError::SizeLimitExceeded)
        ));
        assert!(matches!(
            super::HeapLimit::deserialize_checked(&[1, 0, b'a']),
            Err(qsl::QslError::SizeLimitExceeded)
        ));
        let trusted =
            unsafe { <super::HeapLimit as qsl::Serializable>::deserialize(&[1, 0, b'a']) };
        assert_eq!(trusted.text.as_str(), "a");
        let blocked = super::HeapPairLimit {
            first: qsl::InlineString::try_boxed_from_str("a").unwrap(),
            second: qsl::InlineString::try_boxed_from_str("b").unwrap(),
        };
        assert!(matches!(
            blocked.serialize_checked(),
            Err(qsl::QslError::SizeLimitExceeded)
        ));
        let frame = [1, 0, b'a', 1, 0, b'b'];
        assert!(matches!(
            super::HeapPairLimit::deserialize_checked(&frame),
            Err(qsl::QslError::SizeLimitExceeded)
        ));
        let value = super::HeapPairAllow {
            first: qsl::InlineString::try_boxed_from_str("a").unwrap(),
            second: qsl::InlineString::try_boxed_from_str("b").unwrap(),
        };
        let bytes = value.serialize_checked().unwrap();
        assert_eq!(bytes, frame);
        let back = super::HeapPairAllow::deserialize_checked(&bytes).unwrap();
        assert_eq!(back.first.as_str(), "a");
        assert_eq!(back.second.as_str(), "b");
    }
    #[test]
    fn deserialize_into_retains_heap_string_box_and_numeric_vector_allocations() {
        const {
            assert!(<super::Owned as qsl::Serializable>::REUSES_STORAGE);
            assert!(<super::OwnedLz as qsl::Serializable>::REUSES_STORAGE);
        }
        let mut out = super::Owned {
            name: qsl::alloc::string::String::with_capacity(512),
            numbers: qsl::alloc::vec::Vec::with_capacity(16),
        };
        let name_ptr = out.name.as_ptr();
        let numbers_ptr = out.numbers.as_ptr();
        for len in [0, 1, 3, 16, 0, 8] {
            let source = super::Owned {
                name: "Яa".repeat(len),
                numbers: (0..len).map(|i| (i * 17) as u16).collect(),
            };
            let frame = source.serialize_checked().unwrap();
            unsafe {
                super::Owned::deserialize_into(frame.as_ptr(), frame.len(), &mut out);
            }
            assert_eq!(out.name, source.name);
            assert_eq!(out.numbers, source.numbers);
            assert_eq!(out.name.as_ptr(), name_ptr);
            assert_eq!(out.numbers.as_ptr(), numbers_ptr);
        }
        let mut out = super::OwnedLz {
            text: qsl::InlineString::try_boxed_new().unwrap(),
        };
        let pointer = out.text.as_bytes().as_ptr();
        for text in [
            "",
            "Яa",
            "abcde".repeat(102).as_str(),
            "a".repeat(512).as_str(),
            "",
        ] {
            let source = super::OwnedLz {
                text: qsl::InlineString::try_boxed_from_str(text).unwrap(),
            };
            let frame = source.serialize_checked().unwrap();
            unsafe {
                super::OwnedLz::deserialize_into(frame.as_ptr(), frame.len(), &mut out);
            }
            assert_eq!(out.text.as_str(), text);
            assert_eq!(out.text.as_bytes().as_ptr(), pointer);
        }
    }
    #[test]
    fn primitive_array_reuse_preserves_wide_values_and_compact_byte_barriers() {
        const {
            assert!(<super::WideReuse as qsl::Serializable>::REUSES_STORAGE);
            assert!(<super::ByteReuse as qsl::Serializable>::REUSES_STORAGE);
        }
        let mut wide = super::WideReuse {
            values: qsl::alloc::vec::Vec::with_capacity(8),
        };
        let pointer = wide.values.as_ptr();
        let mut bytes = super::ByteReuse {
            prefix: 0,
            bytes: qsl::alloc::vec::Vec::with_capacity(8),
            suffix: 0,
        };
        let byte_ptr = bytes.bytes.as_ptr();
        for len in [0, 1, 3, 8, 0] {
            let source = super::WideReuse {
                values: (0..len).map(|i| u128::MAX - i as u128).collect(),
            };
            let frame = source.serialize_checked().unwrap();
            unsafe {
                super::WideReuse::deserialize_into(frame.as_ptr(), frame.len(), &mut wide);
            }
            assert_eq!(wide.values, source.values);
            assert_eq!(wide.values.as_ptr(), pointer);
            let source = super::ByteReuse {
                prefix: 3,
                bytes: (0..len).map(|i| (i * 31) as u8).collect(),
                suffix: 2,
            };
            let frame = source.serialize_checked().unwrap();
            unsafe {
                super::ByteReuse::deserialize_into(frame.as_ptr(), frame.len(), &mut bytes);
            }
            assert_eq!((bytes.prefix, bytes.suffix), (3, 2));
            assert_eq!(bytes.bytes, source.bytes);
            assert_eq!(bytes.bytes.as_ptr(), byte_ptr);
        }
    }
    #[test]
    fn fixed_heap_and_shifted_inline_destinations_reuse_their_storage() {
        const {
            assert!(<super::FixedReuse as qsl::Serializable>::REUSES_STORAGE);
            assert!(<super::FixedArrayReuse as qsl::Serializable>::REUSES_STORAGE);
            assert!(<super::ShiftReuse as qsl::Serializable>::REUSES_STORAGE);
        }
        let mut fixed = super::FixedReuse {
            raw: qsl::alloc::string::String::with_capacity(512),
            zipped: qsl::alloc::string::String::with_capacity(512),
        };
        let pointers = (fixed.raw.as_ptr(), fixed.zipped.as_ptr());
        for raw in ["Я".repeat(256), "a".repeat(512)] {
            let source = super::FixedReuse {
                raw: raw.clone(),
                zipped: raw,
            };
            let frame = source.serialize_checked().unwrap();
            unsafe {
                super::FixedReuse::deserialize_into(frame.as_ptr(), frame.len(), &mut fixed);
            }
            assert_eq!(fixed.raw, source.raw);
            assert_eq!(fixed.zipped, source.zipped);
            assert_eq!((fixed.raw.as_ptr(), fixed.zipped.as_ptr()), pointers);
        }
        let mut array = super::FixedArrayReuse {
            prefix: 0,
            values: qsl::alloc::vec::Vec::with_capacity(300),
            suffix: 0,
        };
        let pointer = array.values.as_ptr();
        for (prefix, suffix) in [(0, 3), (3, 0)] {
            let source = super::FixedArrayReuse {
                prefix,
                values: (0..300)
                    .map(|i| {
                        if i % 2 == 0 {
                            i16::MIN + i as i16
                        } else {
                            i16::MAX - i as i16
                        }
                    })
                    .collect(),
                suffix,
            };
            let frame = source.serialize_checked().unwrap();
            unsafe {
                super::FixedArrayReuse::deserialize_into(frame.as_ptr(), frame.len(), &mut array);
            }
            assert_eq!(array.values, source.values);
            assert_eq!((array.prefix, array.suffix), (prefix, suffix));
            assert_eq!(array.values.as_ptr(), pointer);
        }
        let mut shifted = super::ShiftReuse {
            text: qsl::InlineString::new(),
            values: qsl::InlineArray::new(),
            tail: 0,
        };
        for text in [
            "",
            "Яa",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ] {
            let source = super::ShiftReuse {
                text: qsl::InlineString::try_from_str(text).unwrap(),
                values: qsl::InlineArray::try_from_slice(&[0, 65535, 7]).unwrap(),
                tail: 3,
            };
            let frame = source.serialize_checked().unwrap();
            unsafe {
                super::ShiftReuse::deserialize_into(frame.as_ptr(), frame.len(), &mut shifted);
            }
            assert_eq!(shifted.text.as_str(), text);
            assert_eq!(shifted.values.as_slice(), source.values.as_slice());
            assert_eq!(shifted.tail, 3);
        }
    }
}
