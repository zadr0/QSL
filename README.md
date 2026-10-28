# QSL — Schema-driven binary serialization DSL

**Language / Язык:** [English](README.en.md) · [Русский](README.ru.md)  
**Interactive documentation / Интерактивная документация:** [`docs/index.html`](docs/index.html)  
**Detailed technical reference:** [English](docs/architecture.en.md) · [Русский](docs/architecture.ru.md)  
**Public API source index:** [English](docs/API.en.md) · [Русский](docs/API.ru.md)

QSL is a Rust-oriented DSL for **schema-specialized binary serialization**. It translates descriptions of field types, ranges, optionality, string ownership and layout into generated Rust structures and encode/decode methods. It aims to avoid unnecessary runtime decisions where schema information is already known at build time; it does **not** promise to be fastest for every data shape, CPU or transport.

QSL — ориентированный на Rust DSL для **бинарной сериализации со специализацией по схеме**. Описание типов, диапазонов, опциональности, владения строками и раскладки преобразуется в структуры и методы Rust. Цель — убрать вычисления, которые можно выполнить при генерации; это **не** гарантия рекордной скорости на любых данных и процессорах.

```qsl
&Fixed { u32 id; u64 created_at; u16 checksum; }
&Dense[compact] { u8(0..7) flags; u8(0..31) rank; u16(0..1023) code; }
&SeqNumber[predict_seq] { u64 id; u32? extra; }
&TableFields[opt_table] { u64 id; u32? a; str?(0..100) text|decompress=true|; u16? b; }
```

| Feature / Возможность | Status / Статус | Details / Подробнее |
|---|---|---|
| `compact`, `alignment` | Implemented / Реализовано | [RU](docs/architecture.ru.md#4-атрибуты-схем-справочник-и-инварианты) · [EN](docs/architecture.en.md#4-schema-attributes--implementation-and-limitations) |
| `optionals=bitmap`, `predict_seq`, `opt_table` | Implemented / Реализовано | Wire layout and optional metadata / Форматы опциональных полей |
| `binary_opt`, `decompress`, `lifetime` | Implemented / Реализовано | UTF-8 bytes, borrowed views, inline storage and QSL-LZ |
| `QslCustom` | Implemented / Реализовано | Compile-time-sized custom codec trait |
| `RawSocket`, `SerOut` | Implemented / Реализовано | Framed TCP and reusable native buffers |
| Structure macro / Структурный макрос | Planned / Проектируется | No released syntax in supplied sources / Реализации в источниках нет |
| Atomic region cluster `R` | Planned / Проектируется | Not implemented / Не реализован |
| `[safety]` | Parsed flag / Флаг парсера | Not an automatically checked decoder in the supplied generator / Не включает автоматическую проверку входа |

## Documentation layout / Структура документации

- [`docs/index.html`](docs/index.html) — offline-capable HTML reference, RU/EN switching, selectable wire layouts, stage-by-stage code generation, attributes, benchmark explorer and searchable API.
- [`docs/architecture.ru.md`](docs/architecture.ru.md), [`docs/architecture.en.md`](docs/architecture.en.md) — detailed architecture and invariants.
- [`docs/API.ru.md`](docs/API.ru.md), [`docs/API.en.md`](docs/API.en.md) — 276 source-indexed public declarations and trait method signatures; document the exact Rust source revision separately for a published release.
- [`docs/data/all-estimates.csv`](docs/data/all-estimates.csv) — 170 exported Criterion observations, with distinct profiles preserved; no cherry-picked universal speed claims.

**Benchmark generation / Версия тестов:** Historical WebSocket smoke tests used the previous QSL string implementation. Later borrowed `&[u8]` and `Utf8Guard` optimizations are measured separately in Criterion; the old WebSocket RPS/RTT do not measure their effect. / WebSocket-тесты проводились со старой реализацией строк; новые срезы и `Utf8Guard` проверялись отдельно в Criterion. См. [EN](docs/performance.en.md) · [RU](docs/performance.ru.md).

> **Safety note / Безопасность:** Generated trusted/unchecked deserializers require complete, schema-valid data. `RawSocket` frame bounds and the parser flag `[safety]` do not, by themselves, validate arbitrary QSL payloads. Validate untrusted input before calling unsafe decode.
