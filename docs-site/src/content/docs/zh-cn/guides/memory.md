---
title: 缓冲区与复用
description: 写入自有缓冲区、解码到已有值、静态 RT 槽位池和缓存的输出帧。
---

生成器根据已证明的帧长度上限选择输出类型。上限不超过 STACK_LIMIT（包括相等）时使用 `StackOut<N>`；上限更大或未知时使用 alloc::`Vec<u8>`。传输头计入此限制，输出对象自身的长度和对齐元数据不计入。

String、Vec 和 Box 在 no_std 中使用 alloc，分配器由应用提供。InlineString 的本机大小满足 STACK_LIMIT 时直接嵌入对象，否则使用 Box。选择依据是模式容量，而非当前值的长度。

预备好的 unsafe reuse 要求调用前容量足够，并保留适用的 String、Vec 和 `Box<InlineString>` 缓冲区；此路径不扩容。已初始化对象与未初始化目标具有不同契约。

<a id="reading-into-existing-memory"></a>

## 示例 {#example}

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

LocalRegions · StaticRegions · FramePool · FrameStorage. 1–64 slots; lease → Drop. PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.
