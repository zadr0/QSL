# QSL — Rust code-generating binary serialization DSL

[Interactive site](docs/index.html) · [Full technical reference](docs/architecture.en.md) · [Rust API index](docs/API.en.md) · [Русский](README.ru.md)

QSL declares message **field types, order, ranges, optionality, string representations and memory ownership** in a DSL. The parser produces a normalized `QslSchema`; `QslInterpreter` generates Rust structs and specialized reader/writer methods. Known offsets, masks and bit widths are calculated at build time rather than reinterpreted for every record.

```qsl
&Fixed { u32 id; u64 created_at; u16 checksum; }
&Dense[compact] { u8(0..7) flags; u8(0..31) rank; u16(0..1023) code; }
&SeqNumber[predict_seq] { u64 id; u32? extra; }
&TableFields[opt_table] { u64 id; u32? a; str?(0..100) text|decompress=true|; u16? b; }
```

Implemented features include ranked `compact`, optional `predict_seq`, indexed `opt_table`, `optionals=bitmap`, string modes via `binary_opt` and explicit `decompress`, `QslCustom`, `SerOut`, and framed TCP `RawSocket`. A structure macro for custom types and an atomic region cluster `R` are **architectural plans**, not implemented APIs in the supplied archives.

**Safety:** parsing `[safety]` does not currently select a fully checked generated decoder. Trusted unsafe reads require complete schema-valid input. `RawSocket` provides transport framing, not deep validation of arbitrary QSL records.

**Performance:** [`docs/data/all-estimates.csv`](docs/data/all-estimates.csv) includes 170 Criterion observations across separate profiles, with regressions as well as improvements. Borrowed bytes, trusted UTF-8 Guard, checked validation, owned `String`, and `deserialize_into` are different operations. CPU architecture, memory bandwidth, cache behavior and transport influence actual throughput; QSL is not universally the fastest codec.

## Publish on GitHub Pages

Copy the root README files and `docs/` into your repository. Enable **Settings → Pages → Deploy from a branch → main → /docs**. The HTML runs without a framework, package manager or CDN and supports RU/EN switching. Full method signatures, layouts, examples and constraints are in the [technical reference](docs/architecture.en.md) and [API index](docs/API.en.md).

The uploaded QSL core archive lacked `Cargo.toml`, so no fresh standalone compilation or benchmark execution is claimed in this documentation build.

**Benchmark version note:** WebSocket smoke numbers used the older QSL string implementation; later `&[u8]`/`Utf8Guard` optimizations were measured separately in Criterion, not in those WebSocket end-to-end results. See [load tests](docs/performance.en.md).

