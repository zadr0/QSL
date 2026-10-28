# `decompress=true` — attribute specification

**Status:** implemented, except `safety` is only a parsed flag in this revision.

## Syntax

```qsl
&PackedView { str(0..4096) text|decompress=true|; }
&FixedUtf8Compressed { str(1024) text|decompress=true|; }
```

## Exact mechanism and wire effect

QSL-LZ v1 for bounded dynamic strings, fixed framing rev2 for str(N). Compressed framing is used only when the whole result is smaller; otherwise raw. Adjacent numeric fields are not automatically compressed.

## Constraints and invalid combinations

Up to 65535 decoded bytes; cannot combine borrowed true/guard with dynamic decompress. Performance depends on entropy, length and CPU.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Existing case:
`utf8_1024/ascii/read_compressed` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/strings.rs encode_bounded/decode_bounded/fixed v2`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)