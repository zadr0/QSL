# QSL

[English](README.en.md) · [Русский](README.ru.md) · [Interactive docs / Документация](docs/index.html)

QPS = lossless numeric operand compression; opt_table = prediction from length. `[qps]` defaults to Q16 and does not require optional fields.

QPS = обратимое сжатие числовых операндов; opt_table = предиктование по длине. `[qps]` выбирает Q16 и не требует опциональных полей.

```qsl
&P[qps,safety] { u64(0..15) a; u64(0..15) b; }
&S[opt_table,safety] { i32 id; u8? a; u8? b; }
```

Fresh QSL-only data / Свежие данные только QSL: [EN](docs/performance.en.md) · [RU](docs/performance.ru.md).

Current architecture / Текущая архитектура: [EN](docs/architecture.en.md) · [RU](docs/architecture.ru.md).

Checked decoding, Rust structure macros, RT and IPC are implemented in the documented local snapshot. This repository publishes documentation, not a Cargo workspace. Historical pages and measurements are retained as an archive; they are not the current run.

Checked-чтение, Rust-структуры-макросы, RT и IPC реализованы в описанном локальном снимке. Здесь публикуется документация, а не Cargo workspace. Старые страницы и измерения сохранены как архив и не смешиваются со свежим прогоном.

Baseline control / Контроль: default vs opt_table, N=1/2/3/5/10 optional fields — [EN](docs/opt-table-control.en.md) · [RU](docs/opt-table-control.ru.md).
