# QSL architecture: core, lexer, interpreter and facade

This document describes QSL 0.3.0 and qsl-core 0.1.0. [API changes](migration-0.3.0.en.md) · [Complete index](API.en.md).

## Crate boundaries

| Crate | Responsibility | QSL dependencies |
|---|---|---|
| qsl-core | Shared types/errors, schema contracts, range/length analyses, hashes, I/O, string containers, validation and output buffers | none |
| qsl-lexer | Borrowed tokens, QSL/DEQ parsing, file loading, Rust-template expansion and schema builder | qsl-core |
| qsl-interpreter | Rust representation selection; generation of structures, checked/unchecked methods and static tables | qsl-lexer |
| qsl | Application facade, compatible imports and attribute macros | core, lexer, interpreter, qsl-macros-impl |

The frontend no longer owns error or I/O implementations. Its compatible module paths forward to core. Runtime copies in the facade are re-exports too, preserving one QslError/QslCustom type identity through every import path.

Core neither depends on frontend/backend nor parses source text. Shared analyses consume an already constructed model and are not per-frame interpreters. Some compatibility model helpers still describe Rust storage; another backend must explicitly support or reject Rust-specific contracts.

## One schema builder

```rust
use qsl_interpreter::prelude::*;
let builder = build_qsl_schem("schemas")?;
let rust = builder.use_interpreter()?;
```

QslSchemaBuilder owns validated schemas and configuration. use_interpreter borrows that collection, prepares Rust storage and returns generated source. The spelling-compatible use_interpenter alias remains. SchemaBackend provides associated result/error contracts for another backend. LLVM compilation is not implemented.

## Pipeline

1. Lex borrowed tokens and byte spans; parse declarations and constants.
2. Expand templates, assign tags and reject invalid names, cycles and field order.
3. Core analyses calculate bit widths, length graphs, optional masks and schema identities.
4. The Rust backend selects storage using STACK_LIMIT and native-region contracts.
5. Generate concrete structures, codecs and static tables.
6. rustc/LLVM compiles the source and determines the final native ABI.

## Source map

- qsl-core/src/io.rs and bit_io.rs: byte/bit cursors and exact register windows.
- qsl-core/src/validation.rs: bounds, aggregate budgets, UTF-8 and actual string expansion.
- qsl-core/src/strings.rs and modernization.rs: InlineString, StackString, native contracts and 8–128-bit loads.
- qsl-core/src/schema_modernization.rs, types.rs and schema_hash.rs: shared models and analysis.
- qsl-lexer/src/lexer.rs, parser.rs, deq.rs and builder.rs: source processing and schema collection.
- qsl-interpreter/src/frontend.rs and codegen_*.rs: Rust preparation, codecs and dispatch.

## Hot paths and safety

Types, widths and rules are known before execution. Generated readers use prepared operations without parsing a schema. BM owns presence masks, predict_seq owns length-derived indices and opt_table dispatches readers. QPS packs values and does not predict optional presence.

Checked methods enforce cursor contracts before publishing values. Unchecked methods retain caller obligations for complete valid frames, capacity, disjoint storage and lifetimes. Moving an import does not weaken these conditions. Core has no platform-specific Windows socket/IPC modules; applications supply transport and interprocess memory.

## Reproducibility

The schema and wire-affecting options define the format. Moving Rust modules alone does not change a frame. BM v2 remains incompatible with former BM frames; ordinary raw/compact length rules remain unchanged. New string results retain source snapshots, binary SHA-256, Criterion settings and separate OS-noise evidence. Old timing profiles are not presented as current-source measurements.
