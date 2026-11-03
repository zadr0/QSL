# macros

[Current architecture and full contract](../../architecture.en.md).

Apply #[qsl::qsl_macro] / #[qsl::schem_macro] to Rust structs. Original types remain; nested fields flatten into the containing schema. There is no QSL [macro] attribute.

Both attributes are available through qsl by default; implementation lives under qsl/macros. The rust-macros feature enables source imports by the generator, not attribute availability.
