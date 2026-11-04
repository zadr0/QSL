# QSL

[Protocol comparison RU/EN — 16/100, ambient](benchmarks/protocols/README.md) · [Interactive protocol report](docs/protocols/report.html) · [Optimization candidates RU](benchmarks/protocols/OPTIMIZATION_CANDIDATES.ru.md) / [EN](benchmarks/protocols/OPTIMIZATION_CANDIDATES.en.md)

[Examples RU/EN](examples/README.md) · [Fresh results](examples/PERFORMANCE.md) · [Interactive charts](examples/performance.html)

[Русский](#ru) · [English](#en) · [Docs RU](docs/index.html?lang=ru) · [Docs EN](docs/index.html?lang=en)

<a id="ru"></a>

## Русский

**QSL — язык описания данных и генератор Rust-кода для бинарного обмена.** Исходная задача проекта — передавать данные, структура которых известна обеим сторонам заранее: типы полей, допустимые значения, длины и порядок записи.

Разработчик описывает эти условия в схеме. Во время сборки QSL анализирует её и создаёт структуры Rust, методы сериализации и десериализации, проверки и необходимые таблицы. При обмене приложение использует подготовленный код с известным расположением данных.

Такой подход позволяет перенести часть работы со времени выполнения на этап генерации. Его цена — необходимость явно определить формат, подготовить память и соблюдать ограничения схемы. Производительность зависит от этих решений, размеров сообщений и архитектуры машины.

### Где проходит граница применения

QSL рассчитан на управляемый бинарный обмен: обе стороны заранее согласуют структуру сообщения и настройки кодирования. Типичные задачи — внутренний протокол между приложениями, обмен через IPC, телеметрия с известными полями и обработка повторяющихся записей. Текущая реализация генерирует Rust-код.

Узкая специализация состоит в том, что **структура формата известна при сборке**, а данные подчиняются её инвариантам. Значения и длины могут меняться во время выполнения. Верхняя граница нужна тем возможностям, которые опираются на неё: например, встроенной строке и статическому RT-пулу. Подготовка буферов и выбор владения позволяют убрать часть аллокаций из повторяемого пути.

Разработчик отвечает за совместимость схем, допустимые размеры и время жизни памяти. Для внешнего входа выбирается проверяемый API. Маршрутизация запросов, шифрование и согласование версии относятся к протоколу приложения; байты QSL можно передавать через выбранный транспорт. Готовые сокеты и регионы памяти служат отдельными средствами интеграции. Нативные регионы и прямой WinSock-путь текущей реализации зависят от Windows.

### Короткий пример

```qsl
&User {
    u64 id;
    str(1..100) name;
    u8? status;
}
```

Схема описывает запись пользователя: обязательный идентификатор, имя длиной от 1 до 100 **байтов UTF-8** и необязательный статус. По умолчанию этим полям соответствуют `u64`, `String` и `Option<u8>` в Rust. Безопасные методы доступны каждой схеме. `MAX_DATA_SIZE` по умолчанию равен 65535 байтам; проверяются кадр, восстановление QPS и суммарный объём значений. [Контракт и API](docs/safety.ru.md).

**Обязательные поля всегда располагаются перед опциональными.** Дополнительные атрибуты и механики выбираются отдельно; для этой схемы не требуется включать сжатие, таблицы смещений или принудительный inline.

### Как применяется QSL

1. В `.qsl` описываются поля и их ограничения. Файл `.deq` при необходимости задаёт константы и настройки генерации.
2. Генератор вызывается из `build.rs`; полученный Rust-код подключается к приложению.
3. Приложение записывает и читает бинарные сообщения. Для повторного использования памяти предусмотрены буферные методы, включая `serialize_to` и `deserialize_into`.

Обе стороны согласуют схему и настройки кодирования. Изменение порядка полей, диапазонов или выбранного представления может изменить бинарный формат. Низкоуровневые методы требуют корректных размеров, выравнивания и времени жизни переданных областей памяти.

### Что можно настроить

| Задача | Возможности QSL |
|---|---|
| Уменьшить размер представления | Диапазоны значений, `compact`, строковый LZ и QPS |
| Ускорить доступ к опциональным полям | `opt_table`: таблица читателей по тегам; `predict_seq`: индекс по длине |
| Выбрать владение строкой | `String`, заимствованный UTF-8, копирующая обёртка или встроенная ёмкость; фиксированные строки — `[u8; N]` |
| Управлять памятью | Переиспользуемые буферы, статические RT-пулы, нативные регионы и IPC |
| Расширить схему | Rust-шаблоны полей и пользовательские кодеки |

**QPS выполняет обратимое сжатие с точным восстановлением данных. `predict_seq` использует длину и числовой индекс; `opt_table` выбирает читателя по тегу.** У этих механизмов разные задачи и ограничения.

Выгода каждой опции проверяется на конкретной схеме. Меньший кадр может требовать больше времени на восстановление, а заимствование и создание новой строки дают разные формы результата. Условия измерений, сравнения с обычными методами и ограничения приведены в отчёте.

### С чего начать

- [Руководство](docs/guide.ru.md) — подключение, язык схем, форматы, память и контракты API.
- [Интерактивная документация](docs/index.html?lang=ru) — примеры, графики и кардиограммы измерений.
- [Замеры и команды](docs/features.ru.md) — Criterion, исходные данные и индекс `P=M/T`, где `M` — полезные исходные байты, `T` — время в наносекундах.
- [Архитектура](docs/architecture.ru.md) и [математическое введение](docs/mathematics.ru.md) — устройство генератора и основания оптимизаций.
- [Расширение Zed](https://github.com/zadr0/QSL-Extension) — диагностика, раскладка полей и анализ схем на RU/EN.

### Содержимое репозитория

Крейт `qsl` содержит исходники, внутренние атрибутные макросы и Cargo-манифест; его можно собирать отдельно. Документация и стенды находятся в `docs` и `reference`. [Безопасные схемы](docs/safety.ru.md) · [Хэши](docs/schema-hashes.ru.md).

---

<a id="en"></a>

## English

**QSL is a data-description language and Rust code generator for binary exchange.** Its original task is to transfer data whose structure both sides know in advance: field types, valid values, lengths and serialization order.

The developer describes these conditions in a schema. During the build, QSL analyzes it and generates Rust structures, serialization and deserialization methods, validation and the required tables. The application exchanges messages using generated code with a known data layout.

This approach moves part of the work from execution to code generation. The tradeoff is that the developer must define the format explicitly, prepare storage and uphold the schema's constraints. Performance depends on these choices, message sizes and machine architecture.

### Scope of application

QSL targets controlled binary exchange: both sides agree on message structure and encoding settings in advance. Typical uses include an internal application protocol, IPC, telemetry with known fields, and repeated-record processing. The current implementation generates Rust code.

Its narrow specialization is that **format structure is known at build time**, and data obeys that structure's invariants. Values and lengths can still change at runtime. An upper bound is required by facilities that depend on it, such as inline strings and static RT pools. Preparing buffers and choosing ownership can remove some allocations from the repeated path.

The developer maintains schema compatibility, valid sizes and memory lifetimes. External input uses a checked API. Request routing, encryption and version negotiation belong to the application protocol; QSL bytes can travel over the chosen transport. Socket and memory-region facilities provide separate integration tools. The current native-region and direct WinSock implementations depend on Windows.

### A small example

```qsl
&User {
    u64 id;
    str(1..100) name;
    u8? status;
}
```

This schema describes a user record: a required identifier, a name of 1 to 100 **UTF-8 bytes**, and an optional status. By default, the fields map to Rust `u64`, `String` and `Option<u8>`. Every schema provides checked APIs. `MAX_DATA_SIZE` defaults to 65535 bytes, bounding frames, QPS restoration and aggregate values. [Contract and API](docs/safety.en.md).

**Required fields must always precede optional fields.** Additional attributes and mechanisms are selected independently; this schema does not require compression, offset tables or forced inlining.

### Using QSL

1. A `.qsl` file defines fields and their constraints. An optional `.deq` file supplies constants and generation settings.
2. The generator runs from `build.rs`; the application includes its generated Rust code.
3. The application writes and reads binary messages. Buffer APIs, including `serialize_to` and `deserialize_into`, support storage reuse.

Both sides agree on the schema and encoding settings. Changing field order, domains or representation can change the binary format. Low-level methods require correct sizes, alignment and lifetimes for the supplied memory regions.

### Available choices

| Task | QSL facilities |
|---|---|
| Reduce representation size | Value domains, `compact`, string LZ and QPS |
| Access optional fields efficiently | `opt_table`: tag-indexed readers; `predict_seq`: a length-derived index |
| Choose string ownership | `String`, borrowed UTF-8, a copying wrapper or inline capacity; fixed strings use `[u8; N]` |
| Control storage | Reusable buffers, static RT pools, native regions and IPC |
| Extend a schema | Rust field templates and custom codecs |

**QPS performs reversible compression with exact data restoration. `predict_seq` uses length and a numeric index; `opt_table` selects a reader by tag.** These mechanisms have separate responsibilities and limits.

Each option must be assessed on a concrete schema. A smaller frame can take longer to restore; borrowing and constructing a new string produce different result forms. The report includes measurement conditions, baseline comparisons and limitations.

### Where to start

- [Guide](docs/guide.en.md) — integration, schema language, formats, memory and API contracts.
- [Interactive documentation](docs/index.html?lang=en) — examples, charts and measurement waveforms.
- [Measurements and commands](docs/features.en.md) — Criterion, source data and `P=M/T`, where `M` is original useful bytes and `T` is nanoseconds.
- [Architecture](docs/architecture.en.md) and [mathematical introduction](docs/mathematics.en.md) — generator design and the reasoning behind optimizations.
- [Zed extension](https://github.com/zadr0/QSL-Extension) — diagnostics, field layout and schema analysis in English and Russian.

### Repository contents

The standalone `qsl` crate includes source, internal attribute macros and a Cargo manifest. Documentation and fixtures are under `docs` and `reference`. [Checked schemas](docs/safety.en.md) · [Hashes](docs/schema-hashes.en.md).

[Разделение predict_seq / opt_table и переход со старого формата](docs/optional-modes.ru.md).

[Tag-only opt_table control / Контроль отдельного opt_table](examples/tag-dispatch/README.md).
