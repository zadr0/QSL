# compact · QSL 0.2.0

Store numeric ranks and ranged length codes at schema-selected bit widths. For u8(0..7), u8(0..31), u16(0..1023), value widths are 3/5/10 bits. Ordinary opt_table and BM are supported; predict_seq and incompatible scalar-QPS/native-overlay formats are rejected. alignment can add padding. Checked deserialization still requires safety.

[Full guide](../../guide.en.md) · [Safety](../../safety.en.md) · [Migration](../../migration-0.2.0.en.md) · [All attributes](index.md).
