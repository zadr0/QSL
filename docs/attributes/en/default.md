# `default=EXPR` — attribute specification

**Status:** implemented, except `safety` is only a parsed flag in this revision.

## Syntax

```qsl
&SeqNumber[predict_seq] { u64 id; u32? extra|default=7|; }
```

## Exact mechanism and wire effect

The Rust Default object may initialize extra to Some(7), but decoding a missing field gives None. Default is not separately serialized.

## Constraints and invalid combinations

Compile-time numeric expressions only; arbitrary String/custom defaults unsupported. Do not silently replace None with unwrap_or_default.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Existing case:
`records/Packet/read/current` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/parser.rs default; interpreter.rs emit_default_impl`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)