#![allow(non_snake_case, dead_code)]
#[cfg(test)]
use contracts::*;
pub mod benchmark_types {
    include!(concat!(env!("OUT_DIR"), "/benchmark_types.rs"));
}
#[cfg(test)]
mod contracts {
    mod primitive {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/primitive.rs"));
    }
    pub use primitive::DeviceId;
    qsl_lexer::qsl_native_primitive!(DeviceId, u32);
    mod native_macro {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/native_macro.rs"));
    }
    pub use native_macro::SharedMeta;
    include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
    #[cfg(test)]
    mod tests {
        use super::*;
        include!(concat!(env!("OUT_DIR"), "/boundary_tests.rs"));
        #[test]
        fn bitmap_wire_order_and_spare_bits() {
            let value = PredictBitmap::try_new(9, Some(0x1234), Some(7)).unwrap();
            let bytes = value.serialize_checked().unwrap();
            assert_eq!(bytes, [0xc0, 9, 7, 0x34, 0x12]);
            for bit in 0..6 {
                let mut invalid = bytes.clone();
                invalid[0] |= 1 << bit;
                assert!(PredictBitmap::deserialize_checked(&invalid).is_err());
            }
            let mut out = std::mem::MaybeUninit::<PredictBitmap>::uninit();
            assert!(PredictBitmap::deserialize_checked_into(&[1, 9], &mut out).is_err());
        }
        #[test]
        fn bitmap_compact_and_borrowed_fields() {
            let value = Bits::try_new(1, Some(7), Some("abc".into())).unwrap();
            let bytes = value.serialize_checked().unwrap();
            let restored = Bits::deserialize_checked(&bytes).unwrap();
            assert_eq!(restored.a(), Some(7));
            assert_eq!(restored.b.as_deref(), Some("abc"));
            let raw = unsafe { <Bits as qsl::Serializable>::deserialize(&bytes) };
            assert_eq!(raw.a(), Some(7));
            assert_eq!(raw.b, restored.b);
            let value = Borrow {
                id: 1,
                a: Some("one"),
                b: Some("two"),
            };
            let bytes = value.serialize_checked().unwrap();
            let restored = Borrow::deserialize_checked(&bytes).unwrap();
            assert_eq!(restored.a, Some("one"));
            assert_eq!(restored.b, Some("two"));
        }
        #[test]
        fn table_preserves_the_ordinary_tag_wire() {
            for mask in 0..8 {
                let a = (mask & 1 != 0).then_some(3);
                let b = (mask & 2 != 0).then_some(String::from("abc"));
                let c = (mask & 4 != 0).then_some(0x1234);
                let raw = Raw::try_new(42, a, b.clone(), c).unwrap();
                let table = Table::try_new(42, a, b, c).unwrap();
                let bytes = raw.serialize_checked().unwrap();
                assert_eq!(bytes, table.serialize_checked().unwrap());
                let decoded = Table::deserialize_checked(&bytes).unwrap();
                assert_eq!(decoded.a(), raw.a());
                assert_eq!(decoded.b, raw.b);
                assert_eq!(decoded.c(), raw.c());
            }
        }
    }

    #[cfg(test)]
    mod native_tests {
        use super::*;
        #[test]
        fn regions_are_real_nested_fields_and_preserve_wire() {
            let value = Parent {
                Meta: Meta {
                    id: 1,
                    value: 2,
                    tag: *b"test",
                },
                name: "name".into(),
            };
            let bytes = value.serialize_checked().unwrap();
            assert_eq!(&bytes[..8], &1u64.to_le_bytes());
            assert_eq!(&bytes[8..12], &2u32.to_le_bytes());
            assert_eq!(&bytes[12..16], b"test");
            let decoded = Parent::deserialize_checked(&bytes).unwrap();
            assert_eq!(decoded.Meta.tag, *b"test");
            assert_eq!(decoded.name, "name");
            assert_eq!(std::mem::size_of::<Meta>(), 16);
            let value = NestedParent {
                Outer: Outer {
                    lead: 1,
                    Inner: Inner { a: 2, b: 3 },
                    tail: 4,
                },
                marker: 5,
            };
            let bytes = value.serialize_checked().unwrap();
            let decoded = NestedParent::deserialize_checked(&bytes).unwrap();
            assert_eq!(decoded.Outer.Inner.b, 3);
            assert_eq!(bytes.len(), 25);
        }
        #[test]
        fn native_borrow_validates_layout_alignment_and_utf8_before_view() {
            #[repr(align(8))]
            struct Aligned([u8; 64]);
            let meta = ViewMeta {
                id: 1,
                value: 2,
                tag: *b"test",
            };
            let value = NativeBorrow {
                ViewMeta: &meta,
                name: "abc",
            };
            let bytes = value.serialize_checked().unwrap();
            let mut buffer = Aligned([0; 64]);
            buffer.0[..bytes.len()].copy_from_slice(&bytes);
            let decoded = NativeBorrow::deserialize_checked(&buffer.0[..bytes.len()]).unwrap();
            assert_eq!(
                decoded.ViewMeta as *const ViewMeta as *const u8,
                buffer.0.as_ptr()
            );
            buffer.0[12] = 0xff;
            assert!(NativeBorrow::deserialize_checked(&buffer.0[..bytes.len()]).is_err());
            let mut other = Aligned([0; 64]);
            other.0[1..bytes.len() + 1].copy_from_slice(&bytes);
            assert!(NativeBorrow::deserialize_checked(&other.0[1..bytes.len() + 1]).is_err());
        }
    }

