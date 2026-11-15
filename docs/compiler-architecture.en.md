# Compiler architecture: one schema builder, independent backends

This contract is shipped in qsl 0.1.2 with qsl-lexer 0.1.0 and qsl-interpreter 0.1.0.

| Crate | Responsibility | Normal dependencies inside QSL |
| --- | --- | --- |
| qsl-lexer | Tokens, QSL/DEQ parsing, template expansion, shared schema model and analysis | none |
| qsl-interpreter | Rust-specific storage lowering and Rust source generation | qsl-lexer |
| qsl | Runtime, checked/trusted I/O, native regions and compatible compiler facade | both compiler crates, qsl-macros-impl |
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
