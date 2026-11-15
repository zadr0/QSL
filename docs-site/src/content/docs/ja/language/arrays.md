---
title: '配列とビット窓'
description: '配列の長さは要素数、文字列の長さは UTF-8 のバイト数です。InlineArray は有限容量を構造体内に保持し、Vec は別の領域を確保します。STACK_LIMIT は内蔵格納を厳密に制限します。Vec の管理情報をペイロードとしてコピーしません。'
---

配列長は要素数、文字列長は UTF-8 のバイト数です。可変長 `array<u16>(0..16)` は 0〜16 要素、固定長 `array<u16>(16)` は正確に 16 要素を要求します。

Vec は alloc による別バッファを所有します。InlineArray は容量をオブジェクト内に持ち、固定配列には `[T;N]` を使えます。STACK_LIMIT は埋め込みを制限します。Vec の記述子サイズはペイロードサイズではありません。

$$
B_{\mathrm{array}}=H(n)+\sum_{i=1}^{n} w_i.
$$

H は長さメタデータのビット数、n は要素数、w は各要素の符号幅です。整列は別計算です。長さはフレーム外の読み取りを許可しません。入れ子は 32 層までで、要素範囲と長さを個別に検証します。読み取りレジスタはビット位置に応じて選び、常に 128 ビットではありません。

明示的な `aggressive[shift=true]` は InlineArray に適切な NonZero 長を選べます。既定は u32 です。物理最適化は wire を変えず、None と Some(empty) を区別します。
[Read windows](/internals/optimizer/) · [Memory](/guides/memory/)

no_std が除外するのは std であり、alloc ではありません。qsl-core は常に #![no_std] です。既定の alloc 機能は alloc String/Vec、SerOut、ヒープ復元を提供します。アロケータはアプリが用意し、QSL はグローバルアロケータを定義しません。既定機能を無効にすると呼び出し側のバッファだけを使えます。
