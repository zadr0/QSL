# QSL — exchange of data described in advance

[Русский](README.ru.md) · [Complete guide](docs/guide.en.md) · [Interactive documentation](docs/index.html?lang=en)

**QSL is a data-description language and Rust code generator for binary exchange.** Its original task is to transfer data whose structure both sides know in advance: field types, valid values, lengths and serialization order.

The developer describes these conditions in a schema. During the build, QSL analyzes it and generates Rust structures, serialization and deserialization methods, validation and the required tables. The application exchanges messages using generated code with a known data layout.

This approach moves part of the work from execution to code generation. The tradeoff is that the developer must define the format explicitly, prepare storage and uphold the schema's constraints. Performance depends on these choices, message sizes and machine architecture.

## Scope of application

QSL targets controlled binary exchange: both sides agree on message structure and encoding settings in advance. Typical uses include an internal application protocol, IPC, telemetry with known fields, and repeated-record processing. The current implementation generates Rust code.

Its narrow specialization is that **format structure is known at build time**, and data obeys that structure's invariants. Values and lengths can still change at runtime. An upper bound is required by facilities that depend on it, such as inline strings and static RT pools. Preparing buffers and choosing ownership can remove some allocations from the repeated path.

The developer maintains schema compatibility, valid sizes and memory lifetimes. External input uses a checked API. Request routing, encryption and version negotiation belong to the application protocol; QSL bytes can travel over the chosen transport. Socket and memory-region facilities provide separate integration tools. The current native-region and direct WinSock implementations depend on Windows.

## A small example

```qsl
&User {
    u64 id;
    str(1..100) name;
    u8? status;
}
```

This schema describes a user record: a required identifier, a name of 1 to 100 **UTF-8 bytes**, and an optional status. By default, the fields map to Rust `u64`, `String` and `Option<u8>`. Every schema provides checked APIs. `MAX_DATA_SIZE` defaults to 65535 bytes, bounding frames, QPS restoration and aggregate values. [Contract and API](docs/safety.en.md).

**Required fields must always precede optional fields.** Additional attributes and mechanisms are selected independently; this schema does not require compression, offset tables or forced inlining.

## Using QSL

1. A `.qsl` file defines fields and their constraints. An optional `.deq` file supplies constants and generation settings.
2. The generator runs from `build.rs`; the application includes its generated Rust code.
3. The application writes and reads binary messages. Buffer APIs, including `serialize_to` and `deserialize_into`, support storage reuse.

Both sides agree on the schema and encoding settings. Changing field order, domains or representation can change the binary format. Low-level methods require correct sizes, alignment and lifetimes for the supplied memory regions.

## Available choices

| Task | QSL facilities |
|---|---|
| Reduce representation size | Value domains, `compact`, string LZ and QPS |
| Access optional fields efficiently | `opt_table` and `predict_seq`, using known sizes and data length |
| Choose string ownership | `String`, borrowed UTF-8, a copying wrapper or inline capacity; fixed strings use `[u8; N]` |
| Control storage | Reusable buffers, static RT pools, native regions and IPC |
| Extend a schema | Rust field templates and custom codecs |

**QPS performs reversible compression with exact data restoration. Length-based prediction belongs to `opt_table` and `predict_seq`.** These mechanisms have separate responsibilities and limits.

Each option must be assessed on a concrete schema. A smaller frame can take longer to restore; borrowing and constructing a new string produce different result forms. The report includes measurement conditions, baseline comparisons and limitations.

## Where to start

- [Guide](docs/guide.en.md) — integration, schema language, formats, memory and API contracts.
- [Interactive documentation](docs/index.html?lang=en) — examples, charts and measurement waveforms.
- [Measurements and commands](docs/features.en.md) — Criterion, source data and `P=M/T`, where `M` is original useful bytes and `T` is nanoseconds.
- [Architecture](docs/architecture.en.md) and [mathematical introduction](docs/mathematics.en.md) — generator design and the reasoning behind optimizations.
- [Zed extension](https://github.com/zadr0/QSL-Extension) — diagnostics, field layout and schema analysis in English and Russian.

## Repository contents

The standalone `qsl` crate includes source, internal attribute macros and a Cargo manifest. Documentation and fixtures are under `docs` and `reference`. [Checked schemas](docs/safety.en.md) · [Hashes](docs/schema-hashes.en.md).
