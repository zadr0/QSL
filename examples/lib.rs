//! Executable QSL examples. Preparation helpers are outside benchmark timing.
pub mod models {
    include!("models.rs");
}
include!(concat!(env!("OUT_DIR"), "/normal.rs"));
pub mod rt {
    include!(concat!(env!("OUT_DIR"), "/rt.rs"));
}

pub const FIXTURES: usize = 1024;
pub fn texts() -> Vec<String> {
    (0..FIXTURES)
        .map(|i| {
            let mut text: Vec<_> = "Рад, так Рад! ".chars().cycle().take(1024).collect();
            for (j, c) in text.iter_mut().take(3).enumerate() {
                *c = char::from_u32(0x410 + ((i >> (j * 5)) & 31) as u32).unwrap();
            }
            text.into_iter().collect()
        })
        .collect()
}
/// Establish the constructor's capacity and UTF-8 preconditions in safe code.
pub fn inline<const N: usize>(text: &str) -> qsl::QslResult<qsl::InlineString<N>> {
    if N > u16::MAX as usize || text.len() > N {
        return Err(qsl::QslError::InvalidValue);
    }
    Ok(unsafe { qsl::InlineString::from_str_unchecked(text) })
}
pub fn numbers(seed: usize) -> Numbers {
    Numbers {
        f0: (seed & 15) as u64,
        f1: ((seed + 1) & 15) as u64,
        f2: ((seed + 2) & 15) as u64,
        f3: ((seed + 3) & 15) as u64,
        f4: ((seed + 4) & 15) as u64,
        f5: ((seed + 5) & 15) as u64,
        f6: ((seed + 6) & 15) as u64,
        f7: ((seed + 7) & 15) as u64,
    }
}
pub fn compact_numbers(seed: usize) -> CompactNumbers {
    CompactNumbers {
        f0: (seed & 15) as u64,
        f1: ((seed + 1) & 15) as u64,
        f2: ((seed + 2) & 15) as u64,
        f3: ((seed + 3) & 15) as u64,
        f4: ((seed + 4) & 15) as u64,
        f5: ((seed + 5) & 15) as u64,
        f6: ((seed + 6) & 15) as u64,
        f7: ((seed + 7) & 15) as u64,
    }
}
pub fn packed_numbers(seed: usize) -> PackedNumbers {
    PackedNumbers {
        f0: (seed & 15) as u64,
        f1: ((seed + 1) & 15) as u64,
        f2: ((seed + 2) & 15) as u64,
        f3: ((seed + 3) & 15) as u64,
        f4: ((seed + 4) & 15) as u64,
        f5: ((seed + 5) & 15) as u64,
        f6: ((seed + 6) & 15) as u64,
        f7: ((seed + 7) & 15) as u64,
    }
}
pub fn rt_numbers(seed: usize) -> rt::RtNumbers {
    rt::RtNumbers {
        f0: (seed & 15) as u64,
        f1: ((seed + 1) & 15) as u64,
        f2: ((seed + 2) & 15) as u64,
        f3: ((seed + 3) & 15) as u64,
        f4: ((seed + 4) & 15) as u64,
        f5: ((seed + 5) & 15) as u64,
        f6: ((seed + 6) & 15) as u64,
        f7: ((seed + 7) & 15) as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_compact_qps_and_rt_are_exact() {
        for i in 0..16 {
            let a = numbers(i);
            let x = Numbers::deserialize_checked(&a.serialize_checked().unwrap()).unwrap();
            assert_eq!((x.f0, x.f7), (a.f0, a.f7));
            let a = compact_numbers(i);
            let x = CompactNumbers::deserialize_checked(&a.serialize_checked().unwrap()).unwrap();
            assert_eq!((x.f0, x.f7), (a.f0, a.f7));
            let a = packed_numbers(i);
            let x = PackedNumbers::deserialize_checked(&a.serialize_checked().unwrap()).unwrap();
            assert_eq!((x.f0, x.f7), (a.f0, a.f7));
            let lease = rt_numbers(i).serialize().unwrap();
            assert_eq!(lease.as_ref(), numbers(i).serialize_checked().unwrap());
        }
    }
    #[test]
    fn every_text_mode_restores_the_same_utf8() {
        let text = texts().remove(0);
        assert_eq!(text.chars().count(), 1024);
        let a = OwnedText { text: text.clone() };
        let wire = a.serialize_checked().unwrap();
        assert_eq!(OwnedText::deserialize_checked(&wire).unwrap().text, text);
        let a = BorrowedText { text: &text };
        assert_eq!(a.serialize_checked().unwrap(), wire);
        assert_eq!(BorrowedText::deserialize_checked(&wire).unwrap().text, text);
        let a = OwnedReuseText { text: text.clone() };
        assert_eq!(a.serialize_checked().unwrap(), wire);
        assert_eq!(
            OwnedReuseText::deserialize_checked(&wire).unwrap().text.as_str(),
            text
        );
        let a = InlineText { text: text.clone() };
        assert_eq!(
            InlineText::deserialize_checked(&a.serialize_checked().unwrap())
                .unwrap()
                .text
                .as_str(),
            text
        );
        let a = CompressedText { text: text.clone() };
        let compressed = a.serialize_checked().unwrap();
        assert!(compressed.len() < wire.len());
        assert_eq!(
            CompressedText::deserialize_checked(&compressed)
                .unwrap()
                .text
                .as_str(),
            text
        );
    }
    #[test]
    fn scalar_prefix_and_error_checks() {
        let value = MixedMessage {
            a: 3,
            b: 7,
            text: "A".repeat(1024),
        };
        let bytes = value.serialize_checked().unwrap();
        let decoded = MixedMessage::deserialize_checked(&bytes).unwrap();
        assert_eq!((decoded.a, decoded.b, decoded.text), (3, 7, value.text));
        assert_eq!(&bytes[1..5], &1024u32.to_le_bytes());
        assert!(Bounded::deserialize_checked(&[9, 1, 0, 0, 0, b'a']).is_err());
        assert!(Bounded::deserialize_checked(&[1, 255, 255, 255, 255]).is_err());
    }
}

/// A separate, compressible control: 1024 two-byte Cyrillic characters.
pub fn uniform_texts() -> Vec<String> {
    (0..FIXTURES)
        .map(|i| {
            char::from_u32(0x410 + (i & 31) as u32)
                .unwrap()
                .to_string()
                .repeat(1024)
        })
        .collect()
}
#[test]
fn uniform_utf8_really_uses_lz_instead_of_a_shorter_raw_header() {
    for text in uniform_texts().iter().take(32) {
        let value = CompressedText { text: text.clone() };
        let bytes = value.serialize_checked().unwrap();
        assert!(
            bytes.len() < text.len() / 2,
            "LZ unexpectedly used raw fallback"
        );
        assert_eq!(
            CompressedText::deserialize_checked(&bytes)
                .unwrap()
                .text
                .as_str(),
            text
        );
    }
}