    pub struct Envelope(pub Child);
    impl qsl::QslCustom for Envelope {
        const SIZE: usize = 7;
        fn qsl_validate(bytes: &[u8], compact: bool) -> qsl::QslResult<()> {
            if compact || bytes.len() != 7 {
                return Err(qsl::QslError::InvalidValue);
            }
            let n = bytes[0] as usize;
            if n > 6 || bytes[1 + n..].iter().any(|b| *b != 0) {
                return Err(qsl::QslError::InvalidValue);
            }
            Child::validate(&bytes[1..1 + n])
        }
        fn qsl_validate_values(&self) -> qsl::QslResult<()> {
            self.0.validate_values()
        }
        unsafe fn qsl_write_unchecked(&self, to: *mut u8) {
            std::ptr::write_bytes(to, 0, 7);
            let n = <Child as qsl::Serializable>::serialize_into_unsafe(&self.0, to.add(1));
            to.write(n as u8);
        }
        unsafe fn qsl_read_unchecked(from: *const u8) -> Self {
            let n = from.read() as usize;
            Self(<Child as qsl::Serializable>::deserialize_from_raw_parts(
                from.add(1),
                n,
            ))
        }
    }
    #[cfg(test)]
    mod primitive_tests {
        use super::*;
        #[test]
        fn registered_primitive_keeps_its_validated_custom_codec() {
            let value = Poly::try_new(DeviceId(42), Some(DeviceId(3)), Some(7)).unwrap();
            let bytes = value.serialize_checked().unwrap();
            let decoded = Poly::deserialize_checked(&bytes).unwrap();
            assert_eq!(decoded.id, DeviceId(42));
            assert_eq!(decoded.a, Some(DeviceId(3)));
            let mut invalid = bytes.clone();
            invalid[1..5].fill(0);
            assert!(Poly::deserialize_checked(&invalid).is_err());
            assert!(Poly::try_new(DeviceId(0), None, None).is_err());
            for mask in 0..4 {
                let a = (mask & 1 != 0).then_some(DeviceId(3));
                let tail = (mask & 2 != 0).then_some(7);
                let value = PolyPS::try_new(1, a, tail).unwrap();
                let bytes = value.serialize_checked().unwrap();
                let decoded = PolyPS::deserialize_checked(&bytes).unwrap();
                assert_eq!(decoded.a, a);
                assert_eq!(decoded.tail(), tail);
            }
        }
        #[test]
        fn native_primitive_macro_checks_region_values() {
            let value = PolyNative {
                PrimitiveRegion: PrimitiveRegion {
                    id: DeviceId(7),
                    count: 2,
                },
                code: 3,
            };
            let bytes = value.serialize_checked().unwrap();
            assert_eq!(bytes.len(), 9);
            assert_eq!(
                PolyNative::deserialize_checked(&bytes)
                    .unwrap()
                    .PrimitiveRegion
                    .id,
                DeviceId(7)
            );
            let mut invalid = bytes;
            invalid[..4].fill(0);
            assert!(PolyNative::deserialize_checked(&invalid).is_err());
        }
        #[test]
        fn custom_optional_scopes_do_not_share_marker_indices() {
            for inner in 0..4 {
                let child = Child::try_new(
                    4,
                    (inner & 1 != 0).then_some(11),
                    (inner & 2 != 0).then_some(22),
                )
                .unwrap();
                let value = Scoped::try_new(8, Some(Envelope(child)), Some(9)).unwrap();
                let bytes = value.serialize_checked().unwrap();
                let decoded = Scoped::deserialize_checked(&bytes).unwrap();
                let nested = decoded.nested.as_ref().unwrap();
                assert_eq!(nested.0.a(), (inner & 1 != 0).then_some(11));
                assert_eq!(nested.0.b(), (inner & 2 != 0).then_some(22));
                assert_eq!(decoded.marker(), Some(9));
                let mut invalid = bytes;
                invalid[4] |= 1;
                assert!(Scoped::deserialize_checked(&invalid).is_err());
            }
        }
    }

