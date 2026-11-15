---
title: 字符串与所有权
description: QSL 字符串可以生成哪些 Rust 类型，各自在什么情况下被选用，以及解码时如何处理内存。
---

UTF-8 按字节计数：“Яa”占 3 B。String 拥有动态内存，&str 借用仍存活的输入，StackString 和 InlineString 使用有界存储。QSL-LZ 只有在编码确实更小时才选择压缩，否则保留 raw。图展示字节分布与实际字面量、回溯引用。

## 存储与长度

InlineString 在栈内或堆上 Box 中保留同一编解码器；大型堆字符串直接恢复到目标。StackString 与 InlineArray 在对象内保存有界容量。显式 aggressive[shift=true] 可为字符串和数组选取 NonZero 长度，区分 None 和 Some(empty)。

## QSL-LZ {#compressed-strings}

解码器逐个验证展开 token、回溯距离、UTF-8 和累计预算。头中的存储长度不能授权更大的解压输出。长度以 UTF-8 字节计算，不是 Unicode 字符数。
[Arrays](/language/arrays/) · [Limits](/reference/limits/) · [Memory](/guides/manual-layout/) · [QPS](/guides/smaller-frames/)

no_std 排除 std，但不排除 alloc。qsl-core 始终使用 #![no_std]；默认 alloc 功能提供 alloc String/Vec、SerOut 和堆恢复。分配器由应用提供，QSL 不定义全局分配器。禁用默认功能即可仅使用调用方缓冲区。

生成器根据已证明的帧长度上限选择输出类型。上限不超过 STACK_LIMIT（包括相等）时使用 `StackOut<N>`；上限更大或未知时使用 alloc::`Vec<u8>`。传输头计入此限制，输出对象自身的长度和对齐元数据不计入。
