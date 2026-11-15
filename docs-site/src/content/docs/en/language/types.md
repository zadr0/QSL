---
title: "Schema language and types"
description: "QSL documentation: Schema language and types."
---

Choose a value type, then declare its constraints. The generated Rust representation may depend on storage policy: a string can become String, &str or bounded storage.
Family | Meaning
---|---
`u8…u128 / i8…i128` | 8–128-bit integers; intervals and discrete domains.
`f32 / f64` | Floating-point values using their selected codec.
`bool / char / flags` | Boolean, Unicode scalar and bit flags.
`str / string` | UTF-8 with fixed or bounded byte length.
`array<T>` | An array with element codec T and its own length.
`Custom` | A custom type with an explicitly registered contract.
[Domains](/language/numbers/) · [Strings](/language/strings/) · [Arrays](/language/arrays/) · [Custom types](/guides/custom-types/)
