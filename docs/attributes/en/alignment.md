# `alignment` — attribute specification

**Status:** implemented, except `safety` is only a parsed flag in this revision.

## Syntax

```qsl
&Aligned[compact,alignment] { u8(0..7) code; u8(0..7) flags; }
```

## Exact mechanism and wire effect

Each field rounds up to a byte boundary: two three-bit fields occupy 2 bytes instead of 1 in plain compact. The generator inserts writer padding and reader.align().

## Constraints and invalid combinations

This is byte alignment of the wire stream, not SIMD or 64-byte RAM alignment. Padding can reduce throughput.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Existing case:
`bits/write/current/3` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/types.rs FLAG_ALIGNMENT; interpreter.rs minimum_size, emit_fast_write`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)