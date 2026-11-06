include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
/// # Safety
/// One complete validated Packed frame, disjoint initialized destination storage.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn qsl_modern_packed_probe(
    input: *const u8,
    len: usize,
    output: *mut Packed,
) {
    <Packed as qsl::payload_quant::QuantizedRecord>::quant_read_padded(input, len, output);
}
/// # Safety
/// One complete validated Raw frame, disjoint initialized destination storage.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn qsl_modern_raw_probe(input: *const u8, len: usize, output: *mut Raw) {
    <Raw as qsl::Serializable>::deserialize_into(input, len, output);
}

// Release-only inspection targets; these are not additional Criterion cases.
macro_rules! exact_load_probe {
    ($name:ident,$load:ident,$word:ty,$bytes:literal) => {
        /// # Safety
        /// input points to the complete initialized byte window.
        #[no_mangle]
        #[inline(never)]
        pub unsafe fn $name(input: *const u8) -> $word {
            qsl::modernization::$load::<$bytes>(input)
        }
    };
}
exact_load_probe!(qsl_load_u8_probe, load_u8, u8, 1);
exact_load_probe!(qsl_load_u16_probe, load_u16, u16, 2);
exact_load_probe!(qsl_load_u32_probe, load_u32, u32, 3);
exact_load_probe!(qsl_load_u64_probe, load_u64, u64, 5);
exact_load_probe!(qsl_load_u128_probe, load_u128, u128, 9);
