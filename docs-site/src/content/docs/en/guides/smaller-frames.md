---
title: "QSL-LZ and QPS"
description: "QSL documentation: QSL-LZ and QPS."
---

`str(1..100) name|decompress=true|;` selects a distinct text codec. LZ chooses raw representation when compression is not useful; compare sizes including that codec's headers. The current short raw-copy threshold is 32 B. InlineString's shared length/mode word is 1 B for max≤127, 2 B for max≤32767, then 4 B within supported limits. FixedCompressedString has its own contract.

Checked decompression validates every literal/match run, distance, actual restored extent and capacity. A declared length alone cannot authorize writing outside the destination. Restoration is bounded independently of compressed input size.

QPS retains exact operands, including required quotient/remainder information. Q2/Q4/Q8/Q16 are supported; bare qps selects Q16. Presence prediction remains separate.
Path | Contract
---|---
Scalar payload specialization | Known numeric domains/kernels, selected by PayloadQuantPlan
ByteQps / QPB3 | General exact byte groups, 16 B envelope, original length, explicit scratch/restoration
DomainInteger uses the applicable general path rather than a separate scalar specialization. For safety-enabled non-dyn schemas the generator emits a separate whole-payload QPB3 API, including UTF-8, arrays and wide numbers: serialize_qps_to and deserialize_qps_into, with borrowed restoration for Lifetime. Ordinary serialize retains the scalar-prefix wire and QSL-LZ. The application explicitly chooses the whole-frame API; original lengths restore exactly with the frame.

QPB3 has metadata and incompressible modes; arbitrary bytes cannot always shrink losslessly. Restoration storage can exceed wire size. Some prepared scalar kernels require initialized QUANT_PADDING_BYTES after the wire prefix. These are private storage, excluded from as_slice()/network output. Ordinary exact checked APIs and prepared padded APIs have different contracts.
