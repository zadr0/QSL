use qsl::QslResult;
#[derive(Debug, PartialEq)]
pub struct Tiny(u8);
impl qsl::QslCustom for Tiny {
    const SIZE: usize = 1;
    const COMP_SIZE: usize = 1;
    unsafe fn qsl_write_unchecked(&self, to: *mut u8) {
        to.write(self.0);
    }
    unsafe fn qsl_read_unchecked(from: *const u8) -> Self {
        Self(from.read())
    }
    fn qsl_validate(bytes: &[u8], _: bool) -> QslResult<()> {
        if bytes.len() == 1 {
            Ok(())
        } else {
            Err(qsl::QslError::InvalidValue)
        }
    }
    fn qsl_validate_values(&self) -> QslResult<()> {
        Ok(())
    }
}
#[allow(dead_code)]
mod before {
    include!("before.rs");
}
include!(concat!(env!("OUT_DIR"), "/qsl_generated.rs"));
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tag_table_uses_schema_tags_and_preserves_default_bytes() {
        for m in 0..4 {
            let a = (m & 1 != 0).then_some(11);
            let b = (m & 2 != 0).then_some(22);
            let d = PairDefault::try_new(42, a, b)
                .unwrap()
                .serialize_checked()
                .unwrap();
            let t = PairTag::try_new(42, a, b)
                .unwrap()
                .serialize_checked()
                .unwrap();
            assert_eq!(d, t);
            let x = PairTag::deserialize_checked(&d).unwrap();
            assert_eq!((x.id, x.a(), x.b()), (42, a, b));
            assert_eq!(PairTag::SCHEMA_HASH_LIGHT, PairDefault::SCHEMA_HASH_LIGHT);
        }
        // Required tag is not an optional ordinal. It has no callable table entry.
        assert!(__QslFields_PairTag_OPT_TABLE[0].is_none());
        assert!(__QslFields_PairTag_OPT_TABLE[1].is_some());
        assert!(__QslFields_PairTag_OPT_TABLE[2].is_some());
        let mut invalid = PairTag::try_new(42, Some(1), None)
            .unwrap()
            .serialize_checked()
            .unwrap();
        invalid[0] = 1;
        assert!(PairTag::deserialize_checked(&invalid).is_err());
    }
    #[test]
    fn old_combined_numeric_bytes_and_hashes_are_preserved() {
        for m in 0..8 {
            let a = (m & 1 != 0).then_some(7);
            let b = (m & 2 != 0).then_some(0x1234);
            let c = (m & 4 != 0).then_some(0x12345678);
            let old = before::Unique { id: 42, a, b, c }
                .serialize_checked()
                .unwrap();
            let both = Unique::try_new(42, a, b, c)
                .unwrap()
                .serialize_checked()
                .unwrap();
            assert_eq!(old, both);
            assert_eq!(before::Unique::SCHEMA_HASHES, Unique::SCHEMA_HASHES);
            let x = Unique::deserialize_checked(&old).unwrap();
            assert_eq!((x.a(), x.b(), x.c()), (a, b, c));
            let seq = UniquePredict::try_new(42, a, b, c)
                .unwrap()
                .serialize_checked()
                .unwrap();
            assert_eq!(seq, both);
            let x = UniquePredict::deserialize_checked(&seq).unwrap();
            assert_eq!((x.a(), x.b(), x.c()), (a, b, c));
        }
        for m in 0..4 {
            let a = (m & 1 != 0).then_some(8);
            let b = (m & 2 != 0).then_some(9);
            let old = before::Pair { id: 5, a, b }.serialize_checked().unwrap();
            let now = Pair::try_new(5, a, b).unwrap().serialize_checked().unwrap();
            assert_eq!(old, now);
            let x = Pair::deserialize_checked(&old).unwrap();
            assert_eq!((x.a(), x.b()), (a, b));
        }
    }
    #[test]
    fn strings_and_ambiguous_lengths_restore_and_match_old_format() {
        for name in [
            None,
            Some("abc"),
            Some("Рад"),
            Some("abcdefghijklmnopqrstuvwxyza"),
        ] {
            for desc in [
                None,
                Some("z"),
                Some("Рад, так Рад!"),
                Some("0123456789012345678901234567890123"),
            ] {
                let old = before::Text {
                    id: 42,
                    name: name.map(str::to_owned),
                    description: desc.map(str::to_owned),
                }
                .serialize_checked()
                .unwrap();
                let now = Text {
                    id: 42,
                    name: name.map(str::to_owned),
                    description: desc.map(str::to_owned),
                }
                .serialize_checked()
                .unwrap();
                assert_eq!(old, now);
                let x = Text::deserialize_checked(&old).unwrap();
                assert_eq!((x.name.as_deref(), x.description.as_deref()), (name, desc));
                let raw = TextDefault {
                    id: 42,
                    name: name.map(str::to_owned),
                    description: desc.map(str::to_owned),
                }
                .serialize_checked()
                .unwrap();
                let tag = TextTag {
                    id: 42,
                    name: name.map(str::to_owned),
                    description: desc.map(str::to_owned),
                }
                .serialize_checked()
                .unwrap();
                assert_eq!(raw, tag);
                let x = TextTag::deserialize_checked(&raw).unwrap();
                assert_eq!((x.name.as_deref(), x.description.as_deref()), (name, desc));
            }
        }
        assert_eq!(before::Text::SCHEMA_HASHES, Text::SCHEMA_HASHES);
    }
    #[test]
    fn fallback_directory_borrowed_empty_and_custom_fields() {
        for data in [None, Some(vec![]), Some(vec![1, 2, 3])] {
            for flag in [None, Some(7)] {
                let old = before::Fallback {
                    id: 5,
                    data: data.clone(),
                    flag,
                }
                .serialize_checked()
                .unwrap();
                let now = Fallback::try_new(5, data.clone(), flag)
                    .unwrap()
                    .serialize_checked()
                    .unwrap();
                assert_eq!(old, now);
                let x = Fallback::deserialize_checked(&old).unwrap();
                assert_eq!((x.data.as_ref(), x.flag()), (data.as_ref(), flag));
            }
        }
        for text in [None, Some(b"".as_slice()), Some("Рад".as_bytes())] {
            for flag in [None, Some(9)] {
                let bytes = BorrowedTag::try_new(7, text, flag)
                    .unwrap()
                    .serialize_checked()
                    .unwrap();
                let x = BorrowedTag::deserialize_checked(&bytes).unwrap();
                assert_eq!((x.text, x.flag()), (text, flag));
                if text != Some(b"") {
                    let bytes = BorrowedBoth::try_new(7, text, flag)
                        .unwrap()
                        .serialize_checked()
                        .unwrap();
                    let x = BorrowedBoth::deserialize_checked(&bytes).unwrap();
                    assert_eq!((x.text, x.flag()), (text, flag));
                }
            }
        }
        let empty = ZeroTag::try_new(3, Some([]), None)
            .unwrap()
            .serialize_checked()
            .unwrap();
        assert_eq!(
            ZeroTag::deserialize_checked(&empty).unwrap().empty,
            Some([])
        );
        let custom = CustomTag {
            required: Tiny(4),
            extra: Some(Tiny(8)),
        }
        .serialize_checked()
        .unwrap();
        let x = CustomTag::deserialize_checked(&custom).unwrap();
        assert_eq!((x.required, x.extra), (Tiny(4), Some(Tiny(8))));
        let b = EmptyBoth { id: 9 }.serialize_checked().unwrap();
        assert_eq!(EmptyBoth::deserialize_checked(&b).unwrap().id, 9);
        let x = EmptyTag::deserialize_checked(&7u32.to_le_bytes()).unwrap();
        assert_eq!(x.id, 7);
    }
    #[test]
    fn descriptor_dispatch_and_scalar_qps_remain_exact() {
        for a in [None, Some(7)] {
            for text in [None, Some("abc")] {
                let b = StateTag::try_new(5, a, text.map(str::to_owned))
                    .unwrap()
                    .serialize_checked()
                    .unwrap();
                let x = StateTag::deserialize_checked(&b).unwrap();
                assert_eq!((x.id, x.a(), x.text.as_deref()), (5, a, text));
            }
        }
        for a in [None, Some(7)] {
            for b in [None, Some(9)] {
                let bytes = PackedTag::try_new(3, a, b)
                    .unwrap()
                    .serialize_checked()
                    .unwrap();
                let x = PackedTag::deserialize_checked(&bytes).unwrap();
                assert_eq!((x.id, x.a(), x.b()), (3, a, b));
            }
        }
    }
    #[test]
    fn checked_dispatch_never_panics_on_corrupt_input() {
        for len in 0..96 {
            for seed in 0..64u8 {
                let bytes = (0..len)
                    .map(|i| (i as u8).wrapping_mul(37).wrapping_add(seed))
                    .collect::<Vec<_>>();
                let result = std::panic::catch_unwind(|| {
                    let _ = PairTag::deserialize_checked(&bytes);
                    let _ = TextTag::deserialize_checked(&bytes);
                    let _ = Pair::deserialize_checked(&bytes);
                    let _ = Text::deserialize_checked(&bytes);
                    let _ = BorrowedTag::deserialize_checked(&bytes);
                    let _ = LimitTag::deserialize_checked(&bytes);
                });
                assert!(result.is_ok(), "length={len}, seed={seed}");
            }
        }
        let good = LimitTag::try_new(4, Some(6), Some(7))
            .unwrap()
            .serialize_checked()
            .unwrap();
        let mut out = std::mem::MaybeUninit::new(LimitTag::try_new(1, None, None).unwrap());
        let mut bad = good.clone();
        bad[1] = 255;
        assert!(LimitTag::deserialize_checked_into(&bad, &mut out).is_err());
        let out = unsafe { out.assume_init() };
        assert_eq!((out.id, out.a(), out.b()), (1, None, None));
    }
}
