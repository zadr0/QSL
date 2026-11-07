# Optional fields in QSL 0.2.0

[Русский](optional-modes.ru.md) · [Migration](migration-0.2.0.en.md)

## Default: count/tags and match

Required fields precede optional fields. Without prediction or BM the frame preserves its existing packed optional-count/tag header, then required values and present optional values. Default dispatch uses a generated match. One optional field keeps a one-bit presence header. Tags remain incremental schema tags; sparse tags produced by region lowering are mapped explicitly.

## BM: optional-only set-bit traversal

Only optional fields occupy bits. Counts 0 / 1–8 / 9–16 / 17–32 / 33–64 / 65–128 use 0 / 1 / 2 / 4 / 8 / 16 header bytes. The mask is little endian; marker 1 occupies its MSB. Read `marker=WIDTH-mask.trailing_zeros()`, clear `mask &= mask-1`, dispatch marker and repeat until zero. Required values retain declaration order; present optional values are serialized and read in descending marker order.

For `&P[safety,optionals=bitmap,opt_table]{u8 id;u16? a;u8? b;}`, id=9, a=0x1234 and b=7 encode as `C0 09 07 34 12`. BM v2 differs from the former u32/all-fields mask. Both peers must migrate together; no automatic format detection is added.

Before traversal, Safety checks the largest marker once if spare bits exist. Empty masks skip it. A full-width mask has no spare bits. After a successful check every remaining dense OT index is bounded. Field data and end-of-frame checks still run through the checked cursor.

## OT: choose an optional reader

Standalone opt_table preserves default bytes and hashes. In ordinary tagged/BM paths its dense static table contains one reader per optional marker; required fields do not create gaps. Absolute schema tags are mapped to dense ordinals. With BM the marker directly selects that ordinal. A per-field cursor reader is shared with default match dispatch; checked and unchecked contracts retain separate readers.

There is no per-frame table allocation. A non-borrowed raw entry is a function pointer (one pointer on x86-64); checked dispatch has its own table. An indirect call is not guaranteed to outperform a match. Compact ordinary layouts and BM can use OT. The former optional_count*tag_bits<=64 restriction is removed from ordinary streaming headers.

## PS: a length-derived index

predict_seq derives presence from payload length, not tag dispatch. A standalone single optional uses EOF; multiple fields require unambiguous supported fixed-size subset sums. Invalid/ambiguous sums and a zero-size optional codec are rejected where length cannot distinguish presence. The trusted API needs input length: a pointer-only deserialize_unsafe cannot infer it.

## PS + OT

The combined mode preserves its existing length graph and wire contract. `I(L)=sum((L>c_j) as usize)` selects an event; forced fields use existing OT readers, ambiguous fields require the mode's presence metadata. The reserved metadata bytes are not silently removed. General graph limits remain 12 fields / 65535 payload bytes; equal fixed widths support up to 64 fields. Those are distinct from BM limits.

PS is inactive when BM is selected and cannot reorder its traversal. QPS does not predict: it is reversible operand-based scalar packing. Legacy offset directories and state_bits are separate formats with their own compatibility checks. Scalar QPS uses bit offsets rather than forcing packed fields through a byte reader table.

## Compatibility and measurements

Changing from a historical mixed opt_table to standalone opt_table requires choosing the correct attributes on both peers; retain predict_seq+opt_table for the historical length graph. Upgrading an ordinary default/tag-table schema to 0.2.0 preserves its count/tag bytes. BM requires coordinated migration and has revision 2 in its schema descriptor. Dispatch-only OT and BM-inactive PS do not alter BM identity.

The four [November 7 measurements](../qsl-interpreter/tests/optional-v2/results/2026-11-07.md) are not accepted as strict results because of background CPU load. Earlier opt_table figures refer to their captured source versions and must not be presented as 0.2.0 performance.
