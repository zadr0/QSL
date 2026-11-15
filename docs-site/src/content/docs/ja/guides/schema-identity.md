---
title: スキーマの識別
description: strict と light の 2 つのスキーマハッシュ、ハッシュに含まれる情報、ハンドシェイクでの比較方法。
---

QSL はスキーマの解析中に、型ごとに 2 つのハッシュを計算し、生成コードに定数として格納します。エンコードやデコードのときにハッシュを計算することはなく、フレームにも含まれません。

```rust
let strict: u128 = Packet::SCHEMA_HASH_STRICT;
let light: u128 = Packet::SCHEMA_HASH_LIGHT;
let h = Packet::SCHEMA_HASHES; // <Packet as qsl::SchemaIdentity>::SCHEMA_HASHES でも取得できる
println!("{} strict={} light={}", h.algorithm.name(), h.strict_hex(), h.light_hex());
```
ハッシュ | 対象
---|---
strict | 型名とフィールド名、フィールド順、型、範囲、オプションかどうか、フォーマットのオプション。
light | strict と同じ内容から、型名とフィールド名を除いたもの。
`&A { u32 id; }` と `&B { u32 sequence; }` は、light ハッシュが同じで strict ハッシュが異なります。型の異なる 2 つのフィールドを入れ替えると、両方のハッシュが変わります。

`SCHEMA_HASHES` は `qsl::SchemaHashes` 型の値です。
フィールド | 意味
---|---
`version` | ハッシュの入力フォーマットのバージョン。現在は 1。
`algorithm` | `Xxh3_64` または `Xxh3_128`。
`strict`、`light` | ハッシュ値。XXH3-64 では上位 64 ビットがゼロです。
`complete` | 型が [`QslCustom`](/guides/custom-types/#qslcustom) の型を使う場合は `false`。このとき、ハッシュが対象とするのはカスタム型の Rust パスとサイズだけで、コーデックは含まれません。
ハッシュと一緒に、`version`、`algorithm`、`complete` も比較してください。`strict_hex()` と `light_hex()` は、アルゴリズムに応じてハッシュを 16 桁または 32 桁の 16 進数に整形します。

## アルゴリズム

デフォルトは XXH3-64 です。128 ビットにするには、[DEQ ファイル](/language/deq/)で設定します。

```deq
!config USE_HASH = xxHash128;
```

この設定はビルド全体に適用されます。どちらのアルゴリズムも暗号学的ハッシュではありません。不一致は検出できますが、改ざんは検出できません。

## ハッシュが変わるもの

- 定数を評価し、Rust テンプレートを展開した後のフィールドの並び。
- 型、値の範囲、文字列と配列の上下限、どのフィールドがオプションか。
- ワイヤフォーマットのオプション：`compact`、`compact` と組み合わせた `alignment`、`optionals=bitmap`、`predict_seq`、`opt_table` と組み合わせた `predict_seq`、`opt_table=legacy`、`state_bits`。
- 文字列のコーデック：`decompress=true` と `decompress=true`。
- `qps` とそのクォンタム。`[qps]`、`[qps=Q16]`、`[quant=Q16]` は同じ意味です。
- デフォルトのオプションフィールドのフォーマットでは、オプションフィールドのタグとタグの幅。

## ハッシュが変わらないもの

`safety`、`lifetime`、`inline` とその変種、`default`、単独の `opt_table`、`compact` なしの `alignment`、`aggressive`、`transmutate`、`cache` と `loyal_cache`、`QSL_RT` とスロット数。これらが変えるのは生成される Rust コードやメモリの扱いで、バイト列は変わりません。

`String`、`&str`、`StackString`、`lifetime` は、上下限が同じなら同じバイト列を生成するので、ハッシュも同じです。

## 使い方

ハッシュはフォーマットを保守的に表したものです。異なる 2 つのスキーマが同じバイト列をデコードできないことまでは証明しません。両側の一致を確かめるには、データを送る前に、独自のハンドシェイクでハッシュを交換してください。QSL はハンドシェイクを提供しません。
