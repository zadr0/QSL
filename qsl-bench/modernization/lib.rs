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
