# Changelog

## 0.1.2 — 2026-11-07

Split the compiler into qsl-lexer 0.1.0 and qsl-interpreter 0.1.0.
QslSchemaBuilder collects schemas and DEQ configuration once; SchemaBackend
and InterpreterBuilderExt select Rust generation while leaving the collection
reusable. The runtime remains in qsl with compatible compiler re-exports.
Generated Rust for the complete contract fixture is byte-for-byte unchanged.
LLVM backend integration is planned, not implemented.

## 0.1.1 — 2026-11-07

English-only crate README and API introduction; repository quick starts in RU/EN.
Package installation instructions and documentation navigation are updated.


## 0.1.0 — 2026-11-07

Initial crates.io release: schema analysis and Rust code generation, explicit
checked/unsafe APIs, bounded UTF-8 and LZ restoration, numeric QPS, optional-field
dispatch, native regions and schema identities. Includes a runnable RU/EN quick start.
