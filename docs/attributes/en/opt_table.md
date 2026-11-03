# `opt_table`

[Complete contract and migration](../../optional-modes.en.md).

## predict_seq: a length-derived index

`predict_seq` determines field presence from length; it does not choose readers by tag. Standalone mode writes no tags: one optional uses EOF, while several optionals use a numeric index into a static table of unique fixed-size subset sums. Ambiguous sums, a zero-size single optional and analysis-budget overflow are rejected. Input length is required; `deserialize_unsafe(ptr)` without length is unsupported.

## opt_table: reader selection by tag

`opt_table` generates a static reader-function table indexed by the actual schema tag: `readers[tag - 1]`. This is not the optional ordinal: required-field tags leave empty entries. Before trusted dispatch, the checked cursor validates allowed tags, order, duplicates, bounds and values across the complete frame. A reader returns its consumed byte count so the cursor advances without computing field length again.

Standalone `[opt_table]` preserves the default wire format and hashes. For several optionals, decoding reads the count and packed tags from the header, then dispatches readers for present fields. A single optional needs only presence; its tag is already known from the schema. No table is allocated per message. The table has `max_tag` Option-function-pointer entries; each entry occupies one machine pointer on x86-64. A table does not guarantee a speedup: indirect calls and result preparation have costs.

## predict_seq + opt_table: preserved length graph

`[predict_seq,opt_table]` preserves the wire bytes and hashes of the former mixed `[opt_table]` implementation. `predict_seq` determines the length event; `opt_table` supplies the reader for each resolved field. These responsibilities are separate.

After metadata and required fields, decoding derives optional payload length L. The numeric index is `I(L) = sum((L > c_j) as usize)`. Events describe forced and possible fields. For two optional u8 fields, `I=(L>0)+(L>1)`: L=0 means neither, L=2 means both, and L=1 requires metadata to distinguish a from b. Skipping a metadata read does not remove its reserved wire byte. When a tag is not separately stored, the generator knows it from the resolved field position.

For string ranges 3..33 and 1..255, boundaries follow from the ranges. CRITICAL_POINTS and the event graph belong to `predict_seq`; OPT_TABLE dispatches readers. General graph analysis supports 12 fields and 65535 payload bytes; equal fixed widths support up to 64 fields. Unsupported graphs fall back to checked EOF, an exact length table, or an explicit presence mask. This is not a promise of a branch-free complete decoder.

`opt_table=legacy` is the separate older wire offset directory. The scalar QPS specialization and its prepared kernel contain no ordinary field tags and use their own bit offsets; `opt_table` does not force them through a byte reader table. With `state_bits`, D supplies presence while optional reads use the reader table. General outer QPS restores the inner frame first.

### Migration and previous measurements

To retain an old `[opt_table]` frame, select `[predict_seq,opt_table]` on both peers. New standalone `[opt_table]` matches default, but generally differs from the former mixed format. There is no automatic version detection. Previous measurements labelled opt_table, including the 1.46–1.73x example speedups, measured the mixed length graph. They are not measurements of the new tag table; original JSON, sources and values remain unchanged.
