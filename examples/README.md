# QSL examples / Примеры QSL

Запускаемые примеры используют текущие `qsl-core`, `qsl-lexer`, `qsl-interpreter` и фасад `proofs/qsl`. Общая конфигурация явно задаёт `STACK_LIMIT = 256` и `MAX_DATA_SIZE = 65535`. Все примеры проверяемого чтения включают `safety`.

Executable examples use the current compiler and runtime through `proofs/qsl`. The shared configuration explicitly sets `STACK_LIMIT = 256` and `MAX_DATA_SIZE = 65535`. Checked readers enable `safety`.

| Example | Что проверяется / What it demonstrates |
|---|---|
| [basic](01-basic/main.rs) | Raw, compact и QPS; точное восстановление чисел / Exact numeric round-trips |
| [strings](02-strings/README.md) | `String`, `&str`, reuse, маленькие контейнеры без `Box`, большие в heap / Small inline and large heap strings |
| [optionals](03-optionals/main.rs) | Default и `opt_table` для всех восьми комбинаций трёх полей / All presence masks |
| [qps](04-qps/main.rs) | Упаковка чисел; строка и её длина остаются raw / Numeric packing with a raw string |
| [rt](05-rt/main.rs) | 16 занятых слотов; `PoolExhausted` без ожидания; повторное использование / Exhaustion and slot reuse |
| [macros](06-macros/main.rs) | Вложенные Rust-шаблоны, плоские поля, хэши / Nested templates, flattened fields and hashes |
| [safety](07-safety/main.rs) | Отклонение нарушенного диапазона, ложной длины и неверного UTF-8 / Invalid-input rejection |
| [builders](08-builders/README.md) | Builder в `build.rs` и QSL-текст: одинаковые байты и хэши / Typed and textual declarations |
| [bmi2](09-bmi2/README.md) | Scalar и PEXT/PDEP, проверка CPU и точного восстановления / ISA kernels and generated writer |

`InlineString` не оборачивается в `Box` для всех схем. Например, `InlineText` содержит `InlineString<55>` непосредственно, а `CompressedText` — `Box<InlineString<4096>>`, поскольку большой контейнер не помещается в заданный лимит. Выбор выполняется при генерации по максимальной ёмкости схемы; короткое текущее значение не меняет тип. Для маленького выходного кадра возвращается `StackOut<N>`, для большого — `Vec<u8>`. Равенство максимального размера кадра и `STACK_LIMIT` допускает стек.

Для `builders` отдельно задан `STACK_LIMIT = 64`; для `bmi2` — профиль `kaby_lake`. Общие настройки остальных примеров не меняются.

The builders example uses its own 64 B stack limit; the BMI2 example opts into the Kaby Lake profile.

Small generated string containers are stored directly; only containers exceeding the native storage budget use heap storage. Selection uses the schema capacity at generation time. Output frames independently use `StackOut<N>` when their proven maximum is at most `STACK_LIMIT`, otherwise `Vec<u8>`.

Из корня репозитория / From the repository root:

```powershell
cargo test --manifest-path examples/Cargo.toml --all-targets --locked
cargo clippy --manifest-path examples/Cargo.toml --all-targets --locked -- -D warnings
cargo run --manifest-path examples/Cargo.toml --example basic --locked
cargo run --manifest-path examples/Cargo.toml --example strings --locked
cargo run --manifest-path examples/Cargo.toml --example optionals --locked
cargo run --manifest-path examples/Cargo.toml --example qps --locked
cargo run --manifest-path examples/Cargo.toml --example rt --locked
cargo run --manifest-path examples/Cargo.toml --example macros --locked
cargo run --manifest-path examples/Cargo.toml --example safety --locked
cargo run --manifest-path examples/Cargo.toml --example builders --locked
cargo run --manifest-path examples/Cargo.toml --example bmi2 --locked
cargo run --manifest-path examples/quickstart/Cargo.toml --locked
cargo run --manifest-path examples/compiler-pipeline/Cargo.toml --locked
```

[Полный контракт / Full contract](../docs/QSL.md) · [English](../docs/QSL.en.md) · [no_std RU](../docs/no-std-core.ru.md) · [no_std EN](../docs/no-std-core.en.md)

Исторические замеры сохранены в `PERFORMANCE.md`, `performance.html` и `performance/`. Их источники в `benches/showcase.rs` исключены из активной Cargo-матрицы: они описывают прежний API. Запуск приведённых программ проверяет поведение, а не измеряет производительность.

Historical measurements and their original benchmark sources remain as snapshots of the former API. The old showcase is excluded from the active Cargo matrix. Running the examples verifies behavior and does not produce performance measurements.
