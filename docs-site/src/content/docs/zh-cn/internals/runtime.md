---
title: '运行时与缓冲区'
description: 'SerOut 和预备缓冲区用于控制分配与复用。QSL_RT 仅为大小有界且符合条件的 Schema 创建静态槽，默认每个 Schema 16 个。全部占用时返回错误，不进行隐藏的额外分配。平台专用 socket 和内存区域不属于当前运行时。'
---

no_std 排除 std，但不排除 alloc。qsl-core 始终使用 #![no_std]；默认 alloc 功能提供 alloc String/Vec、SerOut 和堆恢复。分配器由应用提供，QSL 不定义全局分配器。禁用默认功能即可仅使用调用方缓冲区。

实现核心 FramePool/FrameStorage 的 no_std 静态内存区域；池满时立即返回 PoolExhausted。
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString 在栈内或堆上 Box 中保留同一编解码器；大型堆字符串直接恢复到目标。StackString 与 InlineArray 在对象内保存有界容量。显式 aggressive[shift=true] 可为字符串和数组选取 NonZero 长度，区分 None 和 Some(empty)。

构建脚本和测试框架可以使用 std，但不进入目标运行时。真实生成的模式在无分配器程序中执行；独立 no_std 库验证 alloc 容器。核心的两种配置和 qsl-rt 也通过 wasm32-wasip2 构建。

[Memory](/guides/memory/) · [Safety](/guides/safety/)

生成器根据已证明的帧长度上限选择输出类型。上限不超过 STACK_LIMIT（包括相等）时使用 `StackOut<N>`；上限更大或未知时使用 alloc::`Vec<u8>`。传输头计入此限制，输出对象自身的长度和对齐元数据不计入。
