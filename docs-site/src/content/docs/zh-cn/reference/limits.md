---
title: "限制与源码"
description: "MAX_DATA_SIZE 默认为 65535 B，限制输入、恢复及累计数据预算。STACK_LIMIT 默认为 256 B，限制内嵌字符串和数组。它们不是进程 RSS 限制。dyn index16 尚不支持；LLVM 后端和 QBProto 握手仍是未来工作。不支持的组合会明确报错。"
qslResources: [{"label": "详细参考与源码", "url": "https://github.com/zadr0/QSL"}]
---

本地开发版本 · 2026-11-14.

MAX_DATA_SIZE 默认为 65535 B，限制输入、恢复及累计数据预算。STACK_LIMIT 默认为 256 B，限制内嵌字符串和数组。它们不是进程 RSS 限制。dyn index16 尚不支持；LLVM 后端和 QBProto 握手仍是未来工作。不支持的组合会明确报错。

详细参考与源码 · [English](/en/reference/limits/)
