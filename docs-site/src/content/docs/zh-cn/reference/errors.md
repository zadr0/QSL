---
title: 错误
description: QslError 的各个变体及其返回时机。
---

生成的方法返回 `qsl::QslResult<T>`，它是 `Result<T, QslError>` 的别名。`QslError` 有 11 个变体，并实现了 `std::error::Error`。它的 `Display` 输出为 `QSL error: <Variant>`，例如 `QSL error: InvalidUtf8`。
变体 | 返回时机
---|---
`UnexpectedEof` | 帧还没结束，输入就结束了。
`BufferTooSmall` | 传入的缓冲区放不下输出。
`LengthOverflow` | 长度计算会溢出。
`InvalidWidth` | 不支持的宽度或 quantum。
`InvalidValue` | 值、长度或元数据字段超出允许的取值范围。
`InvalidUtf8` | 字符串字节不是有效的 UTF-8。
`InvalidSchema` | Schema 无效，或该操作不适用于它。也是 `QslCustom` 校验函数的默认结果。
`InvalidPadding` | 填充位或填充字节不为零。
`PoolExhausted` | 所有 RT 槽位都已被占用。只由不等待的 `try_acquire` 返回。
`SizeLimitExceeded` | 帧或解码后的数据超过 `MAX_DATA_SIZE`。
`AllocationFailed` | 可失败分配失败。
文件和套接字操作还可能返回 `std::io::Error`。

校验通过只说明字节符合 schema 的结构，并不说明它们是谁发送的，也不说明它们是按哪个 schema 写入的。
