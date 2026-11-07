# inline · QSL 0.2.0

Schema syntax: inline, inline=hint, inline=always or inline=never. Without it Rust decides. This controls generated method attributes, not InlineString storage or PS. It does not change wire identity; always may increase instruction-cache pressure and does not guarantee a faster execution.

[Full guide](../../guide.en.md) · [Safety](../../safety.en.md) · [Migration](../../migration-0.2.0.en.md) · [All attributes](index.md).
