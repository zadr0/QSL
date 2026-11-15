---
title: バッファと再利用
description: 自前のバッファへの書き込み、既存の値へのデコード、静的な RT スロットプール、キャッシュした出力フレーム。
---

生成器は証明されたフレームの最大サイズから出力型を選びます。STACK_LIMIT 以下なら等号を含め `StackOut<N>`、より大きい場合や上限が不明な場合は alloc::`Vec<u8>` を使います。送信ヘッダーは含み、出力オブジェクト自体の長さや整列メタデータは含みません。

String、Vec、Box は no_std 内の alloc を使い、アロケーターはアプリが提供します。InlineString のネイティブサイズが STACK_LIMIT に収まれば直接埋め込み、それ以外は Box を使います。現在の値ではなくスキーマの容量で決まります。

準備済みの unsafe reuse は呼び出し前に十分な容量を必要とし、対応する String、Vec、`Box<InlineString>` の領域を維持します。この経路では拡張しません。初期化済みオブジェクトと未初期化の出力先は別の契約です。

<a id="reading-into-existing-memory"></a>

## 例 {#example}

```qsl
&Record[safety] { u64 id; }
```

```rust
let value = Record { id: 42 };
let mut buffer = [0u8; 64];
let used = value.serialize_checked_to(&mut buffer)?;
let mut destination = core::mem::MaybeUninit::<Record>::uninit();
let restored = Record::deserialize_checked_into(&buffer[..used], &mut destination)?;
assert_eq!(restored.id, 42);
```

[STACK_LIMIT](/language/deq/) · [String](/language/strings/) · [RT](/internals/runtime/)

## QSL_RT {#rt-slot-pool}

LocalRegions · StaticRegions · FramePool · FrameStorage. 1–64 slots; lease → Drop. PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.
