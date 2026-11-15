# qps · QSL 0.2.0

qps defaults to Q16; explicit qps=Q2/Q4/Q8/Q16 selects reversible scalar operand packing. It does not predict optional presence and does not quantize strings, arrays or their lengths. Unsuitable schemas retain the explicitly reported raw path. Separate ByteQps can wrap arbitrary bytes with its own 16 B QPB3 framing and restoration buffer; it is not the schema string policy.

[Full guide](../../guide.en.md) · [Safety](../../safety.en.md) · [Migration](../../migration-0.2.0.en.md) · [All attributes](index.md).
