use qsl::Serializable;
use qsl::payload_quant::QuantizedRecord;
use qsl_current_bench::{Qps, Raw};
/// # Safety
/// Exact valid frame, disjoint aligned uninitialized output.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn asm_raw_read(input: *const u8, len: usize, out: *mut Raw) {
    unsafe { Raw::deserialize_into(input, len, out) }
}
/// # Safety
/// Valid source and exact writable frame capacity.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn asm_raw_write(value: *const Raw, out: *mut u8) -> usize {
    unsafe { (*value).serialize_into_unsafe(out) }
}
/// # Safety
/// Valid frame plus initialized QUANT_PADDING_BYTES; disjoint aligned output.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn asm_qps16_read(input: *const u8, len: usize, out: *mut Qps) {
    unsafe { Qps::quant_read_padded(input, len, out) }
}
/// # Safety
/// Valid source and initialized writable frame plus QUANT_PADDING_BYTES.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn asm_qps16_write(value: *const Qps, out: *mut u8) -> usize {
    unsafe { (*value).quant_write_prepared(out) }
}
fn main() {
    for f in [
        asm_raw_read as *const (),
        asm_raw_write as *const (),
        asm_qps16_read as *const (),
        asm_qps16_write as *const (),
    ] {
        std::hint::black_box(f);
    }
}
