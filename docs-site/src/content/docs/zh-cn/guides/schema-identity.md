---
title: Schema 标识
description: 严格和轻量两种 schema 哈希、它们涵盖哪些内容，以及如何在握手中比较它们。
---

QSL 在分析 schema 时为每个类型计算两个哈希，并以常量形式存入生成的代码。编码和解码过程从不计算它们，帧中也不包含它们。

```rust
let strict: u128 = Packet::SCHEMA_HASH_STRICT;
let light: u128 = Packet::SCHEMA_HASH_LIGHT;
let h = Packet::SCHEMA_HASHES; // 也可以写成 <Packet as qsl::SchemaIdentity>::SCHEMA_HASHES
println!("{} strict={} light={}", h.algorithm.name(), h.strict_hex(), h.light_hex());
```
哈希 | 涵盖
---|---
严格 | 类型名和字段名、字段顺序、类型、范围、是否可选以及格式选项。
轻量 | 同上，但不含类型名和字段名。
`&A { u32 id; }` 和 `&B { u32 sequence; }` 的轻量哈希相同，严格哈希不同。交换两个类型不同的字段，两个哈希都会改变。

`SCHEMA_HASHES` 是一个 `qsl::SchemaHashes` 值：
字段 | 含义
---|---
`version` | 哈希输入格式的版本，目前为 1。
`algorithm` | `Xxh3_64` 或 `Xxh3_128`。
`strict`、`light` | 两个哈希值。使用 XXH3-64 时高 64 位为零。
`complete` | 类型使用了 [`QslCustom`](/guides/custom-types/#qslcustom) 类型时为 `false`。此时哈希只涵盖自定义类型的 Rust 路径和大小，不涵盖它的编解码实现。
比较哈希时，要连同 `version`、`algorithm` 和 `complete` 一起比较。`strict_hex()` 和 `light_hex()` 根据算法把哈希格式化为 16 或 32 位十六进制数字。

## 算法

默认使用 XXH3-64。如需 128 位，在 [DEQ 文件](/language/deq/)中设置：

```deq
!config USE_HASH = xxHash128;
```

该设置作用于整个构建。两种算法都不是密码学哈希：它们能发现不一致，但不能发现篡改。

## 哪些改动会改变哈希

- 常量求值、Rust 模板展开之后的字段序列。
- 类型、取值范围、字符串和数组的边界，以及哪些字段是可选的。
- 编码格式选项：`compact`、与 `compact` 同用的 `alignment`、`optionals=bitmap`、`predict_seq`、与 `opt_table` 同用的 `predict_seq`、`opt_table=legacy`、`state_bits`。
- 字符串编解码方式：`decompress=true` 和 `decompress=true`。
- `qps` 及其 quantum。`[qps]`、`[qps=Q16]` 和 `[quant=Q16]` 等价。
- 在默认的可选字段格式中，可选字段的标签及标签宽度。

## 哪些不会

`safety`、`lifetime`、`inline` 及其变体、`default`、单独使用的 `opt_table`、不与 `compact` 同用的 `alignment`、`aggressive`、`transmutate`、`cache` 和 `loyal_cache`、`QSL_RT` 以及槽位数。它们改变的是生成的 Rust 代码或内存处理方式，而不是字节。

边界相同时，`String`、`&str`、`StackString` 和 `lifetime` 产生相同的字节，因此哈希也相同。

## 如何使用

这些哈希对格式的描述是保守的，并不能证明两个不同的 schema 无法解码同样的字节。要确保双方一致，请在发送数据之前，在你自己的握手中交换哈希。QSL 不提供握手。
