---
title: 声明
description: QSL schema 的语法：声明、字段、属性、修饰符、范围、常量表达式和注释。
---

一个 schema 文件包含一个或多个声明，每个声明生成一个 Rust 类型。

```qsl
&Name[attribute, attribute=value] {
    type field;
    type field|modifier=value|;
    type? optional_field;
}
```
部分 | 含义
---|---
`&Name` | 声明名为 `Name` 的类型。
`[ ... ]` | 作用于整个类型的属性，可省略。见[属性参考](/reference/attributes/)。
`type field;` | 必填字段。
`type? field;` | 可选字段。有范围时，`?` 写在范围前面：`u16?(0..1023) level;`。
`\|...\|` | 单个字段的修饰符，写在字段名和分号之间。
## Rust 中的可选字段 {#optional-fields-in-rust}

可选的字符串或数组字段在结构体中是公有的 `Option<T>` 字段。

可选的数值字段存放在私有字段中。用同名方法读取，用 `set_<name>` 修改，值超出字段范围时它返回 `InvalidValue`。含有这类字段的类型会生成 `try_new` 构造函数，它按声明顺序接收所有字段并检查取值：

```qsl
&Reading[safety] { u32 id; u8? level; }
```

```rust
let mut r = Reading::try_new(42, Some(7))?;
assert_eq!(r.level(), Some(7));
r.set_level(None)?;
```

## 字段顺序

必填字段必须位于可选字段之前。无论有没有属性，QSL 对所有类型都强制执行这条规则。

```qsl
&Ok    { u32 id; u16? code; }
&Error { u16? code; u32 id; }   // 构建错误：必填字段在可选字段之后
```

必填字段的值按声明顺序写入帧。可选字段值的顺序取决于[可选字段模式](/language/optionals/)。

## 标签

编译器按声明顺序为每个字段分配标签，从 1 开始。标签不能手动指定。可选字段的默认编码会存储标签（见[可选字段](/language/optionals/)），因此调整字段顺序会改变格式。

未使用 `optionals=bitmap` 的类型最多有 64 个字段。

## 范围

`A..B` 是闭区间，含义取决于类型：

```qsl
&Limits {
    u8(1..3)          kind;    // 取值范围：1、2 或 3
    str(1..100)       name;    // 1 到 100 字节的 UTF-8
    str(4)            tag;     // 恰好 4 字节，生成 [u8; 4]
    array<u16>(0..16) samples; // 0 到 16 个元素
}
```

带检查的方法会强制执行取值范围。使用 `compact` 时，范围还决定字段占多少位。字符串的范围按 UTF-8 字节计，而不是按字符计：`"Ада"` 是 6 字节。

## 常量表达式

边界可以是由整数和具名常量组成的表达式：

```qsl
&Frame {
    str(1..MAX_NAME) name;
    array<u8>(0..MAX_NAME * 2) payload;
}
```

表达式支持 `+ - * / %`、一元负号、括号、作用于 `u8` 到 `i64` 整数类型的 `sizeof(T)`，以及具名常量。常量可以来自 [DEQ 文件](/language/deq/)中的 `!define`，也可以来自类型体内的 `#NAME = expression;` 行，后者定义的常量对其后的行有效。溢出和除以零都是构建错误。

## 注释

`//` 开始行注释，`/* ... */` 是块注释。

```qsl
&Packet[safety] {
    u8 id;      // 消息类型
    u16? a;     /* 大多数帧中没有这个字段 */
}
```

## 哪些写法会被拒绝

以下情况会导致构建失败：

- 必填字段位于可选字段之后；
- 字段名或类型名重复，字段名是 Rust 关键字或以 `__qsl_` 开头；
- 边界无效或上下界颠倒，范围超出整数类型的表示范围；
- 未知或重复的属性和修饰符；
- 未使用 `optionals=bitmap` 时超过 64 个字段，或使用它时超过 128 个可选字段。

非内置的类型名会被当作[自定义类型](/guides/custom-types/)，需要提供 Rust 实现。自定义类型不能带范围。内置类型列表见[类型](/language/types/)。
