use qsl_examples::{
    compact_numbers, numbers, packed_numbers, CompactNumbers, Numbers, PackedNumbers,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = numbers(3).serialize_checked()?;
    let compact = compact_numbers(3).serialize_checked()?;
    let packed = packed_numbers(3).serialize_checked()?;
    let mut output = std::mem::MaybeUninit::<Numbers>::uninit();
    let decoded = Numbers::deserialize_checked_into(&raw, &mut output)?;
    println!(
        "8 ranged u64 values: raw={} B, compact={} B, QPS={} B; restored f0={}, f7={}",
        raw.len(),
        compact.len(),
        packed.len(),
        decoded.f0,
        decoded.f7
    );
    assert_eq!(
        CompactNumbers::deserialize_checked(&compact)?.f7,
        decoded.f7
    );
    assert_eq!(PackedNumbers::deserialize_checked(&packed)?.f7, decoded.f7);
    Ok(())
}
