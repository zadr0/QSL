---
title: 'Runtime, RT and transport'
description: 'QSL documentation: Runtime, RT and transport.'
---

no_std excludes std, not alloc. qsl-core always uses #![no_std]; its default alloc feature provides alloc String/Vec, SerOut and heap restoration. The application supplies the allocator; QSL does not declare a global allocator. Disable default features for caller-buffer-only operation.

no_std static-region providers implementing core FramePool/FrameStorage; a full pool returns PoolExhausted immediately.
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString keeps its codec in stack storage or Box on the heap. Large heap strings restore directly into their destination. StackString and InlineArray store their bounded capacity inside the object. Explicit aggressive[shift=true] can select a NonZero length word for strings and arrays while keeping None distinct from Some(empty).

Build scripts and test harnesses may use std without linking it into the target runtime. Actual generated schemas run in a no-allocator executable; a separate no_std library verifies alloc-backed storage. Both core profiles and qsl-rt cross-check for wasm32-wasip2.

[Memory](/guides/memory/) · [Safety](/guides/safety/)

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.
