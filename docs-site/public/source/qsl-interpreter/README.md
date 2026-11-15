# qsl-interpreter

The Rust-emission backend of QSL. It extends the shared `QslSchemaBuilder` from `qsl-lexer` through a trait. It depends on the frontend and shared qsl-core contracts, without depending on the qsl application facade. Generated application code continues to use `::qsl`.

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
qsl = "=0.4.0"

[build-dependencies]
qsl-interpreter = "=0.4.0"
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

An LLVM backend can implement `SchemaBackend` in its own crate and add a separate extension trait. No LLVM dependency or placeholder compiler is included. This backend is released as qsl-interpreter 0.3.0 alongside qsl 0.3.0.

## 0.2.0 contracts

Bitmap masks contain optional fields only and select a native integer width through 128 bits. BM uses descending set-bit traversal; it is compatible with opt_table and suppresses predict_seq ordering. Default tag headers remain unchanged. Native regions produce real nested Rust types. Explicit primitive wrapper macros validate their integer/layout/domain contract while preserving QslCustom codecs. No Windows-specific runtime module or Cargo native feature is required.

[Full English guide](https://github.com/zadr0/QSL/blob/v0.3.0/docs/guide.en.md) · [Breaking changes and migration](https://github.com/zadr0/QSL/blob/v0.2.0/docs/migration-0.2.0.en.md).

## 0.3.0: shared core and UTF-8 storage

`qsl-core` is always `#![no_std]`. Its default `alloc` feature provides `alloc::string::String`, `alloc::vec::Vec`, owned output and heap restoration; it never imports `std` or declares `#[global_allocator]`. The final application supplies the allocator. Disabling default features removes allocator-backed APIs. All runtime I/O, codec traits, validation and restoration remain in core. `qsl-rt` is also `no_std` and implements core storage contracts with static regions. `qsl-lexer` only collects/analyzes schemas; `qsl-interpreter` only emits Rust. The `qsl` facade reexports runtime APIs and makes compiler support optional.

In the published 0.3.0 grammar, variable UTF-8 fields use String, &str, `StackString<N>` or `InlineString<N>`. New/reuse are destination modes, not additional types. binary_opt=view generates &str and preserves owned String siblings; binary_opt=inline selects InlineString within STACK_LIMIT. Bounded aggressive storage uses StackString within that budget. STACK_LIMIT defaults to 256 B; larger ordinary inline strings fall back to String. Fixed native string fields retain [u8; N]. Guard containers and binary_opt=guard/true/bytes are rejected. Ordinary String and borrowed &str keep the same wire encoding; InlineString has its own schema-selected length header.

Checked APIs retain UTF-8, length, value-domain, optional metadata, decompressed-size and aggregate-budget checks. Unchecked APIs remain explicit caller contracts. [0.3.0 migration](https://github.com/zadr0/QSL/blob/v0.3.0/docs/migration-0.3.0.en.md).

## Unreleased backend work

The local backend adds deterministic physical auto layout without wire reordering, private initialized-prefix optional storage, direct tagged/BM encoders and dyn v1 generation. Dyn checked readers take reusable caller scratch; unchecked methods remain separate. Raw UTF-8 lengths count bytes. Unsupported mode combinations fail during generation. Full 128-bit and discrete numeric domains are now lowered locally; new ISA candidates remain pending; no global latency improvement is claimed without measurement.


### Exact numeric domains (unreleased)

The local compiler supports u128/i128, optional `|step=N|` and `|values=[...]|`, exact ranked COMPACT encoding and native raw values. Progression endpoints must be reachable by the positive step; finite sets are canonicalized in numeric order. Checked readers reject off-domain values and spare ranks. Native macro metadata includes u128/i128. These changes are not published as 0.3.0. The current development grammar rejects all binary_opt modifiers; storage follows schema modes and budgets.

## Unreleased container contract

Normal Rust-source InlineString and typed Builder now select the same QSL-LZ codec; StackString remains raw UTF-8. This changes wire/hash for old source imports that treated InlineString as raw. Large inline strings preserve their codec in Box and restore directly into heap storage. Strict inline limits include length-word/alignment overhead. Prepared trusted reuse retains supported required String/Box/primitive Vec buffers and requires sufficient destination capacity. Runtime implementations remain exclusively in qsl-core; qsl-rt supplies no_std local/thread-shared storage through its traits.

## Schema-selected serialize output

Ordinary serialize()/serialize_checked() return Result with a concrete output type. A proven maximum frame size less than or equal to STACK_LIMIT selects StackOut; otherwise output is alloc::Vec<u8>. StackOut length/alignment metadata is excluded from this frame threshold. Unknown bounds also select Vec. This choice is fixed in generated Rust, with no runtime stack/heap branch.

StackOut<N> holds uninitialized byte capacity and a u32 written length. It exposes as_slice(), AsRef<[u8]>, Deref, len(), is_empty() and capacity(); only the initialized prefix is readable. A 64-byte maximum selects StackOut<64>. At STACK_LIMIT=256 B, a frame of 256 bytes selects StackOut and a frame of 257 bytes selects Vec. Transmitted headers count toward frame size; buffer-object metadata does not. Two strings bounded by 4 KiB select Vec even for short actual values; only the current required capacity is reserved.

Compression is not assumed to shrink the maximum: raw fallback and headers count. Small scalar QPS serialize() preserves a typed QuantizedLease backed by StackOut and its frame() decoder. Explicit QSL_RT retains its pool/lease contract. Caller-buffer serialization remains available. Trusted Serializable/BorrowSerializable::serialize returns SerializeOutput::Output; manually implemented records opt into SerializeOutput explicitly. Unchecked methods retain their separate contracts.

Internal code uses qsl_lexer::layout and qsl_core directly. Files containing only forwarding exports were removed; public facade exports stay in lib.rs. Canonical implementation paths are qsl_core::storage, qsl_core::serialize, qsl_core::payload_quant, qsl_lexer::layout and qsl_lexer::payload_quant.

### Configuring the output limit

DEQ `!config STACK_LIMIT = 128;` and Builder `config(|c| { c.stack_limit(128); })` set the same GenerationConfig.stack_limit. serialize has no separate threshold. StackOut capacity follows the proven frame maximum; STACK_LIMIT bounds wire bytes inclusively. Changing this limit preserves wire bytes and schema hashes.

At STACK_LIMIT=64, a 64-byte frame selects StackOut<64> and a 65-byte frame selects Vec. A zero limit still allows a proven zero-byte frame on the stack. Limits 63, 65, 128 and 256 are also tested. Compact numeric domains use encoded rank width: sixteen u128 fields restricted to 0..15 need an 8-byte frame, not 256 bytes.

Large borrowed schemas retain caller-buffer serialization in no_std builds without alloc. Missing heap-owned output does not disable serialize_checked_to/serialize_to; the no-allocator executable checks this path.
