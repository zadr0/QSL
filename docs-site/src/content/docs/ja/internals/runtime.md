---
title: 'ランタイムとバッファ'
description: 'SerOut と準備済みバッファで割り当てと再利用を管理します。QSL_RT は上限が確定した適格なスキーマだけに静的スロットを作り、既定は各スキーマ 16 個です。満杯なら隠れた割り当てをせずエラーを返します。プラットフォーム専用ソケットとメモリ領域は現在のランタイムに含まれません。'
---

no_std が除外するのは std であり、alloc ではありません。qsl-core は常に #![no_std] です。既定の alloc 機能は alloc String/Vec、SerOut、ヒープ復元を提供します。アロケータはアプリが用意し、QSL はグローバルアロケータを定義しません。既定機能を無効にすると呼び出し側のバッファだけを使えます。

コアの FramePool/FrameStorage を実装する no_std 静的領域。満杯なら直ちに PoolExhausted。
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString はスタック内でもヒープの Box 内でも同じコーデックを保ちます。大きなヒープ文字列は出力先へ直接復元します。StackString と InlineArray は容量をオブジェクト内に保持します。明示的な aggressive[shift=true] は NonZero 長を選び、None と Some(empty) を区別します。

ビルドスクリプトとテストは std を使えても、ターゲットのランタイムには含まれません。実際の生成スキーマをアロケータなしで実行し、別の no_std ライブラリで alloc コンテナを検証します。コアの両設定と qsl-rt は wasm32-wasip2 でもビルドします。

[Memory](/guides/memory/) · [Safety](/guides/safety/)

生成器は証明されたフレームの最大サイズから出力型を選びます。STACK_LIMIT 以下なら等号を含め `StackOut<N>`、より大きい場合や上限が不明な場合は alloc::`Vec<u8>` を使います。送信ヘッダーは含み、出力オブジェクト自体の長さや整列メタデータは含みません。
