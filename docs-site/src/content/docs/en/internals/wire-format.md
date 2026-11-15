---
title: "Dyn v1 and wire format"
description: "QSL documentation: Dyn v1 and wire format."
---


Dyn identifies field extents so a decoder can skip unknown tags. It does not infer the semantics of unknown types or negotiate compatibility automatically.

The 12 B header is `QDY1`, mode:u8, width:u8, reserved:u16=0, revision:u32. Service integers are little-endian. Payload bit offsets are zero-based MSB-first: offset=32,size=4 addresses the high four bits of the fifth byte.
Mode | Representation
---|---
line16/32/64 | `[tag:uW][wire_span:uW][value]`, outer span in bytes
dynamic index32 | `[offset:u16][tag:u8][size:u8]`
dynamic index64 | `[offset:u32][tag:u16][size:u16]`
strict index32 | `[offset:u16][tag:u16]`
strict index64 | `[offset:u32][tag:u32]`
Offsets begin at payload after the header. The final u32 LE trailer stores the byte distance back from the trailer to the index table. Entries are ordered by payload position, making the next offset/table boundary the complete field extent. Index16 is excluded by the accepted v1 contract.

For static values, index size is useful bits. For dynamic strings/arrays, offset points at the length header and size denotes only that header's bit width. UTF-8 length is bytes (`Яa` is 3); array length is elements. The value follows the header; its full extent comes from ordered offsets.

Checked dyn input validates header/reserved bits/revision policy, arithmetic, extents, padding and duplicate tags before publishing Self. It accepts caller scratch explicitly: u32 per index entry or u64 per line entry. It does not allocate a temporary tag Vec. Unknown fields are skipped by checked boundaries. A tag's changed meaning needs an agreed new tag/revision, not merely a matching byte count.

Compact bit fields are supported in permitted indexed forms. Unsupported combinations with QPS, PS, OT, BM, LZ, native overlay and private bitmap return a build-time diagnostic rather than silently changing the format. See .
