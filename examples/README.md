# QSL examples

[Русский](#ru) · [English](#en) · [Замеры / Measurements](PERFORMANCE.md) · [Графики / Charts](performance.html)

<a id="ru"></a>

## Русский

Семь небольших программ показывают полный путь от схемы до проверяемого обмена байтами. Это один Cargo-проект: типы генерируются в build.rs, а не написаны вручную. Для каждого сценария есть отдельный main.rs и соответствующий файл в schemas. Подготовка данных для Criterion вынесена из измеряемых операций.

| Пример | Что показывает | Команда: добавьте имя после --example |
|---|---|---|
| [01 — базовый кодек](01-basic/main.rs) | Восемь u64, обычный формат, compact, числовой QPS; deserialize_checked_into | basic |
| [02 — строки](02-strings/main.rs) | String, заимствование UTF-8, CopyGuard, InlineString, LZ и фиксированный [u8; N] | strings |
| [03 — optional-поля](03-optionals/main.rs) | Все восемь состояний трёх полей; default и opt_table | optionals |
| [04 — QPS](04-qps/main.rs) | Числовая упаковка и отдельное сжатие всего payload с восстановлением | qps |
| [05 — RT](05-rt/main.rs) | 16 живых статических буферов, PoolExhausted и повторное получение слота | rt |
| [06 — макросы и хэши](06-macros/main.rs) | Вложенный Rust-шаблон, плоские поля схемы, два хэша анализа | macros |
| [07 — ошибки](07-safety/main.rs) | Диапазон, поддельная длина, неверный UTF-8 и QslError | safety |

Запуск из корня репозитория:

```powershell
cargo run --manifest-path examples/Cargo.toml --example basic
cargo run --manifest-path examples/Cargo.toml --example strings
cargo run --manifest-path examples/Cargo.toml --example optionals
cargo run --manifest-path examples/Cargo.toml --example qps
cargo run --manifest-path examples/Cargo.toml --example rt
cargo run --manifest-path examples/Cargo.toml --example macros
cargo run --manifest-path examples/Cargo.toml --example safety
```

Общая конфигурация находится в [schemas/config.deq](schemas/config.deq): MAX_DATA_SIZE=65535 B, USE_HASH=xxHash. RT включён отдельно через [schemas/rt.deq](schemas/rt.deq); остальные типы не получают RT-пул. [models.rs](models.rs) содержит исходные Rust-структуры, [lib.rs](lib.rs) — включение сгенерированного кода и подготовку наборов данных. В build-зависимости rust-macros включает чтение Rust-исходников; сами атрибуты доступны через обычную зависимость qsl.

Основные программы используют checked API. Доверенные указательные операции встречаются в отдельно подписанных контрольных замерах, после проверки подготовленных кадров. Заимствованная строка не копирует полезные байты; CopyGuard владеет копией; LZ требует восстановления. Их результаты нельзя считать одним и тем же видом работы.

### Производительность

[Все свежие результаты](PERFORMANCE.md) разделены на числа, строки, optional-поля, общий QPS, выходные буферы и чистую валидацию. Для opt_table используются 1, 2, 3, 5 и 10 одинаковых optional-u8; все состояния равновероятны. Строковый набор содержит 1024 символа UTF-8 в каждой строке. Для повторяющейся кириллицы выделен отдельный контроль настоящего LZ; фразовый набор показывает raw-fallback. Общий QPS измеряется отдельно на 1024 байтах ASCII. Восстановление, проверки и уничтожение владеющего результата учитываются там, где они входят в операцию.

Хэши и раскрытие Rust-макросов выполняются при анализе. Для чтения готовой константы не приводится выдуманное время 0 нс; форматирование hex и печать не являются работой бинарного кодека. Пример safety сопоставляется с отдельно измеренными validate и checked-декодированием числового набора.

```powershell
cargo test --manifest-path examples/Cargo.toml --all-targets --features native --locked
cargo clippy --manifest-path examples/Cargo.toml --all-targets --features native --locked -- -D warnings

# Windows: сначала сборка, затем запуск готового Criterion-файла в JobObject с лимитом 8 GiB.
pwsh -ExecutionPolicy Bypass -File examples/tools/measure.ps1 -ResultDirectory examples/results/my-run

# Новый каталог для отдельного раздела; предыдущие результаты сохраняются.
pwsh -ExecutionPolicy Bypass -File examples/tools/measure.ps1 -ResultDirectory examples/results/my-strings -Filter strings_utf8_1024
```

Для runner нужны PowerShell 7 (`pwsh`) и Python 3. Для PNG/SVG-отчёта: `python -m pip install -r examples/tools/requirements-report.txt`. Запуск на других ОС возможен через cargo bench, но Windows-лимит JobObject там не применяется. Строгая конфигурация Criterion в обоих случаях: прогрев 15 с, измерение 10 с, 200 выборок, 99% CI. Основной индекс P=M/T использует исходные полезные байты и время в нс; размер кадра показан отдельно. Исходники и бинарный файл фиксируются по SHA-256.

<a id="en"></a>

## English

Seven small programs cover the path from a schema to checked binary I/O. They share one Cargo project: build.rs generates the types rather than handwritten structs replacing the generator. Each example has its own main.rs and a matching schema file. Criterion preparation is outside the measured operations.

| Example | Purpose | Name after --example |
|---|---|---|
| [01 — basic codec](01-basic/main.rs) | Eight u64 values, raw/compact/scalar-QPS, final-destination decoding | basic |
| [02 — strings](02-strings/main.rs) | String, borrowed UTF-8, CopyGuard, InlineString, LZ and fixed byte arrays | strings |
| [03 — optional fields](03-optionals/main.rs) | All eight states of three fields; default versus opt_table | optionals |
| [04 — QPS](04-qps/main.rs) | Scalar packing versus explicit whole-payload restoration | qps |
| [05 — RT](05-rt/main.rs) | 16 live static slots, exhaustion and lease reuse | rt |
| [06 — macros and hashes](06-macros/main.rs) | Nested Rust templates, flattened fields and build-time identities | macros |
| [07 — errors](07-safety/main.rs) | Domains, forged length, invalid UTF-8 and QslError | safety |

Run from the repository root using the commands above; replace the example name as needed. schemas/config.deq sets MAX_DATA_SIZE=65535 B and USE_HASH=xxHash. Only the separate RT generation uses schemas/rt.deq. models.rs holds the original Rust types; lib.rs includes generated output and prepares fixtures. The build dependency enables source imports through rust-macros; attribute macros themselves are available from ordinary qsl.

Programs use checked methods. Trusted pointer operations appear only in explicitly labelled controls with prevalidated input. A borrowed view does not copy useful bytes; CopyGuard owns a copy; LZ restores bytes. These are different result contracts.

### Performance

[Fresh results](PERFORMANCE.md) separate numbers, strings, optional fields, whole QPS, output storage and validation. opt_table controls use N=1/2/3/5/10 optional u8 fields with uniform states. Each string has 1024 UTF-8 characters; a separate repeated-Cyrillic control demonstrates genuine LZ while the phrase set uses raw fallback; whole QPS uses a separate 1024-byte ASCII dataset. Restoration, validation and owned-value destruction are included where required by the operation.

Hash generation and Rust-template expansion run during analysis. Reading a constant is not assigned a fictional 0 ns time; hex formatting and printing are not codec work. The safety example is explained alongside measured numeric validation and checked decoding.

The Windows runner builds first, then measures the executable under an 8 GiB JobObject cap. It requires Python 3. Other systems can use cargo bench but do not receive the Windows resource limit. Criterion: 15 s warm-up, 10 s measurement, 200 samples, 99% CI. P=M/T uses original useful bytes per ns; wire size is separate. Source and executable SHA-256 hashes identify the measured build.

### Состав свежего прогона / Fresh run structure

35 основных операций сохранены первым снимком. После обнаружения raw-fallback добавлены только 4 парных контроля String/LZ на повторяющейся кириллице. Код QSL одинаков; каждый раздел сравнивается внутри своего бинарного файла. Все исходники и исходные выборки сохранены отдельно.

The 35 main operations retain their first snapshot. Four matched String/LZ controls were added after observing raw fallback. QSL sources are identical; each comparison group shares one executable. Snapshots and samples remain separate.

```powershell
python -m pip install -r examples/tools/requirements-report.txt
python examples/tools/build-report.py examples/results/run-2026-11-04 examples/results/lz-control-2026-11-04
```
