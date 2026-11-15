---
title: "Memory and reuse"
description: "QSL documentation: Memory and reuse."
---

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.

String, Vec and Box use alloc in a no_std runtime; the application supplies the allocator. A bounded InlineString is embedded when its native size fits STACK_LIMIT and boxed otherwise. The schema capacity, not the current value length, determines this choice.

Prepared unsafe reuse requires sufficient capacity before the call and preserves eligible String, Vec and boxed InlineString storage. It does not grow these buffers. An initialized mutable object and an uninitialized destination have separate contracts.

<a id="reading-into-existing-memory"></a>

## Example {#example}

```qsl
&Record[safety] { u64 id; }
```

```rust
let value = Record { id: 42 };
let mut buffer = [0u8; 64];
let used = value.serialize_checked_to(&mut buffer)?;
let mut destination = core::mem::MaybeUninit::<Record>::uninit();
let restored = Record::deserialize_checked_into(&buffer[..used], &mut destination)?;
assert_eq!(restored.id, 42);
```

[STACK_LIMIT](/language/deq/) · [String](/language/strings/) · [RT](/internals/runtime/)

## QSL_RT {#rt-slot-pool}

qsl-rt provides 1–64 slots. LocalRegions uses no atomics; StaticRegions supports multiple threads. A live lease holds its slot until Drop; a full pool immediately returns PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.
