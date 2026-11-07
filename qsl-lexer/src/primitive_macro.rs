//! Opt-in scalar wrapper validation, separate from schema field-expansion macros.
/// Define or bind a one-field integer wrapper with an explicit schema domain.
/// Declaration form supplies repr(transparent); binding form checks tuple-field
/// offset, native size/alignment and absence of Drop on the target compiler.
/// Both conversion methods use typed constructors and integer casts.
///
/// ```
/// qsl_lexer::qsl_primitive!(pub struct DeviceId(u32);range=(1,100););
/// use qsl_lexer::QslCustom;
/// assert!(DeviceId(0).qsl_validate_values().is_err());
/// ```
///
/// ```compile_fail
/// qsl_lexer::qsl_primitive!(pub struct Invalid(f32);range=(0,100););
/// ```
///
/// ```compile_fail
/// struct Invalid(u8,u32);
/// qsl_lexer::qsl_primitive!(Invalid,u8,range=(0,255));
/// ```
///
/// ```compile_fail
/// qsl_lexer::qsl_primitive!(pub struct Invalid(u8);range=(0,300););
/// ```
#[macro_export]
macro_rules! qsl_primitive {
    ($(#[$attr:meta])* $vis:vis struct $name:ident($repr:ty); range=($min:expr,$max:expr);) => {
        $(#[$attr])*
        #[repr(transparent)]
        #[derive(Clone,Copy,Debug,PartialEq,Eq)]
        $vis struct $name(pub $repr);
        $crate::qsl_primitive!($name,$repr,range=($min,$max));
    };
    ($name:ident,$repr:ty,range=($min:expr,$max:expr)) => {
        const _: fn() = || {
            fn integer<T: $crate::PrimitiveInteger>() {}
            integer::<$repr>();
        };
        const _:()={
            assert!(::std::mem::size_of::<$name>()==::std::mem::size_of::<$repr>(),"QSL primitive size mismatch");
            assert!(::std::mem::align_of::<$name>()==::std::mem::align_of::<$repr>(),"QSL primitive alignment mismatch");
            assert!(::std::mem::offset_of!($name,0)==0,"QSL primitive field must start at offset zero");
            assert!(!::std::mem::needs_drop::<$name>(),"QSL primitive cannot have Drop");
            assert!(::std::mem::size_of::<$repr>()<=8,"QSL primitive scalar exceeds 64 bits");
            assert!(($min as i128)<=($max as i128)&&($min as i128)>=(<$repr>::MIN as i128)&&($max as i128)<=(<$repr>::MAX as i128),"QSL primitive domain is invalid");
        };
        impl $crate::QslCustom for $name {
            const SIZE:usize=::std::mem::size_of::<$repr>();
            const COMP_SIZE:usize={let delta=(($max as i128)-($min as i128))as u64;((64-delta.leading_zeros())as usize+7)>>3};
            fn qsl_validate(bytes:&[u8],compact:bool)->$crate::QslResult<()>{
                let raw=if compact{
                    if bytes.len()!=Self::COMP_SIZE{return Err($crate::QslError::UnexpectedEof)}
                    let mut rank=[0u8;8];rank[..bytes.len()].copy_from_slice(bytes);(u64::from_le_bytes(rank)as i128)+($min as i128)
                }else{<$repr>::from_le_bytes(bytes.try_into().map_err(|_|$crate::QslError::UnexpectedEof)?)as i128};
                if !(($min as i128)..=($max as i128)).contains(&raw){return Err($crate::QslError::InvalidValue)}Ok(())
            }
            fn qsl_validate_values(&self)->$crate::QslResult<()>{
                let raw: $repr=self.0;
                if !(($min as i128)..=($max as i128)).contains(&(raw as i128)){return Err($crate::QslError::InvalidValue)}Ok(())
            }
            unsafe fn qsl_write_unchecked(&self,to:*mut u8){
                let bytes=self.0.to_le_bytes();::std::ptr::copy_nonoverlapping(bytes.as_ptr(),to,bytes.len());
            }
            unsafe fn qsl_read_unchecked(from:*const u8)->Self{
                let bytes=::std::ptr::read_unaligned(from.cast::<[u8;::std::mem::size_of::<$repr>()]>());
                Self(<$repr>::from_le_bytes(bytes))
            }
            unsafe fn qsl_comp_write_unchecked(&self,to:*mut u8){
                let rank=((self.0 as i128)-($min as i128))as u64;let bytes=rank.to_le_bytes();::std::ptr::copy_nonoverlapping(bytes.as_ptr(),to,Self::COMP_SIZE);
            }
            unsafe fn qsl_comp_read_unchecked(from:*const u8)->Self{
                let mut bytes=[0u8;8];::std::ptr::copy_nonoverlapping(from,bytes.as_mut_ptr(),Self::COMP_SIZE);Self(((u64::from_le_bytes(bytes)as i128)+($min as i128))as $repr)
            }
        }
        // The tuple constructor checks the scalar field's Rust type. Const
        // assertions above establish the representation facts per target.
        unsafe impl $crate::QslPrimitive for $name {
            const REPRESENTATION:$crate::PrimitiveRepr=$crate::PrimitiveRepr{
                bits:(::std::mem::size_of::<$repr>()*8)as u8,
                signed:(<$repr>::MIN as i128)<0,min:$min as i128,max:$max as i128,
            };
            fn qsl_to_bits(&self)->u64{let raw:$repr=self.0;raw as u64}
            fn qsl_from_bits(bits:u64)->Self{Self(bits as $repr)}
        }
    };
}

/// Opt into the runtime's existing NativeField contract for a validated tuple
/// primitive. Invoke in the application, where the qsl runtime is available.
/// The representation assertions from qsl_primitive! must also be present.
#[macro_export]
macro_rules! qsl_native_primitive {
    ($name:ident,$repr:ty) => {
        const _: fn($repr) -> $name = $name;
        const _: () = {
            let descriptor = <$name as $crate::QslPrimitive>::REPRESENTATION;
            assert!(
                descriptor.bits == (::std::mem::size_of::<$repr>() * 8) as u8
                    && descriptor.signed == ((<$repr>::MIN as i128) < 0)
            );
            assert!(
                ::std::mem::size_of::<$name>() == ::std::mem::size_of::<$repr>()
                    && ::std::mem::align_of::<$name>() == ::std::mem::align_of::<$repr>()
                    && ::std::mem::offset_of!($name, 0) == 0
                    && !::std::mem::needs_drop::<$name>()
            );
        };
        unsafe impl ::qsl::modernization::NativeField for $name {
            const NATIVE_LAYOUT: ::qsl::modernization::NativeLayout =
                <$repr as ::qsl::modernization::NativeField>::NATIVE_LAYOUT;
            unsafe fn validate_native_fields(address: *const u8) -> ::qsl::QslResult<()> {
                let raw = ::std::ptr::read_unaligned(address.cast::<$repr>());
                let bytes = raw.to_le_bytes();
                <$name as $crate::QslCustom>::qsl_validate(&bytes, false)
            }
        }
    };
}

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
