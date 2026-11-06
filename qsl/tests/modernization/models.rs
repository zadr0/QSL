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
