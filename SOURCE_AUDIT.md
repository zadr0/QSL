# Current source and claim audit / Аудит текущего снимка

Date / Дата: 2026-11-02. QPS codec revision 2; qps default Q16, quant alias, state_bits legacy D format. QPS and opt_table have separate responsibilities.

## Executed / Выполнено

- Core: 69 unit, 10 native tests with serial scheduling, 2 compile-fail — passed.
- Isolated current schema crate: 3 regression tests, including every Q2/Q4/Q8/Q16 golden vector, exact restore, quant alias and the Rust macro from the reported example — passed.
- Node/LSP: 23 tests, RU/EN, numeric QPS without optional fields, macro expansion, layout and generated-source actions — passed.
- Strict Clippy for core/current harness and formatting passed before timing; release analyzer compiled. Lean 4.35: 18 arithmetic theorems checked.
- Criterion: four QPS QSL-only cases plus a separate twenty-case default/opt_table control at N=1/2/3/5/10; 15 s warm-up, 10 s measurement, 200 samples, 99% CI, 8 GiB Job Object. No k6 or other codec was run.
- Source hashes captured before timing and verified afterwards: [manifest](reference/current-source-manifest.json). Exact measured executable SHA is in [data](docs/data/current-qsl.json).
- Selected x86-64 QPS wrappers have zero conditional jumps/calls; raw read has one indirect event call. [ASM](docs/data/qsl-current-asm/). This is not a whole-program proof or cycle-count estimate.

## Scope / Границы

The GitHub repository is a documentation distribution, not a Cargo workspace. Measurement and validation used the corresponding local dev-qsl checkout. The QPS harness is in reference/current-benchmark, the baseline control in reference/opt-table-control, their runner in reference/tools; it requires matching qsl and qsl-macros sources. No new crate release or source publication is implied.

Репозиторий распространяет документацию, а не Cargo workspace. Измерялась соответствующая локальная рабочая копия dev-qsl. Для стенда нужны её qsl и qsl-macros. Это не заявление о новой публикации crate.

Local demo files include a deliberately invalid View schema. They were not silently repaired for a green build; the new feature was compiled/tested in an isolated valid schema crate. The 10 native tests were run serially; no claim that the historically flaky parallel socket suite passed is made.

Локальный View содержит некорректные объявления; ради зелёной сборки они не восстанавливались автоматически. Новая функция проверена отдельной корректной схемой. Native-набор запускался последовательно.

## Implemented and planned / Реализовано и запланировано

Implemented: QPS numeric operands, opt_table length graph, safety checked APIs, qsl_macro/schem_macro Rust imports, string ownership modes, RT pools, native Windows memory/socket/IPC modules. Planned: QBProto handshake and R schema identity. Checked APIs do not validate arbitrary custom unsafe code by themselves. String LZ and QPS are separate codecs.

Реализованы числовой QPS, граф opt_table, checked API, Rust-макросы, строковые режимы, RT и Windows-регионы/сокеты/IPC. QBProto/R остаётся планом. Числовой QPS и строковый LZ разделены.

## Public declaration index / Индекс

574 explicit public declarations and trait signatures were extracted outside Rust comments/string literals. Paths and line numbers refer to this source snapshot, not an invented package release. Native/Windows and rust-macros feature gates still apply. [EN](docs/API.en.md) · [RU](docs/API.ru.md).

## History / История

Older architecture and source-audit pages are preserved in docs/archive/pre-qps-v2. Historical all-estimates/profile/smoke data remain under docs/data, marked archival. The QPS chart uses current-qsl.json only; the separate control chart uses opt-table-control.json. Statements that safety was parsed-only or Rust macros were merely planned describe the old archive, not this snapshot.

Extension 0.5.0: six Tree-sitter corpus cases and 23 Node/LSP tests passed; formatting, strict WASM Clippy and release WASM build passed. Public source: [QSL-Extension](https://github.com/zadr0/QSL-Extension/commit/809016d089133303c3c6d65c9529a88d3833c799). Documentation components were checked offline (RU/EN, numeric restoration, length events, charts, search and local links). Local file-browser preview was unavailable; no visual browser-rendering claim is made.

Baseline control: twenty Criterion smoke cases passed; every size validates/compares all 1024 fixture frames before timing. Source hashes for both profiles remain identical to their pre-run manifests. 8 GiB measured-process limit; control peak job commit 7.89 MiB. opt_table read regressions at N=1 and N=5 are retained explicitly.
