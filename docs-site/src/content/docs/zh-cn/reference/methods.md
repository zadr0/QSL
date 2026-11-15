---
title: 生成的方法
description: QSL 为每个 schema 类型生成的方法和常量，按对调用方的要求分组。
qslResources: [{"label": "docs.rs", "url": "https://docs.rs/qsl/0.3.0/qsl/"}]
---

每个声明都会生成一个带有下列方法的 Rust 类型，具体有哪些方法取决于 schema 的属性。运行时 crate 中公开条目的完整列表见 docs.rs。

## 带检查：可安全处理不可信数据

仅当类型带 `[safety]` 时生成：
方法 | 签名（简化） | 说明
---|---|---
`deserialize_checked` | `fn(&[u8]) -> QslResult<Self>` | 边解码边校验。带 `lifetime` 时输入为 `&'a [u8]`。
`deserialize_checked_into` | `fn(&[u8], &mut MaybeUninit<Self>) -> QslResult<&mut Self>` | 解码到你的内存中。出错时目标保持未初始化。
所有类型都会生成：
方法 | 签名（简化） | 说明
---|---|---
`validate` | `fn(&[u8]) -> QslResult<()>` | 检查整个帧，但不构造值。调用 `deserialize_checked` 之前不需要先调用它。
`validate_values` | `fn(&self) -> QslResult<()>` | 检查范围、长度、固定长度字符串的 UTF-8 以及 `MAX_DATA_SIZE` 预算。
`serialize_checked` | `fn(&self) -> QslResult<`Vec<u8>`>` | 先检查值，再编码。
`serialize_checked_to` | `fn(&self, &mut [u8]) -> QslResult<usize>` | 编码到至少 `serialized_size()` 字节的切片中，返回写入的字节数。
含有可选数值字段的类型还会生成：
方法 | 签名（简化） | 说明
---|---|---
`try_new` | `fn(field, ...) -> QslResult<Self>` | 按声明顺序接收所有字段（可选字段以 `Option<T>` 传入），并检查取值。
`name` | `fn(&self) -> Option<T>` | 读取可选字段 `name`。
`set_name` | `fn(&mut self, Option<T>) -> QslResult<()>` | 修改该字段。值超出范围时返回 `InvalidValue`。
见 [Rust 中的可选字段](/language/declarations/#optional-fields-in-rust)。

## 可信：由调用方保证有效性

这些是 `qsl::Serializable` trait 的方法。带 `lifetime` 的类型改为实现 `qsl::BorrowSerializable<'a>`，其中的解码器名为 `deserialize_borrowed`、`deserialize_borrowed_into` 和 `deserialize_borrowed_unsafe`。在这两个 trait 中，除 `serialized_size` 和 `get_size` 外，所有方法都是 `unsafe` 的。
方法 | 调用方需要保证
---|---
`serialize`、`serialize_to`、`serialize_into_unsafe`、`serialize_unsafe` | 值对 schema 有效；目标有 `serialized_size()` 个可写字节。
`deserialize`、`deserialize_unsafe` | 输入是一个完整、有效的帧。
`deserialize_into(ptr, len, out)` | 帧有效；`out` 可写、已对齐，且与输入不重叠。`out` 中的旧值不会被 drop。
`deserialize_reuse(input, &mut value)` | 帧有效；每个被复用的字符串缓冲区容量足够。在布局允许时保留缓冲区。
`deserialize_prepared_into(ptr, len, &mut value)` | 同上，但使用裸指针和长度。
安全的 trait 方法：
方法 | 返回值
---|---
`serialized_size(&self)` | 需要为该值提供的空间：帧的确切长度；值中含压缩字符串时为上界。
`get_size()` | 最小可能的帧长度。
## 特定场景
方法 | 适用情况
---|---
`serialize()` | 带 RT 池时：安全，检查值，等待空闲槽位并返回租约。在启用 `qps` 打包且没有 RT 池的类型上：安全，返回 `QuantizedOut`。见 [RT 槽位池](/guides/memory/#rt-slot-pool)。
`serialize_quantized_to` | 启用 `qps` 时。检查值和容量，然后运行打包内核。
`quant_write_prepared`、`quant_read_padded` | 启用 `qps` 时。`qsl::payload_quant::QuantizedRecord` 的方法。`unsafe`：调用方保证范围、填充，以及内存互不重叠且已对齐。
`serialize_qps_to`、`deserialize_qps_into`、`deserialize_qps` | `ByteQps` 适配器。仅用于带 `[safety]` 和 `qps`、且不含字符串或数组的类型。
`serialize_cached` | 使用 `cache=N` 或 `loyal_cache` 时。写入 `CachedFrame`。
`deserialize_transmutate_copy`、`deserialize_overlay` | [`transmutate`](/reference/attributes/#transmutate)。各有一个跳过检查的 `_unchecked` 变体。
## 常量
常量 | 含义
---|---
`SCHEMA_HASH_STRICT: u128` | 包含名称的哈希。见 [schema 标识](/guides/schema-identity/)。
`SCHEMA_HASH_LIGHT: u128` | 不含名称的哈希。
`SCHEMA_HASHES` | 两个哈希，以及算法、版本和完整性。也可以通过 `<T as qsl::SchemaIdentity>::SCHEMA_HASHES` 访问。
`MAX_DATA_SIZE` | 带检查的方法所用的预算。
`NATIVE_BYTES`、`NATIVE_ALIGNMENT` | 生成的结构体的 `size_of` 和 `align_of`。
`WIRE_MAX_BYTES: Option<usize>` | 最大帧大小；没有上限时为 `None`。
`CACHE_BYTES` | `serialize_cached` 的容量。仅在使用 `cache=N` 或 `loyal_cache` 时存在。
`NATIVE_WIRE_COMPATIBLE` | 原生布局是否与编码布局一致。仅在使用 `transmutate` 时存在。
`QUANT_PADDING_BYTES` | 预备的 `qps` 读取所需的额外可读字节数。目前为 `0`。
`REUSES_STORAGE` 是 `Serializable` trait 的常量：当 `deserialize_reuse` 原地更新缓冲区时，`<T as qsl::Serializable>::REUSES_STORAGE` 为 `true`。

生成器根据已证明的帧长度上限选择输出类型。上限不超过 STACK_LIMIT（包括相等）时使用 `StackOut<N>`；上限更大或未知时使用 alloc::`Vec<u8>`。传输头计入此限制，输出对象自身的长度和对齐元数据不计入。
