# qsl-lexer

The QSL frontend: lexical scanning, QSL/DEQ parsing, schema template expansion, schema hashes and build-time analysis. It collects schemas without selecting a code generator. It has no dependency on `qsl-interpreter` or the QSL runtime.

```rust
use qsl_lexer::QslSchemaBuilder;
let builder = QslSchemaBuilder::from_source("&Packet[safety]{u64 id;str(1..64) text;}")?;
assert_eq!(builder.schemas()[0].name, "Packet");
# Ok::<(), Box<dyn std::error::Error>>(())
```

`build_qsl_schem(path)` / `QslSchemaBuilder::scan(path)` load one QSL file or the immediate QSL and DEQ files of a directory, sorted by path. `from_files` loads explicit paths in caller order. Rust template source paths are explicit through `from_files_with_rust` (feature `rust-macros`). Load errors are returned; the library does not terminate the process or silently discard malformed files.

`QslSchemaBuilder` owns expanded schemas and configuration and offers read-only access. `AnalyzedModule` is an alias for this same collection, not another storage layer. `SchemaBackend` defines associated output/error types; `builder.use_backend(&mut backend)` borrows the collection for target-specific generation.

The Rust extension trait comes from `qsl-interpreter`. A future LLVM crate can implement the same contract and add `use_llvm()` without making the frontend depend on either backend. LLVM generation is not implemented. Existing Rust custom-codec names and representation attributes remain annotations of the current QSL model: another backend must explicitly support or reject their ABI. Rust compatibility helpers remain available, but Rust storage lowering is not performed when collecting the shared model.

`Lexer` exposes borrowed tokens and byte spans. Parsing reuses its scanner without allocating a full token list. This frontend is released as qsl-lexer 0.2.0 alongside qsl 0.2.0.

## 0.2.0 contracts

Bitmap masks contain optional fields only and select a native integer width through 128 bits. BM uses descending set-bit traversal; it is compatible with opt_table and suppresses predict_seq ordering. Default tag headers remain unchanged. Native regions produce real nested Rust types. Explicit primitive wrapper macros validate their integer/layout/domain contract while preserving QslCustom codecs. No Windows-specific runtime module or Cargo native feature is required.

[Full English guide](https://github.com/zadr0/QSL/blob/v0.2.0/docs/guide.en.md) · [Breaking changes and migration](https://github.com/zadr0/QSL/blob/v0.2.0/docs/migration-0.2.0.en.md).
