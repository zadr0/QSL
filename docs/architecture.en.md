# Compiler architecture: one schema builder, independent backends

This contract is shipped in qsl 0.2.0 with qsl-lexer 0.2.0 and qsl-interpreter 0.2.0.

| Crate | Responsibility | Normal dependencies inside QSL |
| --- | --- | --- |
| qsl-lexer | Tokens, QSL/DEQ parsing, template expansion, shared schema model and analysis | none |
| qsl-interpreter | Rust-specific storage lowering and Rust source generation | qsl-lexer |
| qsl | Portable runtime, checked/trusted I/O, native-layout helpers and compiler facade | both compiler crates, qsl-macros-impl |
| qsl-llvm (planned) | Target-specific LLVM lowering and compilation | qsl-lexer; other dependencies to be decided |

## Collection and selection

The prototype's QslSchemaBuilder idea is adapted to the current parser and configuration model. There is one schema owner. `build_qsl_schem(path)` returns a validated collection; it does not choose a generator or write output. The collection contains owned schemas and DEQ settings and is accessible read-only. `AnalyzedModule` is a type alias for this same builder.

```rust,ignore
use qsl_interpreter::prelude::*;
let builder = build_qsl_schem("schemas")?;
let rust = builder.use_interpreter()?;
std::fs::write(output.join("qsl_gen.rs"), rust)?;
```

For a source string use `QslSchemaBuilder::from_source`; for explicit file lists use `from_files` / `from_files_with_rust`. Directory collection sorts immediate QSL/DEQ paths. Explicit lists retain caller order. Duplicate declarations, conflicting configuration and malformed files return errors. Rust template sources remain explicit.

## Backend contract

`SchemaBackend` declares associated `Output` and `Error` types and a `compile(&mut self, &QslSchemaBuilder)` method. `use_backend` dispatches to a concrete implementation at build time. `RustGenerator` implements this trait and returns generated Rust bytes; `InterpreterBuilderExt` provides `use_interpreter()` and its spelling-compatible alias `use_interpenter()`.

The backend borrows the collection. It performs target-specific lowering on a clone and cannot mutate the original through the public interface. The same builder can be sent to multiple backends or retried after a target failure. No backend selection, trait dispatch or schema parsing is added to generated hot paths.

## LLVM boundary

A future qsl-llvm crate can implement SchemaBackend and export its own extension trait with use_llvm(). It must choose its target architecture, data layout, ABI, intrinsics, object/JIT output and runtime linkage. There is no fake LLVM implementation or dependency in the current split.

Shared data records declared field order, domains, codecs, quantization and optional-field modes before Rust storage selection. Rust's PreparedModule contains its own lowering decisions. Existing Rust-facing annotations and compatibility helpers remain in the current schema model. Custom Rust codecs and native-layout assumptions are not automatically portable: a different backend must support them explicitly or reject them. Shared validation currently enforces the existing QSL language contracts; this is not a claim of arbitrary cross-target ABI support.

## Compatibility and verification

The qsl facade retains existing imports and checked/unchecked runtime methods. The legacy build_qsl_files helpers now delegate collection to QslSchemaBuilder and select Rust generation. Generated code continues to name ::qsl. Physical runtime I/O stays outside the frontend and generator crates.

The compiler-pipeline example has only qsl-interpreter as its build dependency and qsl as its runtime dependency. It exercises the shared builder, DEQ configuration, checked round-trip and truncated-input rejection. Tests check backend reuse, failure isolation and unchanged generated source. No new performance claims or broad benchmark runs accompany this structural change.

## 0.2.0 lowering and dispatch

`qsl-lexer/src/bitmap.rs` derives optional count, word width and marker bounds. `regions.rs` lowers each explicit native range into an independently named schema and one typed parent field, retaining the parent's ordinary tags. `rust_macro.rs`/`schema_macro.rs` expand reusable Rust field templates and retain explicitly native Rust definitions. Region names/layouts are checked before Rust emission.

`qsl-interpreter/src/codegen_optional.rs` owns ordinary count/tag and BM cursor loops. Default match and static OT share the same unchecked field handlers; checked capture has its own fallible handlers. `codegen_qps.rs` retains PS/OT graph handlers. BM disables effective PS so the mask alone determines traversal. `codegen_native.rs` supplies region codec/NativeField adapters and validated views.

`QslSchemaBuilder::with_primitive` registers explicit QslPrimitive layout facts before parser validation; imports reapply those facts after Rust expansion. qsl_primitive! uses typed constructors and a fixed whole-byte compact rank codec. It is not a blanket scalar replacement of arbitrary QslCustom implementations. Cross-target ABI facts are rechecked by generated target-side const assertions.

The runtime no longer contains the Windows native module or any Windows-library dependency. SerOut uses the Rust allocator; RawSocket uses std TCP. Storage selection, register loads, COPY and LIFETIME do not require the removed Cargo feature. Application-owned IPC/TLS and a future LLVM backend are separate concerns.

## Release and reproducibility

qsl, qsl-lexer and qsl-interpreter are version 0.2.0. qsl-macros-impl stays 0.1.0 because its implementation did not change. BM v2 is a wire break; ordinary default/tag headers are preserved. Review the [migration contract](migration-0.2.0.en.md), [runtime safety](safety.en.md) and [source API index](API.en.md). The November 7 Criterion pass is explicitly rejected as a strict result, so this structural release claims no measured speedup.

RT blocking uses std::sync::Mutex/Condvar; acquisition keeps its atomic counter. No direct kernel32 or platform syscall binding remains in QSL.
