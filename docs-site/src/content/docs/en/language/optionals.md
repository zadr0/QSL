---
title: "PS, OT and bitmap"
description: "QSL documentation: PS, OT and bitmap."
---


Required values have no individual tag in the ordinary format. Optional metadata follows the selected codec: one presence bit, count/tags or BM. Absence never writes a default value.
Mechanism | Role
---|---
Default | Base presence encoding
predict_seq / PS | Payload length → numeric state/index → presence
opt_table / OT | Tag → static reader function
PS+OT | Agreed length graph plus tag dispatch
optionals=bitmap / BM | Optional-only wire mask
Length alone cannot distinguish every same-sized combination; tag skipping requires a proven unambiguous schema/format. OT does not replace PS, and QPS does not predict optional states.

BM v2 selects u8/u16/u32/u64/u128 from optional count. Its set-bit traversal defines payload order, which need not equal default order. Peers must agree on the revision.

Aggressive bitmap is physical Self storage: private MaybeUninit slots, native tag−1 presence bits and get/set/clear methods. Absent slots are not read; Drop touches only occupied slots. This is independent of wire BM.

Ordinary numeric optional storage may use separate payload/mask or a proven NonZero niche. Prefer try_new/accessors over assuming a public Option&lt;T&gt; field. New DomainInteger-specific niche lowering remains pending. opt_table=legacy and state_bits are explicit compatibility formats, not aliases for current PS/OT/QPS.
