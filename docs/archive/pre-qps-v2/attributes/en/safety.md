# `safety` — attribute specification

**Status:** implemented, except `safety` is only a parsed flag in this revision.

## Syntax

```qsl
&CustomPacket[safety] { Token code; }
```

## Exact mechanism and wire effect

The parser stores FLAG_SAFETY, but the inspected QslInterpreter does not select a checked decoder based on this flag. diagnostics checked I/O is a separate mechanism.

## Constraints and invalid combinations

The flag does not make untrusted network messages safe. Unsafe reads require verified lengths, values, UTF-8 and schema revision.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Existing case:
`records/Fixed/read/current` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/parser.rs flag dispatch; io_checked.rs`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)