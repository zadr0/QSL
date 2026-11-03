# `lifetime` — attribute specification

**Status:** implemented. Every schema has checked APIs; `[safety]` remains compatible; trusted methods retain their own preconditions.

## Syntax

```qsl
&View[lifetime] { string text; }
```

## Exact mechanism and wire effect

Generates View<'a> with &'a str and BorrowSerializable<'a>. Values borrow wire bytes rather than allocating String; input lifetime and immutability remain caller obligations.

## Constraints and invalid combinations

Custom types and some nested arrays cannot be borrowed. At least one actually borrowed field is necessary to justify a generated lifetime.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Historical case (not part of the fresh QSL-only run):
`utf8_1024/ascii/read_bytes` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/types.rs QslSchema::validate; interpreter.rs generate_lifetime`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)
