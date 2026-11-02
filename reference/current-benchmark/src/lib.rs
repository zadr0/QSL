#![allow(unsafe_op_in_unsafe_fn)]
include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
#[cfg(test)]
mod tests {
    use super::*;
    use qsl::Serializable;
    #[test]
    fn qps_required_quantums_and_constant_frames() {
        let v = Standalone { a: 3, b: 15 };
        let wire = Serializable::serialize(&v);
        assert_eq!(&wire[..], &[0xf3]);
        let x = Standalone::deserialize_checked(&wire).unwrap();
        assert_eq!((x.a, x.b), (3, 15));
        macro_rules! check {
            ($ty:ident,$golden:expr) => {{
                let v = $ty {
                    f0: 0,
                    f1: 1,
                    f2: 2,
                    f3: 3,
                    f4: 4,
                    f5: 5,
                    f6: 6,
                    f7: 15,
                };
                let wire = Serializable::serialize(&v);
                assert_eq!(&wire[..], &$golden);
                let x = $ty::deserialize_checked(&wire).unwrap();
                assert_eq!(
                    (x.f0, x.f1, x.f2, x.f3, x.f4, x.f5, x.f6, x.f7),
                    (0, 1, 2, 3, 4, 5, 6, 15)
                );
            }};
        }
        check!(Q2, [0x80, 0x91, 0xa2, 0xf3]);
        check!(Q4, [0x40, 0xc8, 0x51, 0xf9]);
        check!(Q8, [0x20, 0x64, 0xa8, 0xfc]);
        check!(Q16, [0x10, 0x32, 0x54, 0xf6]);
    }
    #[test]
    fn qps_macro_can_have_no_source_level_optional_marker() {
        for m in 0u8..8 {
            let v = MacroPredict {
                id: 42,
                extras_a: (m & 1 != 0).then_some(255),
                extras_b: (m & 2 != 0).then_some(65535),
                extras_c: (m & 4 != 0).then_some(u32::MAX),
            };
            let wire = Serializable::serialize(&v);
            let x = MacroPredict::deserialize_checked(&wire).unwrap();
            assert_eq!(
                (x.id, x.extras_a, x.extras_b, x.extras_c),
                (v.id, v.extras_a, v.extras_b, v.extras_c)
            );
        }
    }
    #[test]
    fn optional_values_and_quant_alias_are_exact() {
        for m in 0u8..16 {
            let v = Qps {
                base: -7,
                a: 13,
                b: (m & 1 != 0).then_some(999),
                c: (m & 2 != 0).then_some(-37),
                constant: (m & 4 != 0).then_some(7),
                wide: (m & 8 != 0).then_some(u64::MAX),
            };
            let alias = Alias {
                base: v.base,
                a: v.a,
                b: v.b,
                c: v.c,
                constant: v.constant,
                wide: v.wide,
            };
            let wire = Serializable::serialize(&v);
            assert_eq!(&wire[..], &Serializable::serialize(&alias)[..]);
            let x = Qps::deserialize_checked(&wire).unwrap();
            assert_eq!(
                (x.base, x.a, x.b, x.c, x.constant, x.wide),
                (v.base, v.a, v.b, v.c, v.constant, v.wide)
            );
            for len in 0..wire.len() {
                assert!(Qps::validate(&wire[..len]).is_err());
            }
        }
    }
}
