#![allow(unsafe_op_in_unsafe_fn)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Token(pub u16);
impl qsl::QslCustom for Token {
    const SIZE: usize = 2;
    fn qsl_validate(bytes: &[u8], _: bool) -> qsl::QslResult<()> {
        if bytes.len() == 2 {
            Ok(())
        } else {
            Err(qsl::QslError::InvalidValue)
        }
    }
    unsafe fn qsl_write_unchecked(&self, p: *mut u8) {
        unsafe { p.cast::<u16>().write_unaligned(self.0.to_le()) }
    }
    unsafe fn qsl_read_unchecked(p: *const u8) -> Self {
        Self(u16::from_le(unsafe { p.cast::<u16>().read_unaligned() }))
    }
}
include!(concat!(env!("OUT_DIR"), "/normal.rs"));
pub mod rt {
    include!(concat!(env!("OUT_DIR"), "/rt.rs"));
}
#[cfg(test)]
mod tests {
    use super::*;
    use qsl::{Serializable, qps_bytes::ByteQps};
    use std::mem::MaybeUninit;
    #[test]
    fn whole_records_restore_strings_arrays_and_numeric_fields() {
        let v = Whole {
            id: 42,
            text: "Рад, так Рад 🦀".repeat(32),
            data: (0..=255).collect(),
        };
        let mut raw = vec![0; 8192];
        let mut wire = vec![0; 8208];
        let mut restoration = vec![0; 8192];
        let f = v.serialize_qps_to(&mut raw, &mut wire).unwrap();
        let mut out = MaybeUninit::<Whole>::uninit();
        let x = Whole::deserialize_qps_into(f.as_slice(), &mut restoration, &mut out).unwrap();
        assert_eq!((x.id, &x.text, &x.data), (v.id, &v.text, &v.data));
        unsafe { out.assume_init_drop() };
        macro_rules! check {
            ($q:literal) => {{
                let f = ByteQps::<$q>::encode_record_to(&v, &mut raw, &mut wire).unwrap();
                let mut out = MaybeUninit::<Whole>::uninit();
                let x = f.decode_record_into(&mut restoration, &mut out).unwrap();
                assert_eq!((x.id, &x.text, &x.data), (v.id, &v.text, &v.data));
                unsafe { out.assume_init_drop() };
            }};
        }
        check!(2);
        check!(4);
        check!(8);
        check!(16);
    }
    #[test]
    fn borrowed_values_borrow_the_restoration_region() {
        let v = WholeBorrow {
            text: "UTF-8: 🦀",
            data: b"array",
        };
        let mut raw = [0; 128];
        let mut wire = [0; 144];
        let mut restored = [0; 128];
        let f = v.serialize_qps_to(&mut raw, &mut wire).unwrap();
        let x = WholeBorrow::deserialize_qps(f.as_slice(), &mut restored).unwrap();
        assert_eq!((x.text, x.data), (v.text, v.data));
    }
    #[test]
    fn optionals_and_formats_preserve_all_states() {
        for m in 0..8 {
            let v = OptionalDefault {
                id: 17,
                a: (m & 1 != 0).then_some(3),
                b: (m & 2 != 0).then_some(37),
                c: (m & 4 != 0).then_some(999),
            };
            let raw = v.serialize();
            let x = OptionalDefault::deserialize_checked(&raw).unwrap();
            assert_eq!((x.id, x.a, x.b, x.c), (v.id, v.a, v.b, v.c));
        }
    }
    #[test]
    fn byte_qps_validates_restored_schema_before_pointer_read() {
        let mut wire = [0; 256];
        let f = ByteQps::<16>::encode_to(&[0xff; 128], &mut wire).unwrap();
        let mut restored = [0; 128];
        let mut out = MaybeUninit::<Whole>::uninit();
        assert!(f.decode_record_into(&mut restored, &mut out).is_err());
    }
    #[test]
    fn group_macro_matches_manual_wire() {
        let a = MacroUnit {
            meta_id: 1,
            meta_tick: 2,
            code: 3,
        };
        let b = ManualUnit {
            meta_id: 1,
            meta_tick: 2,
            code: 3,
        };
        assert_eq!(a.serialize().as_ref(), b.serialize().as_ref());
    }
}
