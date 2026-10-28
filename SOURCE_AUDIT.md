# Source and claim audit / Аудит источников и утверждений

## Provided sources

| Archive / File | Inspected content | Documentation coverage |
|---|---|---|
| `QSL.rar` | 13 Rust modules (`compat`, `deq`, `interpreter`, `io`, `io_checked`, `lib`, `optional`, `parser`, `parser_util`, `serialize`, `socket`, `strings`, `types`) | Parser, generator paths, flags, string representation, unsafe contracts, raw TCP, public API |
| `qsl-demo.rar` | 7 `.qsl` example files, `schema.deq`, `build.rs`, `main.rs`, tests | Schema examples, custom Token, representative generated operations |
| `qsl-bench(1).rar` | Three Criterion benches; `all-estimates.csv/json`, `profiles.json` | 170 saved Criterion observations; profiles kept distinct |
| `smoke-2-summary.csv` | 10 WebSocket smoke cases, 5 codecs × batch 1/64 | Load latency, records/s, MiB/s, CPU and RSS comparisons |
| Earlier supplied Markdown | Runtime implementation context, contract and benchmark commands | Cross-check against current source; source code takes priority when descriptions disagree |

## Inspected but not executed

The delivered `QSL.rar` has **no Cargo.toml**. In the local runtime, `cargo` is also unavailable. This documentation was produced by source inspection and interpretation of saved results, **not** by compiling the library, emitting a new `qsl_gen.rs`, generating new assembly, or running fresh benchmarks. The generated-method guides explicitly label illustrative code as pseudocode; actual generator implementation is in `qsl/src/interpreter.rs`.

## Verified flags

`FLAG_COMPACT`, `FLAG_SAFETY`, `FLAG_ALIGNMENT`, `FLAG_LIFETIME`, `FLAG_OPTIONAL_BITMAP`, `FLAG_OPT_TABLE`, `FLAG_PREDICT_SEQ`, `FLAG_FIELD_BORROW` are declared in `types.rs`. `parser.rs` implements `binary_opt` and `decompress` field modifiers. `QslSchema::validate` verifies the sequential constraints. Crucially, `FLAG_SAFETY` appears in parser/type definitions but not in the inspected `interpreter.rs` code-path selection. This is documented as a **parsed flag without generated checked-path implementation** in this source snapshot.

## Status distinctions

- **Implemented:** `binary_opt`, `opt_table`, `predict_seq`, borrowing, `InlineString`, `Utf8Guard`, QSL-LZ, `QslCustom`, `OptionalOffsets`, `SerOut`, `RawSocket`, parser/generator.
- **Planned:** structure macro that automatically implements custom codecs; thread-local region allocation and shared atomic cluster `R`. Their public API, wire requirements and measured performance are **not** invented.
- **Distinct concepts:** `Utf8Guard` borrows a byte slice rather than allocating its own copy; zero-copy view vs copy-to-caller storage must not be conflated.
- **Untrusted network input:** binary TCP frame-length checks are not a full semantic validator. Do not pass arbitrary invalid records to unchecked decode.

## Benchmark caveats

Criterion 170 rows span multiple `profile` versions, samples and data categories; a profile selector is required for comparisons. K6 smoke cases have a short 2-second measurement window and a single WebSocket; CPU totals are from full-run processes and must not be divided by measurement-window records without interval alignment. No universally fastest-codec claim is made.

## Generated outputs

- `docs/index.html` and `docs/assets/*`: bilingual static HTML, no external libraries.
- `docs/architecture.{ru,en}.md`: full reference.
- `docs/attributes/{ru,en}/*.md`: individual attribute specifications.
- `docs/{ru,en}/custom-types.md`: QslCustom implementation, safety and macro roadmap.
- `docs/{ru,en}/generated-methods.md`: method-level pseudo-code walkthroughs.
- `docs/API.{ru,en}.md`: source-indexed declaration/method signatures with file + line numbers.
- `docs/performance.{ru,en}.md` and `docs/data/*`: benchmarking.
- `README.md`, `README.en.md`, `README.ru.md`: GitHub repository entry points.
