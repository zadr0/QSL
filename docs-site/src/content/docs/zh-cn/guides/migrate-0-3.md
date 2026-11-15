---
title: 升级到 0.3.0
description: qsl 0.3.0 的变更：拆分出 qsl-core，精简字符串表示方式。
qslResources: [{"label": "0.2.0 迁移说明", "url": "https://github.com/zadr0/QSL/blob/main/docs/migration-0.2.0.en.md"}]
---

QSL 0.3.0 把共享运行时移到了新的 crate `qsl-core` 0.1.0 中，并移除了旧的字符串 API。位图（v2）和普通字符串的编码格式没有变化。

## 同时更新版本

```toml title="Cargo.toml"
[dependencies]
qsl = "=0.4.0"

[build-dependencies]
qsl = "=0.4.0"
```

如果你直接依赖内部 crate，也要把 `qsl-lexer` 和 `qsl-interpreter` 升级到 0.3.0。属性宏仍在 `qsl-macros-impl` 0.1.0 中。

## Crate 拆分
Crate | 内容 | 依赖
---|---|---
`qsl-core` | 错误、schema 模型、共享分析、I/O、UTF-8 容器、校验、缓冲区、底层编解码 | 不依赖其他 QSL crate
`qsl-lexer` | 词法分析器、QSL 和 DEQ 解析器、文件加载、模板展开、schema 构建器 | `qsl-core`
`qsl-interpreter` | Rust 代码生成 | `qsl-lexer`
`qsl` | 门面、宏、兼容入口 | 以上全部以及 `qsl-macros-impl`
旧的模块路径以重新导出的形式继续可用。无论从哪个路径导入，`QslError` 和 `QslCustom` 都各自只是一个类型。

## 字符串

保留四种表示方式：
选择 | Rust 类型
---|---
普通字符串 | `String`
`aggressive`，上界不超过 `STACK_LIMIT` | `StackString<N>`
`decompress=true`，上界不超过 `STACK_LIMIT` | `InlineString<N>`
`lifetime` | `&str`
已移除：`binary_opt=guard`、`binary_opt=true`、`Utf8Guard`、`Utf8CopyGuard` 和字节串模式。使用它们的 schema 无法通过构建；请把每个这样的字段改为上面四种表示之一。

*新建*和*复用*是方法上的选择，不是类型。解码到已有的 `String` 之前，先给它足够的容量。见[字符串](/language/strings/)和[缓冲区与复用](/guides/memory/#reading-into-existing-memory)。

## Trait 方法改为 unsafe

`Serializable::serialize` 和 `BorrowSerializable::serialize` 现在是 `unsafe` 的：由调用方保证值对 schema 有效。请改用 `serialize_checked`。带 RT 池或启用了 `qps` 打包的类型所生成的 `serialize()` 是另一个方法，它仍然是安全的，并返回 `Result`。

## 从 0.1 升级到 0.2

如果你从 0.1 升级，请先阅读 0.2.0 迁移说明。要点：位图格式变了（v2）；`native` Cargo feature 以及 Windows 套接字和 IPC 区域已被移除；`serialize` 现在返回 `Vec<u8>`。
