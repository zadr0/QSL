#![allow(dead_code, unused_variables)]
use std::sync::atomic::{AtomicUsize, Ordering};
static DECODE_CALLS: AtomicUsize = AtomicUsize::new(0);
pub struct Counted(u8);
impl qsl::QslCustom for Counted {
    const SIZE: usize = 1;
    fn qsl_validate(bytes: &[u8], _: bool) -> qsl::QslResult<()> {
        if bytes.len() == 1 {
            Ok(())
        } else {
            Err(qsl::QslError::UnexpectedEof)
        }
    }
    fn qsl_validate_values(&self) -> qsl::QslResult<()> {
        Ok(())
    }
    unsafe fn qsl_write_unchecked(&self, to: *mut u8) {
        to.write(self.0);
    }
    unsafe fn qsl_read_unchecked(from: *const u8) -> Self {
        DECODE_CALLS.fetch_add(1, Ordering::SeqCst);
        Self(from.read())
    }
}
mod normal {
    use super::Counted;
    include!(concat!(env!("OUT_DIR"), "/normal.rs"));
}
mod small {
    use super::Counted;
    include!(concat!(env!("OUT_DIR"), "/small.rs"));
}
mod tiny {
    use super::Counted;
    include!(concat!(env!("OUT_DIR"), "/tiny.rs"));
}
#[qsl::qsl_macro]
struct AttributeOne {
    value: u8,
}
#[qsl::schem_macro]
struct AttributeTwo {
    value: u8,
}

