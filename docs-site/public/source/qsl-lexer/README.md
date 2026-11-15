# qsl-lexer

The QSL frontend: lexical scanning, QSL/DEQ parsing, schema template expansion, schema hashes and build-time analysis. It collects schemas without selecting a code generator. It owns schema contracts and depends on `qsl-core` for allocation-free runtime foundations, and has no dependency on `qsl-interpreter` or the `qsl` facade.

```rust
use qsl_lexer::QslSchemaBuilder;
let builder = QslSchemaBuilder::from_source("&Packet[safety]{u64 id;str(1..64) text;}")?;
assert_eq!(builder.schemas()[0].name, "Packet");
# Ok::<(), Box<dyn std::error::Error>>(())
```

`build_qsl_schem(path)` / `QslSchemaBuilder::scan(path)` load one QSL file or the immediate QSL and DEQ files of a directory, sorted by path. `from_files` loads explicit paths in caller order. Rust template source paths are explicit through `from_files_with_rust` (feature `rust-macros`). Load errors are returned; the library does not terminate the process or silently discard malformed files.

`QslSchemaBuilder` owns expanded schemas and configuration and offers read-only access. `AnalyzedModule` is an alias for this same collection, not another storage layer. `SchemaBackend` defines associated output/error types; `builder.use_backend(&mut backend)` borrows the collection for target-specific generation.

The Rust extension trait comes from `qsl-interpreter`. A future LLVM crate can implement the same contract and add `use_llvm()` without making the frontend depend on either backend. LLVM generation is not implemented. Existing Rust custom-codec names and representation attributes remain annotations of the current QSL model: another backend must explicitly support or reject their ABI. Rust compatibility helpers remain available, but Rust storage lowering is not performed when collecting the shared model.

`Lexer` exposes borrowed tokens and byte spans. Parsing reuses its scanner without allocating a full token list. This frontend is released as qsl-lexer 0.3.0 alongside qsl 0.3.0.

## 0.2.0 contracts

Bitmap masks contain optional fields only and select a native integer width through 128 bits. BM uses descending set-bit traversal; it is compatible with opt_table and suppresses predict_seq ordering. Default tag headers remain unchanged. Native regions produce real nested Rust types. Explicit primitive wrapper macros validate their integer/layout/domain contract while preserving QslCustom codecs. No Windows-specific runtime module or Cargo native feature is required.

[Full English guide](https://github.com/zadr0/QSL/blob/v0.3.0/docs/guide.en.md) · [Breaking changes and migration](https://github.com/zadr0/QSL/blob/v0.2.0/docs/migration-0.2.0.en.md).

## 0.3.0: shared core and UTF-8 storage

`qsl-core` is always `#![no_std]`. Its default `alloc` feature provides `alloc::string::String`, `alloc::vec::Vec`, owned output and heap restoration; it never imports `std` or declares `#[global_allocator]`. The final application supplies the allocator. Disabling default features removes allocator-backed APIs. All runtime I/O, codec traits, validation and restoration remain in core. `qsl-rt` is also `no_std` and implements core storage contracts with static regions. `qsl-lexer` only collects/analyzes schemas; `qsl-interpreter` only emits Rust. The `qsl` facade reexports runtime APIs and makes compiler support optional.

In the published 0.3.0 grammar, variable UTF-8 fields use String, &str, `StackString<N>` or `InlineString<N>`. New/reuse are destination modes, not additional types. binary_opt=view generates &str and preserves owned String siblings; binary_opt=inline selects InlineString within STACK_LIMIT. Bounded aggressive storage uses StackString within that budget. STACK_LIMIT defaults to 256 B; larger ordinary inline strings fall back to String. Fixed native string fields retain [u8; N]. Guard containers and binary_opt=guard/true/bytes are rejected. Ordinary String and borrowed &str keep the same wire encoding; InlineString has its own schema-selected length header.

Checked APIs retain UTF-8, length, value-domain, optional metadata, decompressed-size and aggregate-budget checks. Unchecked APIs remain explicit caller contracts. [0.3.0 migration](https://github.com/zadr0/QSL/blob/v0.3.0/docs/migration-0.3.0.en.md).

## Unreleased declaration API

QslBuilder configures DEQ and collects declarations using the existing QslSchemaBuilder. QslSchemBuilder is an alias. name/field supply schema and field identities; build/finish validate before backend selection. max_data_size retains its frame/restoration meaning, while stack_limit controls inline storage. New declarations include physical auto, private aggressive bitmap and versioned dyn line/index modes. binary_opt is retired in this development grammar. These changes are not part of the published 0.3.0 API.


### Exact numeric domains (unreleased)

The local compiler supports u128/i128, optional `|step=N|` and `|values=[...]|`, exact ranked COMPACT encoding and native raw values. Progression endpoints must be reachable by the positive step; finite sets are canonicalized in numeric order. Checked readers reject off-domain values and spare ranks. Native macro metadata includes u128/i128. These changes are not published as 0.3.0. The current development grammar rejects all binary_opt modifiers; storage follows schema modes and budgets.

## Unreleased container contract

Normal Rust-source InlineString and typed Builder now select the same QSL-LZ codec; StackString remains raw UTF-8. This changes wire/hash for old source imports that treated InlineString as raw. Large inline strings preserve their codec in Box and restore directly into heap storage. Strict inline limits include length-word/alignment overhead. Prepared trusted reuse retains supported required String/Box/primitive Vec buffers and requires sufficient destination capacity. Runtime implementations remain exclusively in qsl-core; qsl-rt supplies no_std local/thread-shared storage through its traits.
