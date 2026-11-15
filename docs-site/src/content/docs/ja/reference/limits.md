---
title: "制限とソース"
description: "MAX_DATA_SIZE の既定値は 65535 B、STACK_LIMIT は 256 B です。前者は入力・復元・合計データ、後者は内蔵文字列・配列を制限します。RSS の上限ではありません。dyn index16 は未対応です。LLVM バックエンドと QBProto ハンドシェイクは将来計画で、不正な組み合わせは生成時に拒否します。"
qslResources: [{"label": "詳細リファレンスとソース", "url": "https://github.com/zadr0/QSL"}]
---

ローカル開発版 · 2026-11-14.

MAX_DATA_SIZE の既定値は 65535 B、STACK_LIMIT は 256 B です。前者は入力・復元・合計データ、後者は内蔵文字列・配列を制限します。RSS の上限ではありません。dyn index16 は未対応です。LLVM バックエンドと QBProto ハンドシェイクは将来計画で、不正な組み合わせは生成時に拒否します。

詳細リファレンスとソース · [English](/en/reference/limits/)
