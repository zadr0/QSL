---
title: 'Arrays and bit windows'
description: 'QSL documentation: Arrays and bit windows.'
---

An array stores its element count. Strings instead count UTF-8 bytes.

**Variable length:** `array<u16>(0..16)` permits zero to sixteen elements. **Fixed length:** `array<u16>(16)` requires exactly sixteen. The element codec determines wire size; the Rust container determines ownership and memory placement.

Vec owns a separate allocation. InlineArray keeps bounded capacity inside the object. Fixed arrays may use [T; N]. STACK_LIMIT constrains the choice; a Vec descriptor is not the array payload.

Size in bits:

$$
B_{\mathrm{array}}=H(n)+\sum_{i=1}^{n} w_i.
$$

H is length metadata, n the element count and w the element codec width. Codec alignment is additional. Array length never authorizes reading beyond the frame.

Nested arrays support up to 32 levels. Element domains and array lengths are checked independently. Reader registers depend on bit position and operations; they are not always 128-bit.
[Read windows](/internals/optimizer/) · [Memory](/guides/memory/)

InlineString keeps its codec in stack storage or Box on the heap. Large heap strings restore directly into their destination. StackString and InlineArray store their bounded capacity inside the object. Explicit aggressive[shift=true] can select a NonZero length word for strings and arrays while keeping None distinct from Some(empty).

no_std excludes std, not alloc. qsl-core always uses #![no_std]; its default alloc feature provides alloc String/Vec, SerOut and heap restoration. The application supplies the allocator; QSL does not declare a global allocator. Disable default features for caller-buffer-only operation.
