//! Compatibility path; portable implementation is owned by qsl-core.
pub use qsl_core::modernization::*;

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{QslSchema, QslType};
    #[test]
    fn arrays_use_native_data_bytes_for_the_strict_limit() {
        let schema = QslSchema {
            name: "A".into(),
            max_tag: 1,
            fields: vec![crate::QslField {
                name: "values".into(),
                tag: 1,
                data: QslType::FixedArray(Box::new(QslType::Flags(17)), 2),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(
            lower_storage(&schema, 6).unwrap().fields[0].storage,
            StorageKind::Default
        );
        assert_eq!(
            lower_storage(&schema, 8).unwrap().fields[0].storage,
            StorageKind::FixedArray(2)
        );
        let mut native = schema;
        native.options.transmutate = true;
        assert!(check_storage_limit(&native, 6).is_err());
    }
    #[test]
    fn optional_accessor_collisions_are_rejected() {
        for source in ["&A{u8? NATIVE_BYTES;}", "&A{u8? a;u8? set_a;}"] {
            assert!(crate::parse_qsl(source).is_err(), "{source}");
        }
    }
    #[test]
    fn generated_registers_match_every_exact_window_width() {
        for bytes in 1..=16 {
            let fields = (0..bytes).map(|i| format!("u8 f{i};")).collect::<String>();
            let schema = crate::parse_qsl(&format!("&Window[safety]{{{fields}}}")).unwrap();
            let expected = register_word(bytes).unwrap();
            let mut generator = crate::QslInterpreter::new();
            generator.generate(&schema).unwrap();
            let output = String::from_utf8(generator.output).unwrap();
            assert!(
                output.contains(&format!("__qsl_register_0_0:{expected}=")),
                "{bytes}: {expected}"
            );
            assert!(
                output.contains(&format!("load_{expected}::<{bytes}>")),
                "{bytes}: {expected}"
            );
            assert!(!output.contains("::load_exact::"));
            let groups = register_groups(&schema[0]).unwrap();
            assert_eq!(groups[0].register_bits, bytes.next_power_of_two() * 8);
        }
        assert!(register_word(17).is_err());
    }
    #[test]
    fn typed_loads_zero_extend_partial_unaligned_windows() {
        let bytes = [0xA5, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let source = unsafe { bytes.as_ptr().add(1) };
        macro_rules! check {
            ($load:ident,$ty:ty,$n:literal) => {
                let result: $ty = unsafe { $load::<$n>(source) };
                let mut expected = [0u8; 16];
                expected[..$n].copy_from_slice(&bytes[1..1 + $n]);
                assert_eq!(result as u128, u128::from_le_bytes(expected));
            };
        }
        check!(load_u8, u8, 0);
        check!(load_u8, u8, 1);
        check!(load_u16, u16, 2);
        check!(load_u32, u32, 3);
        check!(load_u32, u32, 4);
        check!(load_u64, u64, 5);
        check!(load_u64, u64, 6);
        check!(load_u64, u64, 7);
        check!(load_u64, u64, 8);
        check!(load_u128, u128, 9);
        check!(load_u128, u128, 10);
        check!(load_u128, u128, 11);
        check!(load_u128, u128, 12);
        check!(load_u128, u128, 13);
        check!(load_u128, u128, 14);
        check!(load_u128, u128, 15);
        check!(load_u128, u128, 16);
    }
    #[test]
    fn exact_partial_register() {
        let b = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(unsafe { load_exact::<9>(b.as_ptr()) }, 0x090807060504030201);
    }
    #[test]
    fn drops_only_initialized() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        struct D(Arc<AtomicUsize>);
        impl Drop for D {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
        let n = Arc::new(AtomicUsize::new(0));
        {
            let mut a = InlineArray::<D, 256>::new();
            a.push(D(n.clone())).unwrap();
            a.push(D(n.clone())).unwrap();
        }
        assert_eq!(n.load(Ordering::Relaxed), 2);
    }
}
