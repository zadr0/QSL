#[qsl::schem_macro(aggressive_transmutate)]
#[repr(align(32))]
pub struct SharedMeta {
    pub id: u64,
    pub value: u64,
}
