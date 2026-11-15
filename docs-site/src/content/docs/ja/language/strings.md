---
title: 文字列と所有権
description: QSL の文字列がなりうる Rust の型、それぞれが選ばれる条件、デコード時のメモリの扱い。
---

UTF-8 はバイトで数えます。「Яa」は 3 B です。String は動的メモリを所有し、&str は生存する入力を借用します。StackString と InlineString は有界格納です。QSL-LZ は実際に小さくなる場合だけ圧縮し、それ以外は raw を使います。分布と実際のリテラル・後方参照を図で確認できます。

## 格納方式と長さ

InlineString はスタック内でもヒープの Box 内でも同じコーデックを保ちます。大きなヒープ文字列は出力先へ直接復元します。StackString と InlineArray は容量をオブジェクト内に保持します。明示的な aggressive[shift=true] は NonZero 長を選び、None と Some(empty) を区別します。

## QSL-LZ {#compressed-strings}

各展開 token、後方参照距離、UTF-8、合計予算を検査します。格納長ヘッダーは大きな復元先を許可しません。長さは Unicode 文字数ではなく UTF-8 バイト数です。
[Arrays](/language/arrays/) · [Limits](/reference/limits/) · [Memory](/guides/manual-layout/) · [QPS](/guides/smaller-frames/)

no_std が除外するのは std であり、alloc ではありません。qsl-core は常に #![no_std] です。既定の alloc 機能は alloc String/Vec、SerOut、ヒープ復元を提供します。アロケータはアプリが用意し、QSL はグローバルアロケータを定義しません。既定機能を無効にすると呼び出し側のバッファだけを使えます。

生成器は証明されたフレームの最大サイズから出力型を選びます。STACK_LIMIT 以下なら等号を含め `StackOut<N>`、より大きい場合や上限が不明な場合は alloc::`Vec<u8>` を使います。送信ヘッダーは含み、出力オブジェクト自体の長さや整列メタデータは含みません。
