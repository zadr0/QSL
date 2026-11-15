---
title: DEQ 設定
description: DEQ ファイルには、型付きの定数と、MAX_DATA_SIZE、STACK_LIMIT、RT プールなどビルド全体に適用される設定を書きます。
---

`.deq` ファイルには 2 種類のディレクティブを書きます。スキーマの上下限で使える型付きの定数と、コード生成の設定です。DEQ ファイルは `build_qsl_files` の 2 番目の引数に渡します。

```rust title="build.rs"
qsl::build_qsl_files(&["schemas/packet.qsl"], &["schemas/config.deq"], out)?;
```

## 定数

```deq title="config.deq"
!define MAX_NAME: u16 = 100;
```

型は `u8`〜`u64`、`i8`〜`i64`、`usize`、`isize` のいずれかで、値はその型に収まる必要があります。式ではスキーマと同じ演算子を使え、整数型に対する `sizeof(T)` も使えます。後の定数は、前の定数を `!NAME` として参照できます。

```deq title="config.deq"
!define HEADER: usize = sizeof(u64) + 2;
!define FRAME: usize = !HEADER * 4;
```

スキーマでは、定数を名前で参照します。

```qsl
&User[safety] { u64 id; str(1..MAX_NAME) name; }
```

## 設定

```deq title="config.deq"
!config MAX_DATA_SIZE = 65535;
!config USE_HASH = xxHash;
```
設定 | デフォルト | 意味
---|---|---
`MAX_DATA_SIZE` | `65535` | フレーム、圧縮から復元したデータ、デコードした値の合計サイズに対する上限（バイト）。1〜`isize::MAX`。
`STACK_LIMIT` | `256` | 文字列または配列のバッファ 1 つあたりのインライン容量の上限（バイト）。上限がこれより大きいフィールドは `String` または `Vec` になります。`0`〜`u32::MAX`。
`USE_HASH` | `xxHash` | スキーマハッシュのアルゴリズム。`xxHash` は XXH3-64、`xxHash128` は XXH3-128 です。`XXH3_64` と `XXH3_128` も指定できます。
`QSL_RT` | `false` | `true` にすると、最大フレームサイズが有限の型に静的なスロットプールを生成します。[バッファと再利用](/guides/memory/#rt-slot-pool)を参照してください。
`QSL_RT_DEFAULT` | `16` | 型ごとのスロット数。1〜64。数えるのはバイト数ではなくスロット数です。
設定はビルド全体に適用されます。同じ名前を 2 回設定するとエラーになります。別々のファイルで設定した場合も同じです。未知のディレクティブや未知の設定名もエラーです。

DEQ ファイルにはファイルパスを書きません。どのファイルをコンパイルするかは `build.rs` が決めます。
