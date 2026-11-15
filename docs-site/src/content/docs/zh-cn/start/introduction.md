---
title: Quick Schema Language (QSL) 是什么
description: Quick Schema Language (QSL) 是一门 schema 语言，也是 Rust 二进制编解码器的生成器。本页说明它做什么、哪些事留给你，以及各个 crate 如何划分。
---

Quick Schema Language (QSL) 由两部分组成：一门描述二进制消息的语言，以及把这些描述转换成 Rust 代码的代码生成器。你在 schema 中写明字段类型、取值范围和长度限制。执行 `cargo build` 时，QSL 读取 schema，生成读写字节的结构体和方法。程序在运行时不解析 schema。

二进制序列化把值转换为字节，反序列化则从字节还原值。在 QSL 中，schema 定义字段，生成的 Rust 代码完成双向转换。通信双方需要一致的 schema 和属性。可以先阅读 [Rust 二进制序列化快速上手](/start/quickstart/)，再了解[线格式](/internals/wire-format/)和[位打包示例](/guides/smaller-frames/)。

```qsl
&Reading[safety,compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

QSL 根据这个 schema 生成 `Reading` 结构体，字段类型依次为 `u8`、`u8` 和 `u16`，同时生成编码、解码和校验它的方法。使用 `compact` 后，三个值共占 3 + 5 + 10 = 18 位，一帧是 3 字节而不是 4 字节。

## 你可以控制什么

- **编码布局**：整数编码后的宽度，以及帧如何标记哪些可选字段存在。
- **内存**：字符串是 `String`、借用的 `&str` 还是内联缓冲区，以及解码时是否复用已有对象。
- **校验**：`[safety]` 生成带检查的解码方法。它们在返回值之前校验边界、范围、UTF-8 和总大小预算。

## 哪些事留给应用

QSL 只定义数据如何表示。除了 `qsl::RawSocket`（一个通过 TCP 发送带长度前缀的帧的薄封装），它不处理传输，也不提供 TLS、路由或 schema 版本协商。通信双方需要相同的 schema 和相同的属性。如果修改了格式，要同时把变更发布到所有对端。QSL 不会检测一帧使用的是哪种格式。

每个生成的类型都以常量形式带有自己的 schema 哈希（见 [schema 标识](/guides/schema-identity/)）。你可以在自己的握手中发送它们，QSL 不会替你做这件事。

## Crate 划分
Crate | 版本 | 作用
---|---|---
`qsl` | 0.3.0 | 你直接依赖的 crate。重新导出运行时，提供 `build_qsl_files` 和属性宏。
`qsl-core` | 0.1.0 | 共享运行时：错误、I/O 游标、字符串容器、校验、缓冲区。
`qsl-lexer` | 0.3.0 | `.qsl` schema 和 `.deq` 配置文件的解析器，以及 schema 构建器。
`qsl-interpreter` | 0.3.0 | 根据校验过的 schema 生成 Rust 源码。
`qsl-macros-impl` | 0.1.0 | `qsl_macro` 和 `schem_macro` 属性宏，由 `qsl` 重新导出。
大多数项目只需添加 `qsl`：在 `[dependencies]` 和 `[build-dependencies]` 下各添加一次。各 crate 之间的依赖关系见[架构](/internals/architecture/)。

## 现状

QSL 0.3.0 在 Windows 11 x86-64 上使用 Rust 1.98 开发和测试，1.98 也是最低支持版本。属性都需要显式启用。属性会改变生成的代码，但没有哪个属性保证能提速。页面列出了测量了什么，以及在什么条件下测量。

下一步：[快速上手](/start/quickstart/)。
