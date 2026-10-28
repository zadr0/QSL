# `compact` — attribute specification

**Status:** implemented, except `safety` is only a parsed flag in this revision.

## Syntax

```qsl
&Dense[compact] { u8(0..7) flags; u8(0..31) rank; u16(0..1023) code; }
```

## Exact mechanism and wire effect

Ranks (3,5,10 bits) occupy 18 bits. For a fixed compact layout, the generator emits constant masks and shifts. comp_size/comp_write/comp_read are generated and the fast path can be unrolled.

## Constraints and invalid combinations

Not a universal compressor: String and numeric fields are not compressed together. opt_table/predict_seq are rejected; alignment can add padding.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Existing case:
`records/Dense/read/current` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/types.rs FLAG_COMPACT; interpreter.rs emit_compact_flat_write/read`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)