# `optionals=bitmap` — attribute specification

**Status:** implemented, except `safety` is only a parsed flag in this revision.

## Syntax

```qsl
&User[compact,optionals=bitmap] { u32(10..17) id; string? name; }
```

## Exact mechanism and wire effect

Every declared field has a bit, including required fields. The first field is bit 31, the second bit 30 of the first little-endian u32; required presence bits must be 1.

## Constraints and invalid combinations

Adds full four-byte metadata words even for a single optional. Reordering the schema requires a protocol revision. Incompatible with sequential attributes.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Existing case:
`records/User/read/current` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/types.rs bitmap_words; interpreter.rs emit_fast_header`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)