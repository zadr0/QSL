---
title: '数组与位窗口'
description: '数组长度表示元素数量，字符串长度表示 UTF-8 字节数。InlineArray 将有限容量嵌入结构体，Vec 使用独立分配的内存。STACK_LIMIT 严格限制内嵌存储；固定和动态部分分别分析，不能把 Vec 描述符当作负载。'
---

数组长度按元素计数；字符串长度按 UTF-8 字节计数。变量长度 `array<u16>(0..16)` 允许零至十六个元素，固定长度 `array<u16>(16)` 要求恰好十六个。

Vec 通过 alloc 拥有独立缓冲区。InlineArray 在对象内保存有界容量，固定数组可使用 `[T;N]`。STACK_LIMIT 限制内嵌存储；Vec 描述符大小不是负载大小。

$$
B_{\mathrm{array}}=H(n)+\sum_{i=1}^{n} w_i.
$$

H 是长度元数据位数，n 是元素个数，w 是各元素编码宽度；对齐另行计算。长度不允许读取帧外数据。最多支持 32 层嵌套，元素范围与数组长度分别验证。读取寄存器按位位置选择，不固定为 128 位。

显式 `aggressive[shift=true]` 可为 InlineArray 选择合适的 NonZero 长度字。默认长度字为 u32；物理优化不改变 wire，None 与 Some(empty) 始终不同。
[Read windows](/internals/optimizer/) · [Memory](/guides/memory/)

no_std 排除 std，但不排除 alloc。qsl-core 始终使用 #![no_std]；默认 alloc 功能提供 alloc String/Vec、SerOut 和堆恢复。分配器由应用提供，QSL 不定义全局分配器。禁用默认功能即可仅使用调用方缓冲区。
