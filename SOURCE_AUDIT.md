# Current source and claim audit / Аудит текущего снимка

Date / Дата: 2026-11-03. Scalar QPS revision 2, general byte envelope QPB3 revision 3. QPS is exact operand compression; opt_table predicts presence from ordinary-payload length. Both are optional; required fields must precede optional fields.

## Executed / Выполнено

- Core: 72 unit tests, 10 serial native/IPC/socket tests, 2 wire-vector tests and 2 compile-fail examples passed. Strict core Clippy and formatting passed.
- Feature harness: 5 regression tests passed, covering ordinary/optional formats, Rust-template equivalence, whole-record restoration, validation and borrowed restoration lifetimes. Criterion test mode had validated the 95 main fixtures before measurement.
- Optimized kernel: portable/BMI2 equivalence for Q2/Q4/Q8/Q16 and all B=0..8 widths; high bases, tails, exact slices, malformed frames and unchanged golden wire vectors are tested.
- Lean 4.35: 22 arithmetic theorems checked without sorry. This is not a proof of the Rust compiler, full decoder or IPC protocol.
- Extension 0.6.0: 24 Node/LSP tests, 6 Tree-sitter corpus cases, formatting, strict wasm32-wasip2 Clippy and release WASM build passed. RU/EN; scalar/general distinction; no false optional-field requirement for QPS; single-u8 compression warning.

[Extension 0.6.0 source](https://github.com/zadr0/QSL-Extension/commit/eff7c3f84af326b190f7b563fdcc54ffa24bcc11).

## Measurement provenance / Происхождение замеров

The initial 95 main cases and 8 focused UTF-8/whole-record cases use their captured November 2 sources. A register/BMI2 optimization changed only qsl/src/qps_bytes.rs among measured runtime sources. The 14 byte cases and 8 UTF-8 cases were remeasured sequentially on November 3. There are **103 distinct cases and 125 fresh measurement observations**, not 125 distinct scenarios. The other 81 rows retain the original snapshot. Each row records profile/source_manifest; the initial 22 results remain historical groups, alongside the earlier 24 scalar/N controls. No samples from different binaries are pooled.

Criterion 0.8.2; warm-up 15 s per case; measurement 10 s; 200 samples; 99% CI. Windows 11 x86-64, i5-7400, RAM 24 GiB / 2133 MHz; Rust 1.98.0 / LLVM 22.1.8; CPU mask 0x4 and above-normal measuring-thread priority. The OS was not isolated. An 8 GiB Job Object bounds the measured process and descendants, not compilation. Peak job commit in the optimized runs: 7.02 MiB and 8.35 MiB. No k6 or competing codec was run.

[Initial source manifest](reference/features-source-manifest.json) · [Initial UTF-8](reference/qps-utf8-source-manifest.json) · [Optimized byte kernel](reference/qps-words-source-manifest.json) · [Optimized UTF-8](reference/qps-utf8-words-source-manifest.json) · [Current implementation](reference/current-implementation-manifest.json) · [Measurements](docs/data/features.json) · [Report EN](docs/features.en.md) · [RU](docs/features.ru.md).

P=M/T uses original useful bytes. Borrow-view creation, prepared copies and new owned objects have different output contracts. Sample waveforms show batch means, not individual-call p99 or network latency. The general codec exceeds the requested overhead/25 ns targets; those targets are not declared achieved. Packed frames must be strictly shorter than source, including their header. Explicit raw fallback costs 16 B and is shown in wire results.

## Scope / Границы

This GitHub repository distributes documentation and harnesses, not the complete Cargo workspace. Reproduction requires the matching local dev-qsl source tree. Local original/optimized executables and measured-source ZIP files are retained in dated docs/qsl-features folders. No crate release or publication of the engine source is implied.

Локальная схема View намеренно некорректна и сохранена по прямому указанию. Сборка всего demo не объявляется успешной. Функции проверены изолированным корректным стендом. Native-тесты запускаются последовательно; это не заявление о стабильности исторически нестабильного параллельного socket-набора.

## Implementation and claims / Реализация и утверждения

Implemented: QSL/DEQ generation, string modes/LZ, scalar and general QPS, opt_table/predict_seq, checked APIs, Rust templates, RT, native Windows memory, sockets and IPC. QBProto handshake/R remains planned. Arbitrary custom unsafe code is outside the built-in validator guarantee. Outer QPS restores the entire inner frame before field offsets or inner lengths are used; the original outer length is not quantized.

The public declaration index covers 600 explicit declarations/trait signatures outside comments and Rust string literals. Native and rust-macros feature gates still apply. Paths/lines refer to the current-implementation manifest. Earlier ASM wrappers refer only to their archived scalar snapshot; no whole-codec zero-branch or exact cycle-count claim is made.

Offline checks exercise every chart group/operation/metric, 200-point waveforms, RU/EN, numeric restoration, malformed model frames, API search and local links. Browser preview of local files was unavailable, so no visual browser-rendering verification is claimed. Older docs/data and archived pages remain explicitly historical.

## Documentation workbench / Мастерская документации

The bilingual introduction is fully present in README.md. The documentation now includes an explicit scope/constraint explorer, build/runtime stages, a numeric wire-layout model, a string-memory explorer using existing measurements, actual optional-state frames and QPS group operands/restoration. No new timing data or performance estimates were introduced for these interface changes.

[Reference vectors](docs/data/explorer-vectors.json) cover 32 generated dense records in three formats, four optional states and three malformed-frame errors. UI byte models are checked against these native Rust results. Model limits are labelled beside the controls; the browser is not presented as the native QSL generator. [Reference harness](reference/docs-lab/README.md).
