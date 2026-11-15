---
title: 校验输入
description: 属性 [safety] 生成什么，带检查的方法校验哪些内容，以及哪些仍由调用方负责。
---

来自套接字、文件或其他进程的字节都是不可信的。用带检查的方法解码它们；这些方法只为带 `safety` 属性的类型生成。

```qsl
&Packet[safety] {
    u32 id;
    str(1..100) text;
}
```

```rust
let bytes = packet.serialize_checked()?;
let decoded = Packet::deserialize_checked(&bytes)?;

// 或者解码到你已有的内存中：
let mut slot = std::mem::MaybeUninit::uninit();
let decoded: &mut Packet = Packet::deserialize_checked_into(&bytes, &mut slot)?;
```

## 带检查的解码校验什么

在返回值之前，生成的解码器会检查：

- 帧不短于最小的有效帧，包含所有字段，并且在 schema 规定的位置结束；
- 每个数值都在声明的范围内；
- 每个字符串和数组的长度都在边界内；
- 字符串是有效的 UTF-8；
- 可选字段元数据与 schema 一致；
- 填充位和填充字节为零；
- 压缩字符串还原后不超过其上界；
- 解码值的总大小不超过 `MAX_DATA_SIZE`。

校验和解码在同一遍中完成，不需要先调用 `validate`。如果解码失败，`deserialize_checked_into` 会让 `slot` 保持未初始化状态。

## 大小限制

`MAX_DATA_SIZE`（默认 65535 字节）限制输入帧和解码后载荷的总大小。从帧中读出的长度会在预留内存之前对照其边界和这个预算进行检查。可以在 [DEQ 文件](/language/deq/)中调高或调低它。

仓库中的 safety 示例针对下面这个类型，把三个错误的帧传给 `validate`：

```qsl
&Bounded[safety] { u8(1..3) kind; str(1..100) name; }
```
输入 | 结果
---|---
`09 01 00 00 00 61` | `InvalidValue`：`kind` 为 9，超出 `1..3`。
`01 ff ff ff ff` | `UnexpectedEof`：5 字节比 6 字节的最小有效帧短。
`01 01 00 00 00 ff` | `InvalidUtf8`：`ff` 不是 UTF-8。
如果帧的大小有效但长度是伪造的，例如 `01 ff ff ff ff 61`，会得到 `InvalidValue`，因为在复制任何字节之前，长度就已经与上界 `100` 做了比较。

## 带检查的编码

`serialize_checked` 按 schema 检查值（范围、长度、固定长度字符串的 UTF-8、`MAX_DATA_SIZE`），并返回新的 `Vec<u8>`。`serialize_checked_to` 写入你提供的切片，返回写入的字节数，或返回 `BufferTooSmall`。如果最后的大小检查失败，切片可能已经被写入了一部分。无论是否带 `[safety]`，所有类型都会生成这两个方法。

## 无检查方法

`[safety]` 增加带检查的方法，但不会移除无检查的方法。`qsl::Serializable` trait 的方法（例如 `deserialize_into`）以及基于裸指针的读取函数都假定：

- 帧完整，并且对该 schema 有效；
- 目标内存足够大、正确对齐，且与输入不重叠；
- 被借用的输入比结果活得更久。

只对你自己生成或已经校验过的帧使用它们。它们都是 `unsafe` 的，自 0.3.0 起也包括 `Serializable::serialize` 和 `BorrowSerializable::serialize`。带 RT 池或启用了 `qps` 打包的类型所生成的 `serialize()` 是另一个方法，它是安全的，返回 `Result`。

## 校验的局限

- 校验只能证明字节符合 schema 的结构，不能证明它们由哪个 schema 产生：布局相同的两个 schema 接受相同的字节。请在握手中使用 [schema 哈希](/guides/schema-identity/)或你自己的区分标识。
- 校验不会认证发送方。
- [自定义类型](/guides/custom-types/)的安全性取决于它的 `qsl_validate` 实现。默认实现拒绝所有输入。

生成器根据已证明的帧长度上限选择输出类型。上限不超过 STACK_LIMIT（包括相等）时使用 `StackOut<N>`；上限更大或未知时使用 alloc::`Vec<u8>`。传输头计入此限制，输出对象自身的长度和对齐元数据不计入。
