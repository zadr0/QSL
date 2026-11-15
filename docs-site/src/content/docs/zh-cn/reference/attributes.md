---
title: 属性
description: QSL 的所有类型属性和字段修饰符、它们生成什么，以及是否改变编码格式。
---

类型属性写在类型名后的方括号中：`&Packet[safety,compact]`。字段修饰符写在字段名后的两个竖线之间：`str(1..100) name;`。它们都是可选的，每个最多出现一次。

**编码**一栏表示该属性是否改变字节，从而是否改变 [schema 哈希](/guides/schema-identity/)。如果改变，两端必须使用相同的设置。

## 一览
属性 | 位置 | 编码 | 作用
---|---|---|---
[`safety`](#safety) | 类型 | 否 | 生成带检查的解码方法。
[`compact`](#compact) | 类型 | 是 | 用最少的位存储带范围的值。
[`alignment`](#alignment) | 类型 | 与 `compact` 同用时 | 让每个 compact 字段从字节边界开始。
[`optionals=bitmap`](#optionalsbitmap) | 类型 | 是 | 覆盖可选字段的存在掩码。
[`predict_seq`](#predict_seq) | 类型 | 是 | 由载荷长度推导字段是否存在。
[`opt_table`](#opt_table) | 类型 | 仅 `legacy` | 可选字段读取函数的静态表。
[`qps`、`quant`](#qps-quant) | 类型 | 是 | 数值字段的可逆打包。
[`aggressive`](#aggressive) | 类型 | 否 | 有界字符串和数组使用内联存储。
[`lifetime`](#lifetime) | 类型 | 否 | 从输入借用字符串和字节。
[`transmutate`](#transmutate) | 类型 | 否 | 原生布局与编码布局一致。
[`inline`](#inline) | 类型 | 否 | 生成方法的 `#[inline]` 策略。
[`slots=N`](#slotsn) | 类型 | 否 | 该类型的 RT 槽位数。
[`cache=N`、`loyal_cache`](#cachen-loyal_cache) | 类型 | 否 | 输出到调用方提供的 `CachedFrame`。
[`state_bits=Qn`](#state_bitsqn) | 类型 | 是 | 旧版的存在索引格式。
[`decompress=true`](#decompresstrue) | 字段 | 是 | 对单个字符串做有界 LZ 压缩。
[`default=N`](#defaultn) | 字段 | 否 | 生成 `impl Default`。
## 类型属性

### safety {#safety}

生成用于解码不可信输入的 `deserialize_checked` 和 `deserialize_checked_into`。它们在返回值之前检查边界、范围、UTF-8、可选字段元数据、填充、解压，以及 `MAX_DATA_SIZE` 总预算。无检查方法仍然可用。无论有没有 `safety`，都会生成 `serialize_checked`、`serialize_checked_to`、`validate` 和 `validate_values`。见[校验输入](/guides/safety/)。

### compact {#compact}

把每个带范围的整数存储为 `x - min`，占 `bit_length(max - min)` 位，可跨字节边界打包。`u8(0..7)`、`u8(0..31)` 和 `u16(0..1023)` 分别占 3、5 和 10 位。字符串和数组的长度按各自的边界以同样方式存储。可与 `opt_table` 和 `optionals=bitmap` 一起使用。不能与 `predict_seq`、`state_bits` 或 `qps` 同时使用。见[缩小帧体积](/guides/smaller-frames/)。

### alignment {#alignment}

与 `compact` 一起使用时，让每个字段从字节边界开始，帧可能因此变大。不使用 `compact` 时不改变格式。它不是缓冲区的内存对齐。不能与 `qps` 同时使用。

### optionals=bitmap {#optionalsbitmap}

写入一个掩码，每个可选字段占一位，宽度根据可选字段数为 0 到 16 字节。最多支持 128 个可选字段。与 0.2.0 之前使用的掩码不兼容。不能与 `state_bits`、`opt_table=legacy` 或 `qps` 同时使用。见[可选字段](/language/optionals/#bitmap)。

### predict_seq {#predict_seq}

根据载荷长度推断哪些可选字段存在。要求每种存在字段的组合都对应不同的长度。设置了 `optionals=bitmap` 时被忽略。不能与 `compact` 同时使用。见[可选字段](/language/optionals/#length-index-predict_seq)。

### opt_table {#opt_table}

`opt_table`（或 `opt_table=static`）通过静态表而不是 `match` 分派可选字段的读取函数。字节相同，哈希也相同。与 `predict_seq` 组合时，选择 0.2.0 之前 `[opt_table]` 生成的长度图格式。

`opt_table=legacy` 是另一种格式。它在必填字段之后写入一个计数字节，然后为每个可选字段写入一个小端序 `u32`：字段不存在时为 `0`，否则为该值在可选载荷中的偏移加 1。它不能与普通的 `opt_table`、`predict_seq`、`state_bits`、`optionals=bitmap` 或 `qps` 同时使用。

### qps、quant {#qps-quant}

`qps` 即 `qps=Q16`。`qps=Q2`、`Q4`、`Q8` 和 `Q16` 设置 quantum。`quant=Qn` 是别名。不带值的 `quant` 是错误，同时使用两个名字也是错误。只打包数值字段：字符串、数组及其长度沿用各自的编解码方式，并且数值字段必须排在它们前面。不能与 `compact`、`alignment`、`optionals=bitmap` 或 `opt_table=legacy` 同时使用。

没有 RT 池时，启用了打包的类型会得到一个安全的 `serialize()`，返回 `QslResult<QuantizedOut>`。见[缩小帧体积](/guides/smaller-frames/)。

### aggressive {#aggressive}

改变 Rust 中的存储方式，不改变字节，也不改变接受哪些输入：

- 当最大值不超过 `STACK_LIMIT` 时，`str(min..max)` 变为 `StackString<max>`，`array<T>(min..max)` 变为 `InlineArray<T, max>`。更大的字段仍是 `String` 或 `Vec`。
- 可选数值字段存储为能容纳其范围的最小宽度的 `NonZero` 整数，前提是这个宽度不超过字段类型本身的宽度。

### lifetime {#lifetime}

为类型加上生命周期 `'a` 并从输入借用：字符串变为 `&'a str`，`array<u8>` 变为 `&'a [u8]`。输入必须比值活得更久。其他元素类型的数组和压缩字符串无法借用，会导致构建失败；没有任何可借用内容的 `lifetime` 类型同样会导致构建失败。

### transmutate {#transmutate}

让类型的原生布局与编码布局一致。每个字段都必须是固定大小：`compact`、`qps`、可选字段以及带范围的字符串或数组都会导致构建错误。编码大小必须等于结构体大小，所以要在 Rust 插入填充的位置加上 `padding(N)` 字段。

不带 `lifetime` 时，如果 `NATIVE_WIRE_COMPATIBLE` 为 true，`deserialize_transmutate_copy` 会把帧整块复制到一个拥有所有权的值中，否则逐字段解码。带 `[safety]` 时它是安全的，并会校验结果；不带 `[safety]` 时它是 `unsafe` 的。

带 `lifetime` 时，`deserialize_overlay` 返回指向输入内部的 `&Self`。构建时会检查每个编码偏移是否等于原生偏移，并且在大端目标上构建失败。带 `[safety]` 时，`deserialize_overlay` 先检查 `NATIVE_WIRE_COMPATIBLE`、输入长度和对齐，再检查各个值，`deserialize_checked` 也做同样的检查。不带 `[safety]` 时，`deserialize_overlay` 是 `unsafe` 的，不做任何检查。

两者都有跳过检查的 `_unchecked` 变体。

### inline {#inline}

`inline`（等同于 `inline=hint`）、`inline=always` 或 `inline=never` 会给生成的方法加上对应的 inline 属性。不设置时，生成的方法不带 inline 属性，由编译器决定。`always` 可能增大代码体积，并不保证提速。

### slots=N {#slotsn}

覆盖该类型的 RT 槽位数，1 到 64。仅在设置了 `!config QSL_RT = true;` 时生效。见 [RT 槽位池](/guides/memory/#rt-slot-pool)。

### cache=N、loyal_cache {#cachen-loyal_cache}

生成 `CACHE_BYTES` 和 `serialize_cached`，后者写入调用方提供的 `qsl::modernization::CachedFrame<N>`。`loyal_cache` 把 `CACHE_BYTES` 设为最大帧大小。类型的最大帧大小必须有限，`N` 小于它时构建失败。它准备的是输出内存，不缓存值。见[缓存的输出帧](/guides/memory/#cached-output-frames)。

### state_bits=Qn {#state_bitsqn}

取值为 `Q1`（默认）、`Q2`、`Q4`、`Q8` 或 `Q16`。选择一种旧版的存在索引格式。它不是压缩模式。不能与 `optionals=bitmap`、`compact` 或 `opt_table=legacy` 同时使用。

## 字段修饰符

### binary_opt {#binary_opt}

binary_opt 已全部移除。使用 [lifetime] 借用 &str、[aggressive] 选择有界 StackString，或 decompress=true 选择 InlineString。STACK_LIMIT 对全部字符串和数组生效。

### decompress=true {#decompresstrue}

用 QSL 的有界 LZ 编解码器编码该字符串。短于 32 字节的字符串，以及压缩后不会变短的字符串，按原样存储。每个解码出的 token 都会对照已还原的大小、目标容量和 `MAX_DATA_SIZE` 进行检查。字符串的上界不能超过 65535 字节。带范围的字符串变为 `InlineString<max>`，超过 `STACK_LIMIT` 时为 `String`。不能与 `lifetime` 或 `lifetime` 同时使用。见[压缩字符串](/language/strings/#compressed-strings)。

### default=N {#defaultn}

为类型生成 `impl Default`。该字段的初始值为 `N`，如果是可选字段则为 `Some(N)`。其他字段的初始值为各自的最小有效值。`N` 必须在字段范围内，且字段必须是数值类型。如果类型中还有自定义类型的字段，构建会失败。它不改变字节，也不改变哈希。
