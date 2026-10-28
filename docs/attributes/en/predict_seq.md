# `predict_seq` — attribute specification

**Status:** implemented, except `safety` is only a parsed flag in this revision.

## Syntax

```qsl
&SeqNumber[predict_seq] { u64 id; u32? extra|default=7|; }
```

## Exact mechanism and wire effect

Absent tail: 8-byte id; present tail: id plus four-byte extra. No presence header. Reads require cursor/len. A truncated tail of length 9..11 violates the unsafe decoder contract.

## Constraints and invalid combinations

Exactly one optional at the end. Cannot combine with compact, bitmap or opt_table. A zero-byte present field cannot be represented. `deserialize_unsafe(ptr)` panics.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Existing case:
`optional/predict_seq_present_read` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/types.rs validate; interpreter.rs emit_sequential_read/emit_trait`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)