# Changelog

## 0.2.0 — 2026-11-08

- Change BM to optional-only masks selected from u8/u16/u32/u64/u128 and descending set-bit traversal. Validate spare bits once before dispatch; suppress PS ordering under BM.
- Share optional readers between match/OT and reuse OT handlers for PS. Preserve ordinary non-BM count/tag wire encoding.
- Lower native regions to actual nested types; support nested/reused definitions, COPY and checked aligned borrowing.
- Add explicitly registered scalar wrappers through qsl_primitive!/qsl_native_primitive!, compile-time layout checks, per-value validators and exact custom compact array widths.
- Remove Windows-specific sockets/IPC regions, the native feature, windows-sys and direct OS wait FFI; RT waits use standard Rust synchronization. Preserve portable SerOut and unchecked APIs; ordinary serialization consistently uses Vec/slices.
- Publish current RU/EN repository documentation, English package docs, source-index and migration notes. Retain historical measurements with version labels; the noisy November 7 run is not accepted as a strict result.

This is a breaking pre-1.0 release. BM peers must coordinate migration. qsl-lexer and qsl-interpreter are 0.2.0; qsl-macros-impl is unchanged at 0.1.0.

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
