# qsl-macros-impl

Procedural attributes used by [QSL](https://github.com/zadr0/QSL):
qsl::qsl_macro and qsl::schem_macro. QSL re-exports both attributes through its
main crate. This implementation package is versioned together with qsl.

Rust templates retain their Rust type and expand into schema fields during
QSL analysis. The aggressive_transmutate argument provides fixed native layout
metadata and native field validation.

MIT. See LICENSE.
