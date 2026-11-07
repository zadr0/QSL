# qsl-interpreter

The Rust-emission backend of QSL. It extends the shared `QslSchemaBuilder` from `qsl-lexer` through a trait. It depends on the frontend, not on the runtime. Generated application code continues to use `::qsl`.

```rust
use qsl_interpreter::prelude::*;
let builder = QslSchemaBuilder::from_source("&Packet[safety]{u64 id;}")?;
let rust = builder.use_interpreter()?;
assert!(String::from_utf8(rust)?.contains("pub struct Packet"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use qsl as the application runtime and qsl-interpreter as the build dependency:

```toml
[dependencies]
qsl = "=0.2.0"

[build-dependencies]
qsl-interpreter = "=0.2.0"
```

For files in `build.rs`:

```rust,ignore
use qsl_interpreter::prelude::*;
let builder = build_qsl_schem("schemas")?;
let rust = builder.use_interpreter()?;
std::fs::write(std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("qsl_gen.rs"), rust)?;
```

`InterpreterBuilderExt` supplies `use_interpreter()` and the spelling-compatible `use_interpenter()` alias. It returns generated UTF-8 Rust bytes. `RustGenerator` also implements `SchemaBackend`; `use_backend` supports a configured generator and target-specific outputs/errors. The collection remains reusable after generation or a target error.

Rust storage selection and native-overlay rewriting produce a backend-specific `PreparedModule` from cloned schema data. `generate_analyzed` accepts the shared collection. `prepare_schemas` / `generate_prepared` expose direct Rust-target preparation. `QslInterpreter`, `generate`, `generate_parsed`, `build_qsl_files` and `build_qsl_files_with_rust` retain their existing roles. File-build helpers delegate to the shared builder and write `qsl_gen.rs`.

An LLVM backend can implement `SchemaBackend` in its own crate and add a separate extension trait. No LLVM dependency or placeholder compiler is included. This backend is released as qsl-interpreter 0.2.0 alongside qsl 0.2.0.

## 0.2.0 contracts

Bitmap masks contain optional fields only and select a native integer width through 128 bits. BM uses descending set-bit traversal; it is compatible with opt_table and suppresses predict_seq ordering. Default tag headers remain unchanged. Native regions produce real nested Rust types. Explicit primitive wrapper macros validate their integer/layout/domain contract while preserving QslCustom codecs. No Windows-specific runtime module or Cargo native feature is required.

[Full English guide](https://github.com/zadr0/QSL/blob/v0.2.0/docs/guide.en.md) · [Breaking changes and migration](https://github.com/zadr0/QSL/blob/v0.2.0/docs/migration-0.2.0.en.md).
