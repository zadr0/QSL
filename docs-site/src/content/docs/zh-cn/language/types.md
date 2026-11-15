---
title: 类型
description: QSL 的内置类型、它们生成的 Rust 类型，以及默认编码方式。
---

## 内置类型
Schema | Rust 类型 | 说明
---|---|---
`u8` `u16` `u32` `u64` `i8` `i16` `i32` `i64` | 相同的原始类型 |
`T(A..B)`，`T` 为整数类型 | 相同的原始类型 | 值必须在范围内。`compact` 和 `qps` 可以缩小它的编码大小。
`flags(N)` | `u8`、`u16`、`u32` 或 `u64`，由 `N` 决定 | `N` 为 1 到 64。高于 `N` 的位必须为零。
`str`、`string`、`String` | `String` | 变长，只受 `MAX_DATA_SIZE` 限制。
`str(A..B)` | `String` 或其他字符串类型 | 闭区间边界，单位为 UTF-8 字节。见[字符串](/language/strings/)。
`str(N)` | `[u8; N]` | 恰好 `N` 字节 UTF-8，编码中不含长度。`N` 超过 `STACK_LIMIT` 时为 `String`。
`array<T>`、`vec<T>` | `Vec<T>` | `T` 是内置整数类型或[已注册的原始类型](/guides/custom-types/#validated-scalar-wrappers)。
`array<T>(N)` | `[T; N]` | 恰好 `N` 个元素，编码中不含元素个数。数组超过 `STACK_LIMIT` 字节时为 `Vec<T>`。
`array<T>(A..B)` | `Vec<T>` | 元素个数为闭区间。使用 `aggressive` 且不超过 `STACK_LIMIT` 时为 `InlineArray<T, B>`。
`padding(N)` | `()` | 编码中为 `N` 个零字节。
`T?` | `Option<T>` | 可选字段。数值型可选字段通过访问方法读写，见[声明](/language/declarations/#optional-fields-in-rust)。
其他任意名字 | 你的 Rust 类型 | 必须实现 `QslCustom`，见[自定义类型](/guides/custom-types/)。
使用 `lifetime` 属性时，字符串和 `u8` 数组以 `&'a str` 和 `&'a [u8]` 的形式从输入借用。

## 默认编码

没有改变格式的属性时：

- **整数**：原生宽度，小端序。`u16(0..1023)` 占 2 字节。
- **字符串**：`u32` 字节长度，后跟 UTF-8 字节。固定长度的 `str(N)` 没有长度。
- **动态数组**：`u32` 元素个数，后跟各元素。固定数组没有元素个数。
- **必填字段**：按声明顺序排列。
- **可选字段**：在值之前加上存在信息元数据。见[可选字段](/language/optionals/)。

帧中不携带类型信息，双方都必须知道 schema。

## 编码大小与对象大小

两者相互独立：

```qsl
&Reading[compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

编码后是 3 + 5 + 10 = 18 位，写成 3 字节。在内存中，`Reading` 是包含 `u8`、`u8` 和 `u16` 的 `#[repr(C)]` 结构体，占 4 字节。`Reading::NATIVE_BYTES` 保存的就是这个大小。
