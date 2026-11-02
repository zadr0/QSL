# `opt_table` — attribute specification

**Status:** implemented, except `safety` is only a parsed flag in this revision.

## Syntax

```qsl
&TableFields[opt_table] { u64 id; u32? a; str?(0..100) text|decompress=true|; u16? b; }
```

## Exact mechanism and wire effect

For three optionals the wire stores count=3 and 3×u32 LE. If a and b are present but text absent, offsets are 1,0,5 for a(4 B)+b(2 B) payload.

## Constraints and invalid combinations

Required fields first; no compact/bitmap/predict_seq. A single optional uses EOF instead of a directory. Offset parsing validates metadata, not field content.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Existing case:
`optional/seven_table_read` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/optional.rs OptionalOffsets; interpreter.rs emit_sequential_write/read`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)