#[test]
fn checked_api_without_safety_attribute_roundtrips_all_layouts() {
    let value = normal::Plain {
        id: 42,
        text: "hello".into(),
        bytes: vec![1, 2, 3],
        flag: Some(7),
    };
    let bytes = value.serialize_checked().unwrap();
    let result = normal::Plain::deserialize_checked(&bytes).unwrap();
    assert_eq!(result.text, "hello");
    assert_eq!(result.bytes, vec![1, 2, 3]);
    let mut storage = [0u8; 128];
    let n = value.serialize_checked_to(&mut storage).unwrap();
    assert_eq!(&storage[..n], bytes);
    let borrowed = normal::Borrowed {
        text: "hello",
        bytes: &[1, 2, 3],
    };
    let bytes = borrowed.serialize_checked().unwrap();
    let mut out = std::mem::MaybeUninit::uninit();
    let result = normal::Borrowed::deserialize_checked_into(&bytes, &mut out).unwrap();
    assert_eq!(result.text, "hello");
    assert!(result.text.as_ptr() >= bytes.as_ptr());
    let c = normal::Compact {
        tag: 2,
        text: "abc".into(),
        bytes: vec![3, 4],
        flag: Some(9),
    };
    let b = c.serialize_checked().unwrap();
    assert_eq!(
        normal::Compact::deserialize_checked(&b).unwrap().bytes,
        vec![3, 4]
    );
    let fixed = normal::Fixed {
        text: *b"rust",
        bytes: vec![1, 2, 3],
    };
    assert!(normal::Fixed::deserialize_checked(&fixed.serialize_checked().unwrap()).is_ok());
    let optional = normal::Optional {
        id: 4,
        text: Some("abc".into()),
        flag: Some(7),
    };
    assert!(normal::Optional::deserialize_checked(&optional.serialize_checked().unwrap()).is_ok());
    let sequential = normal::Sequential { id: 3, flag: None };
    assert!(
        normal::Sequential::deserialize_checked(&sequential.serialize_checked().unwrap()).is_ok()
    );
    let state = normal::State {
        first: Some(1),
        second: Some(3),
    };
    assert!(normal::State::deserialize_checked(&state.serialize_checked().unwrap()).is_ok());
    let directory = normal::Directory {
        id: 2,
        first: Some(1),
        second: Some(3),
    };
    assert!(
        normal::Directory::deserialize_checked(&directory.serialize_checked().unwrap()).is_ok()
    );
    let packed = normal::Packed { a: 3, b: 5 };
    assert!(normal::Packed::deserialize_checked(&packed.serialize_checked().unwrap()).is_ok());
}
#[test]
fn bad_lengths_frames_and_utf8_are_errors_before_constructing_values() {
    use qsl::QslError::*;
    let bad = u32::MAX.to_le_bytes();
    assert_eq!(normal::Text::validate(&bad), Err(SizeLimitExceeded));
    assert_eq!(normal::Bytes::validate(&bad), Err(SizeLimitExceeded));
    assert_eq!(small::Text::validate(&[0; 65]), Err(SizeLimitExceeded));
    assert_eq!(normal::Text::validate(&[1, 0, 0, 0, 255]), Err(InvalidUtf8));
    assert!(normal::Text::validate(&[3, 0, 0, 0, b'a']).is_err());
    assert!(normal::Text::validate(&[0, 0, 0, 0, 0]).is_err());
    DECODE_CALLS.store(0, Ordering::SeqCst);
    let mut bad = vec![7];
    bad.extend_from_slice(&u32::MAX.to_le_bytes());
    assert!(normal::Custom::deserialize_checked(&bad).is_err());
    assert_eq!(DECODE_CALLS.load(Ordering::SeqCst), 0);
}
#[test]
fn decompressed_values_share_one_budget_instead_of_per_field_limits() {
    let text = "A".repeat(48);
    let value = normal::Compressed {
        first: unsafe { qsl::InlineString::from_str_unchecked(&text) },
        second: unsafe { qsl::InlineString::from_str_unchecked(&text) },
    };
    let bytes = value.serialize_checked().unwrap();
    assert!(bytes.len() < 64);
    assert_eq!(
        small::Compressed::validate(&bytes),
        Err(qsl::QslError::SizeLimitExceeded)
    );
    let out = small::Compressed {
        first: unsafe { qsl::InlineString::from_str_unchecked(&text) },
        second: unsafe { qsl::InlineString::from_str_unchecked(&text) },
    };
    assert!(matches!(
        out.serialize_checked(),
        Err(qsl::QslError::SizeLimitExceeded)
    ));
    let value = normal::Packed { a: 3, b: 5 };
    let bytes = value.serialize_checked().unwrap();
    assert_eq!(bytes.len(), 1);
    assert_eq!(
        tiny::Packed::validate(&bytes),
        Err(qsl::QslError::SizeLimitExceeded)
    );
}
#[test]
fn malicious_qps_declared_size_is_rejected_before_restoration() {
    let mut frame = [0u8; 17];
    frame[..4].copy_from_slice(b"QPB3");
    frame[4] = 16;
    frame[5] = 1;
    frame[8..16].copy_from_slice(&(1u64 << 32).to_le_bytes());
    let mut restoration = [0xA5; 128];
    let mut out = std::mem::MaybeUninit::uninit();
    assert!(matches!(
        normal::Packed::deserialize_qps_into(&frame, &mut restoration, &mut out),
        Err(qsl::QslError::SizeLimitExceeded)
    ));
    assert_eq!(restoration, [0xA5; 128]);
    let value = normal::Packed { a: 3, b: 5 };
    let mut scratch = [0; 64];
    let mut wire = [0; 64];
    let frame = value.serialize_qps_to(&mut scratch, &mut wire).unwrap();
    assert!(
        normal::Packed::deserialize_qps_into(frame.as_slice(), &mut restoration, &mut out).is_ok()
    );
}
#[test]
fn damaged_frames_never_panic_across_builtin_modes() {
    for length in 0..100 {
        for seed in [0u8, 1, 7, 127, 128, 255] {
            let data = vec![seed; length];
            for validate in [
                normal::Plain::validate,
                normal::Text::validate,
                normal::Bytes::validate,
                normal::Fixed::validate,
                normal::Compact::validate,
                normal::Compressed::validate,
                normal::Guard::validate,
                normal::Optional::validate,
                normal::Sequential::validate,
                normal::State::validate,
                normal::Directory::validate,
                normal::Packed::validate,
            ] {
                assert!(std::panic::catch_unwind(|| validate(&data)).is_ok());
            }
        }
    }
}
#[test]
fn value_validation_prevents_invalid_fixed_buffers_and_preserves_identity() {
    let bad = normal::Fixed {
        text: *b"rust",
        bytes: vec![1],
    };
    let mut out = [0xA5; 32];
    assert!(bad.serialize_checked_to(&mut out).is_err());
    assert_eq!(out, [0xA5; 32]);
    assert_eq!(normal::Plain::SCHEMA_HASHES, small::Plain::SCHEMA_HASHES);
    assert_eq!(normal::Plain::MAX_DATA_SIZE, 65535);
    assert_eq!(small::Plain::MAX_DATA_SIZE, 64);
    assert_eq!(AttributeOne { value: 1 }.value, 1);
    assert_eq!(AttributeTwo { value: 2 }.value, 2);
}
