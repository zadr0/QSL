# QSL examples / Примеры

Примеры обновлены под строгий STACK_LIMIT и текущий API. По умолчанию предел 256 B: большие строки inline/LZ восстанавливаются в String; wire-кодек сохраняется. Числовые optional-поля используют try_new/getter/setter. Все примеры с checked-чтением явно включают safety.

These examples use the current strict STACK_LIMIT and generated APIs. The default 256 B limit sends larger inline/LZ COPY strings to String while preserving their codec. Numeric optionals use constructors/accessors; checked examples opt into safety.

| Example | Поведение / Behavior |
|---|---|
| basic | raw, compact и scalar QPS; exact values |
| strings | String, borrowed UTF-8, fixed bytes, inline/LZ heap fallback |
| optionals | default и opt_table, identical wire bytes |
| qps | scalar prefix packing; string and its length stay raw |
| rt | 16 live leases; the 17th waits, then resumes after release |
| macros | nested Rust definitions and flattened generated fields |
| safety | invalid domain, oversized length and invalid UTF-8 return errors |

```powershell
cargo test --manifest-path examples/Cargo.toml --locked
cargo run --manifest-path examples/Cargo.toml --example basic --locked
cargo run --manifest-path examples/Cargo.toml --example strings --locked
cargo run --manifest-path examples/Cargo.toml --example optionals --locked
cargo run --manifest-path examples/Cargo.toml --example qps --locked
cargo run --manifest-path examples/Cargo.toml --example rt --locked
cargo run --manifest-path examples/Cargo.toml --example macros --locked
cargo run --manifest-path examples/Cargo.toml --example safety --locked
```

[Current contract RU](../docs/modernization-2026-11-06/MODERNIZATION.ru.md) · [EN](../docs/modernization-2026-11-06/MODERNIZATION.en.md) · [Four fresh Criterion cases](../qsl-bench/modernization/report.html)

Исторические 39 измерений и их графики остаются в PERFORMANCE.md, performance.html и performance/. Они относятся к прежнему API, в том числе Guard и nonblocking RT. Старый benches/showcase.rs сохранён для истории и исключён из активной Cargo-матрицы. Новый стенд — qsl-bench/modernization; эти старые 39 случаев повторно не запускаются.

The historical 39-case data and charts remain in PERFORMANCE.md, performance.html and performance/. They describe the former API, including Guard and nonblocking RT. The old showcase benchmark source is retained as history and excluded from the active Cargo matrix. The current stand has four targeted cases; the historical matrix is not rerun.
