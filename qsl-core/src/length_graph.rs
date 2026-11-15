//! Runtime data for compiled optional-field offset recipes.
/// Static field-layout recipe; byte positions never live inside a record.
#[derive(Clone, Copy, Debug)]
pub struct OffsetRecipe {
    pub prior_fields: u64,
    pub min: usize,
    pub max: usize,
    pub length_slot: Option<(usize, usize)>,
}
