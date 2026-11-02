# QSL — exchange of data described in advance

[Русский](README.ru.md) · [Complete guide](docs/guide.en.md) · [Interactive documentation](docs/index.html)

**QSL is a data-description language and Rust code generator for binary exchange.** Its original task is to transfer data whose structure both sides know in advance: field types, valid values, lengths and serialization order.

The developer describes these conditions in a schema. During the build, QSL analyzes it and generates Rust structures, serialization and deserialization methods, validation and the required tables. The application exchanges messages using generated code with a known data layout.

This approach moves part of the work from execution to code generation. The tradeoff is that the developer must define the format explicitly, prepare storage and uphold the schema's constraints. Performance depends on these choices, message sizes and machine architecture.

## A small example

```qsl
&User[safety] {
    u64 id;
    str(1..100) name;
    u8? status;
}
```

This schema describes a user record: a required identifier, a name of 1 to 100 **UTF-8 bytes**, and an optional status. By default, the fields map to Rust `u64`, `String` and `Option<u8>`. The `safety` attribute adds APIs for checked decoding of input data.

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
- [Interactive documentation](docs/index.html) — examples, charts and measurement waveforms.
- [Measurements and commands](docs/features.en.md) — Criterion, source data and `P=M/T`, where `M` is original useful bytes and `T` is nanoseconds.
- [Architecture](docs/architecture.en.md) and [mathematical introduction](docs/mathematics.en.md) — generator design and the reasoning behind optimizations.
- [Zed extension](https://github.com/zadr0/QSL-Extension) — diagnostics, field layout and schema analysis in English and Russian.

## Repository contents

This repository publishes QSL documentation, measurement results and reproduction harnesses. Building the harnesses requires the corresponding `dev-qsl` sources; the complete Cargo workspace is not published here. [Implementation and validation status](SOURCE_AUDIT.md) distinguishes implemented mechanisms from plans. QBProto schema negotiation and the R identifier remain planned.
