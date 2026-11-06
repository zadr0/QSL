# Shared schema builder and separate compiler crates

`build.rs` collects the schema and DEQ configuration once, then selects Rust generation through an extension trait:

```rust,ignore
use qsl_interpreter::prelude::*;
let builder = build_qsl_schem(".")?;
let rust = builder.use_interpreter()?;
```

The application depends on `qsl` for the generated runtime; its build dependency is `qsl-interpreter`. The program checks round-trip decoding and rejection of a truncated frame. Directory scanning selects immediate `.qsl` and `.deq` files in sorted order; no Rust template paths are inferred.

```powershell
cargo run --manifest-path examples/compiler-pipeline/Cargo.toml
```

The same builder can be reused with another `SchemaBackend`. A future `use_llvm()` extension will select LLVM generation without changing this collection; LLVM is not implemented here.
