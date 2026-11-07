//! Compatibility path; implementation is owned by qsl-core.

#[cfg(test)]
mod tests {

    use crate::{QslCustom, QslPrimitive, QslSchemaBuilder};
    crate::qsl_primitive!(pub struct DeviceId(u32);range=(1,100););
    crate::qsl_primitive!(pub struct SignedId(i16);range=(-10,10););
    #[test]
    fn wrapper_validation_and_safe_construction() {
        assert!(DeviceId(0).qsl_validate_values().is_err());
        assert!(DeviceId(100).qsl_validate_values().is_ok());
        assert_eq!(DeviceId::qsl_from_bits(42), DeviceId(42));
        assert_eq!(SignedId::qsl_from_bits((-7i16) as u64), SignedId(-7));
        assert!(DeviceId::qsl_validate(&101u32.to_le_bytes(), false).is_err());
    }
    crate::qsl_primitive!(pub struct ConstantId(u8);range=(7,7););
    crate::qsl_primitive!(pub struct FullWidth(u64);range=(0,u64::MAX););
    #[test]
    fn compact_macro_preserves_domain_endpoints_and_zero_width_values() {
        assert_eq!(ConstantId::COMP_SIZE, 0);
        assert!(ConstantId::qsl_validate(&[], true).is_ok());
        let mut bytes = [0u8; 8];
        for value in [-10i16, 0, 10] {
            let original = SignedId(value);
            unsafe {
                original.qsl_comp_write_unchecked(bytes.as_mut_ptr());
            }
            assert!(SignedId::qsl_validate(&bytes[..SignedId::COMP_SIZE], true).is_ok());
            assert_eq!(
                unsafe { SignedId::qsl_comp_read_unchecked(bytes.as_ptr()) },
                original
            );
        }
        assert!(SignedId::qsl_validate(&[21], true).is_err());
        assert!(SignedId::qsl_validate(&[], true).is_err());
        for value in [0u64, u64::MAX] {
            let original = FullWidth(value);
            unsafe {
                original.qsl_comp_write_unchecked(bytes.as_mut_ptr());
            }
            assert!(FullWidth::qsl_validate(&bytes, true).is_ok());
            assert_eq!(
                unsafe { FullWidth::qsl_comp_read_unchecked(bytes.as_ptr()) },
                original
            );
        }
    }
    #[test]
    fn native_custom_array_obeys_the_same_stack_limit() {
        let builder=QslSchemaBuilder::new().with_primitive::<DeviceId>("DeviceId").unwrap().with_source(
            "&P[safety]{#region(aggressive_transmutate=\"Large\");array<DeviceId>(65) ids;#end_region;}"
        ).unwrap();
        let native = builder
            .schemas()
            .iter()
            .find(|s| s.name == "Large")
            .unwrap();
        // Storage policy depends on the configured backend limit, not parsing.
        assert!(crate::modernization::check_storage_limit(native, 256).is_err());
        assert!(crate::modernization::check_storage_limit(native, 512).is_ok());
    }
    #[test]
    fn metadata_is_applied_before_parser_validation() {
        let builder = QslSchemaBuilder::new()
            .with_primitive::<DeviceId>("DeviceId")
            .unwrap()
            .with_source("&P[transmutate,safety]{DeviceId id;}")
            .unwrap();
        assert_eq!(
            builder.schemas()[0].fields[0]
                .data
                .fixed_wire_size()
                .unwrap(),
            Some(4)
        );
        assert!(crate::parse_qsl("&P[transmutate,safety]{DeviceId id;}").is_err());
    }
}
