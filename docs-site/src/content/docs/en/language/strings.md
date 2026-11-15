---
title: 'Strings, arrays and ownership'
description: 'QSL documentation: Strings, arrays and ownership.'
---

A QSL string is UTF-8 text. Bounds are bytes: “Яa” occupies 3 B. Limits apply to restored text, so a forged compressed length cannot authorize an out-of-bounds write.

## How storage is selected

**String** owns a dynamic buffer. **&str** borrows validated raw text from a live input. **StackString** and **InlineString** keep bounded bytes inside the object. Fixed strings use **[u8; N]** in supported fixed mode.

Storage depends on lifetime, aggressive, the codec and STACK_LIMIT. The limit also applies to explicit inline modes. new creates a destination value; reuse reuses eligible storage. These are decoding strategies, not additional string types.

## When a string compresses {#compressed-strings}

QSL-LZ checks bounded repetition candidates and selects compression only when it actually saves space. Otherwise it stores raw bytes. Frequent characters alone do not guarantee savings: match length, distance and metadata matter.


Arrays have a [separate guide](/language/arrays/); [QPS](/guides/smaller-frames/) is a different reversible codec.

InlineString keeps its codec in stack storage or Box on the heap. Large heap strings restore directly into their destination. StackString and InlineArray store their bounded capacity inside the object. Explicit aggressive[shift=true] can select a NonZero length word for strings and arrays while keeping None distinct from Some(empty).

no_std excludes std, not alloc. qsl-core always uses #![no_std]; its default alloc feature provides alloc String/Vec, SerOut and heap restoration. The application supplies the allocator; QSL does not declare a global allocator. Disable default features for caller-buffer-only operation.

[Limits](/reference/limits/) · [Memory](/guides/manual-layout/)

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.
