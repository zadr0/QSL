use crate::*;
#[test]
fn internal_schema_templates_keep_flattened_rust_output() {
    let mut schemas = parser::parse_qsl_raw_with_constants(
        "&Packet[safety]{Meta first;Meta second;u8? extra;}&Meta{Base base;u16 code;}&Base{u64 id;}",
        &Default::default(),
    ).unwrap();
    schemas[1].flags |= FLAG_MACRO;
    schemas[2].flags |= FLAG_MACRO;
    let mut generator = RustGenerator::new();
    generator.generate(&schemas).unwrap();
    let rust = String::from_utf8(generator.output).unwrap();
    assert!(rust.contains("pub struct Meta"));
    assert!(rust.contains("pub struct Base"));
    assert!(rust.contains("pub first_base_id: u64"));
    assert!(!rust.contains("pub first: Meta"));
}
#[cfg(feature = "rust-macros")]
#[test]
fn rust_templates_are_expanded_before_emission_without_duplicate_types() {
    let defs="#[qsl::qsl_macro] pub struct Meta {pub base:Base,pub code:u16} #[qsl::schem_macro] pub struct Base {pub id:u64}";
    let schemas = rust_macro::parse_qsl_with_rust(
        "&Packet[safety]{Meta first;Meta second;u8? extra;}",
        &Default::default(),
        &[defs],
    )
    .unwrap();
    let mut generator = RustGenerator::new();
    generator.generate(&schemas).unwrap();
    let rust = String::from_utf8(generator.output).unwrap();
    assert!(rust.contains("pub first_base_id: u64"));
    assert!(!rust.contains("pub struct Meta"));
    assert!(!rust.contains("pub first: Meta"));
}
