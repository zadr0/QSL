# `predict_seq` — attribute specification

**Status:** implemented. Every schema has checked APIs; `[safety]` remains compatible; trusted methods retain their own preconditions.

## Syntax

```qsl
&SeqNumber[predict_seq] { u64 id; u32? extra|default=7|; }
```

## Exact mechanism and wire effect

Absent tail: 8-byte id; present tail: id plus four-byte extra. No presence header. Reads require cursor/len. A truncated tail of length 9..11 violates the unsafe decoder contract.

## Constraints and invalid combinations

Standalone predict_seq uses the remaining frame length for a trailing optional sequence. Required fields always precede optional fields. compact and bitmap are incompatible. With opt_table, predict_seq selects direct numeric masks instead of an indirect event call; it does not change bytes. Standalone EOF inference cannot represent a present zero-byte field. Use APIs that supply frame length; deserialize_unsafe(ptr) without length is not supported for this mode.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Historical case (not part of the fresh QSL-only run):
`optional/predict_seq_present_read` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/types.rs validate; interpreter.rs emit_sequential_read/emit_trait`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)
