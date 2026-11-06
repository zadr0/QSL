#[qsl::schem_macro(aggressive_transmutate)]
pub struct Meta {
    pub id: u64,
    pub value: u32,
    pub tag: qsl::InlineString<4>,
}

#[qsl::schem_macro(aggressive_transmutate)]
pub struct Nested {
    pub lead: u8,
    pub meta: Meta,
    pub end: u8,
}

#[qsl::schem_macro(aggressive_transmutate)]
pub struct ArrayMeta {
    pub values: [u16; 3],
}

#[qsl::schem_macro(aggressive_transmutate)]
pub struct PaddedMeta {
    pub id: u64,
    pub value: u8,
}

// An instrumented, layout-compatible native type proves that trusted region
// entry points do not call either the field or whole-object validator.
#[qsl::schem_macro]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CountedMeta {
    pub id: u64,
    pub value: u32,
    pub tag: [u8; 4],
}

// SAFETY: Meta has the same repr(C) fields, offsets and size. Its validator
// inspects only initialized leaves and accepts only valid CountedMeta values.
unsafe impl qsl::modernization::NativeField for CountedMeta {
    const NATIVE_LAYOUT: qsl::modernization::NativeLayout =
        <Meta as qsl::modernization::NativeField>::NATIVE_LAYOUT;

    unsafe fn validate_native_fields(address: *const u8) -> qsl::QslResult<()> {
        #[cfg(test)]
        crate::native_validation_probe::record();
        <Meta as qsl::modernization::NativeField>::validate_native_fields(address)
    }
}