    #[cfg(test)]
    mod macro_tests {
        use super::*;
        #[test]
        fn native_rust_macro_reuses_its_type_without_flattening_parent() {
            let value = MacroParent {
                data: SharedMeta { id: 7, value: 8 },
                marker: 9,
            };
            let bytes = value.serialize_checked().unwrap();
            assert_eq!(bytes.len(), 17);
            let decoded = MacroParent::deserialize_checked(&bytes).unwrap();
            assert_eq!(decoded.data.id, 7);
            assert_eq!(decoded.data.value, 8);
        }
    }
}
#[test]
fn arrays_and_lz_use_the_same_dynamic_optional_dispatch() {
    for mask in 0..4 {
        let values = (mask & 1 != 0).then_some(vec![u64::MAX, 17]);
        let tail = (mask & 2 != 0).then_some(3);
        let value = Arrays::try_new(1, values.clone(), tail).unwrap();
        let bytes = value.serialize_checked().unwrap();
        let decoded = Arrays::deserialize_checked(&bytes).unwrap();
        assert_eq!(decoded.values, values);
        assert_eq!(decoded.tail(), tail);
        let raw = unsafe { <Arrays as qsl::Serializable>::deserialize(&bytes) };
        assert_eq!(raw.values, values);
    }
    let text = qsl::InlineString::<128>::spaces::<64>();
    let value = Lz::try_new(1, Some(text), Some(7)).unwrap();
    let bytes = value.serialize_checked().unwrap();
    let decoded = Lz::deserialize_checked(&bytes).unwrap();
    assert_eq!(decoded.text.unwrap().as_str().as_bytes(), &[b' '; 64]);
    let value = ArrayParent {
        ArrayRegion: ArrayRegion { bytes: [7; 100] },
        tail: 9,
    };
    let bytes = value.serialize_checked().unwrap();
    let decoded = ArrayParent::deserialize_checked(&bytes).unwrap();
    assert_eq!(decoded.ArrayRegion.bytes, [7; 100]);
}
#[test]
fn malformed_frames_do_not_panic_or_publish_destination_values() {
    let mut seed = 0x12345678u64;
    for length in 0..128 {
        let mut bytes = vec![0u8; length];
        for b in &mut bytes {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            *b = seed as u8;
        }
        let result = std::panic::catch_unwind(|| {
            let _ = Bm128Table::deserialize_checked(&bytes);
            let _ = Arrays::deserialize_checked(&bytes);
            let _ = Lz::deserialize_checked(&bytes);
            let _ = Scoped::deserialize_checked(&bytes);
        });
        assert!(result.is_ok());
    }
    let mut destination = std::mem::MaybeUninit::<PredictBitmap>::uninit();
    let size = std::mem::size_of::<PredictBitmap>();
    unsafe {
        std::ptr::write_bytes(destination.as_mut_ptr().cast::<u8>(), 0xa5, size);
    }
    assert!(PredictBitmap::deserialize_checked_into(&[1, 9], &mut destination).is_err());
    let untouched = unsafe { std::slice::from_raw_parts(destination.as_ptr().cast::<u8>(), size) };
    assert!(untouched.iter().all(|b| *b == 0xa5));
}

#[cfg(test)]
mod compact_primitive_tests {
    use super::contracts::*;
    #[test]
    fn opt_in_macro_compresses_rank_and_rejects_invalid_codes() {
        let value = PolyCompact {
            id: DeviceId(42),
            a: Some(DeviceId(7)),
        };
        let bytes = value.serialize_checked().unwrap();
        assert_eq!(bytes, [1, 41, 6]);
        let decoded = PolyCompact::deserialize_checked(&bytes).unwrap();
        assert_eq!(decoded.id, DeviceId(42));
        assert_eq!(decoded.a, Some(DeviceId(7)));
        assert!(PolyCompact::deserialize_checked(&[1, 255, 6]).is_err());
    }
}

#[cfg(test)]
mod decoded_budget_tests {
    use super::contracts::DeviceId;
    mod limited {
        use super::DeviceId;
        include!(concat!(env!("OUT_DIR"), "/budget_types.rs"));
    }
    #[test]
    fn compact_primitive_cannot_hide_its_original_data_width() {
        assert!(matches!(
            limited::BudgetCompact::deserialize_checked(&[41]),
            Err(qsl::QslError::SizeLimitExceeded)
        ));
        let value = limited::BudgetCompact { id: DeviceId(42) };
        assert!(matches!(
            value.serialize_checked(),
            Err(qsl::QslError::SizeLimitExceeded)
        ));
    }
}

#[test]
fn custom_arrays_use_the_codec_compact_width_and_validate_each_value() {
    let value = PolyArray {
        ids: vec![DeviceId(1), DeviceId(100)],
        extras: Some(vec![DeviceId(42), DeviceId(7)]),
    };
    let bytes = value.serialize_checked().unwrap();
    let decoded = PolyArray::deserialize_checked(&bytes).unwrap();
    assert_eq!(decoded.ids, value.ids);
    assert_eq!(decoded.extras, value.extras);
    assert!(bytes.len() < 16);
    let invalid = PolyArray {
        ids: vec![DeviceId(0), DeviceId(1)],
        extras: None,
    };
    assert!(invalid.serialize_checked().is_err());
    for length in 0..bytes.len() {
        assert!(PolyArray::deserialize_checked(&bytes[..length]).is_err());
    }
    let mut corrupt = bytes;
    corrupt[1] = 255;
    assert!(PolyArray::deserialize_checked(&corrupt).is_err());
}
