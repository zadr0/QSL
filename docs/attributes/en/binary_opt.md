# `binary_opt` — attribute specification

**Status:** implemented, except `safety` is only a parsed flag in this revision.

## Syntax

```qsl
&ByteView { str(0..4096) text|binary_opt=true|; }
&GuardView { str(0..4096) text|binary_opt=guard|; }
&InlineView { str(0..4096) text|binary_opt=inline|; }
```

## Exact mechanism and wire effect

true generates &'a [u8], guard generates borrowed Utf8Guard<'a>, inline generates InlineString<MAX>. The default creates String. Trusted byte and guard views do not allocate.

## Constraints and invalid combinations

Only string/str. Inline requires a finite bound ≤65535; fixed str(N) always becomes [u8;N]. Decompressed bytes cannot be borrowed from compressed input.

## Runtime contract

Both peers must use compatible schema and wire revisions. The unsafe path does not verify arbitrary network input; source lifetime, data bounds and ranks must satisfy the schema.

## What to measure

Compare complete writes and reads on identical inputs, with metadata and payload separated; record wire size, allocations, CPU, benchmark profile and p99. Existing case:
`utf8_1024/ascii/read_guard` — [all-estimates.csv](../../data/all-estimates.csv).

## Source implementation

`qsl/src/parser.rs binary_opt; strings.rs Utf8Guard/InlineString`.

[← All attributes](index.md) · [Architecture](../../architecture.en.md)