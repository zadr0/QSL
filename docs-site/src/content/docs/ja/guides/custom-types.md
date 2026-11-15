---
title: Rust テンプレートとカスタム型
description: Rust の構造体をフィールドのグループとして再利用し、検証付きの整数ラッパーを登録し、QslCustom を実装し、ネストしたリージョンを宣言します。
---

自前の Rust 型をスキーマに取り込む方法は 4 つあります。それぞれが解決する問題は異なります。
仕組み | 使う場面
---|---
`#[qsl::qsl_macro]` テンプレート | 同じフィールドのグループが複数のスキーマに現れる。
`qsl::qsl_primitive!` | `DeviceId(u32)` のような、整数の newtype がある。
`QslCustom` | バイト列を完全に制御する必要がある。
`#region` | ネイティブレイアウトが固定された、ネストした構造体が必要。
## Rust テンプレート

Rust の構造体をテンプレートとしてマークし、その名前をフィールドの型として使います。

```rust title="src/meta.rs"
#[qsl::qsl_macro]
pub struct Meta { pub id: u32, pub tick: u64 }
```

```qsl
&Packet { Meta meta; u16 code; }
```

生成される `Packet` は、フラットなフィールド `meta_id`、`meta_tick`、`code` を持ちます。`#[qsl::schem_macro]` は、同じマーカーの別名です。

テンプレートのフィールドに使えるのは、`u8`〜`u64`、`i8`〜`i64`、`String`、`&str`、`qsl::InlineString<N>`、整数の固定長配列、これらの `Option<T>`、そしてほかのテンプレートです。テンプレートは 64 階層までネストできます。循環と名前の衝突はビルドエラーです。テンプレートへの参照はオプションにできず、デフォルト値も持てません。代わりに、テンプレート内の個々のフィールドを `Option<T>` にしてください。

`build.rs` では、`build_qsl_files_with_rust` に Rust ファイルを列挙します。この関数を使うには、ビルド依存関係の `qsl` で `rust-macros` フィーチャーを有効にする必要があります。

```toml title="Cargo.toml"
[build-dependencies]
qsl = { version = "=0.4.0", features = ["rust-macros"] }
```

```rust title="build.rs"
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var("OUT_DIR")?;
    qsl::build_qsl_files_with_rust(&["packet.qsl"], &[], &["src/meta.rs"], &out)?;
    Ok(())
}
```

## 検証付きスカラーラッパー {#validated-scalar-wrappers}

`qsl::qsl_primitive!` は、`u8`/`i8`、`u16`/`i16`、`u32`/`i32`、`u64`/`i64` のいずれかを包む、値の範囲付きの newtype を宣言します。

```rust title="primitive.rs"
qsl::qsl_primitive!(pub struct DeviceId(u32); range=(1,100););
```

マクロはコンパイル時に、ラッパーのサイズとアラインメントが整数と同じであること、整数がオフセット 0 にあること、型が `Drop` を持たないこと、範囲が整数に収まることをチェックします。デフォルトのコーデックは、整数をリトルエンディアンで書き込みます。`compact` のコーデックは、`value - min` を範囲を表せる最小のバイト数で格納します。値が 1 つだけの範囲は 0 バイトです。

ビルドスクリプトからこの型が見える必要があるので、`build.rs` とクレートの両方がインクルードするファイルに置いてください。スキーマビルダーに登録してから、コードを生成します。

```rust title="build.rs"
mod primitive {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/primitive.rs"));
}
use qsl::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rust = QslSchemaBuilder::new()
        .with_primitive::<primitive::DeviceId>("DeviceId")?
        .with_source("&Packet[safety]{DeviceId id; DeviceId? extra;}")?
        .use_interpreter()?;
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("qsl_gen.rs");
    std::fs::write(out, rust)?;
    Ok(())
}
```

このビルダーを使う方法では DEQ ファイルを読まず、デフォルトの設定を使います。登録したラッパーは配列の要素にもできます。[ネストしたリージョン](#nested-regions)の中で使う場合は、クレート内で `qsl::qsl_native_primitive!(DeviceId, u32);` も呼び出してください。

## QslCustom {#qslcustom}

組み込みのコーデックがどれも合わない場合は、`qsl::QslCustom` を実装します。
項目 | 必須 | 意味
---|---|---
`SIZE` | はい | エンコード後のサイズ（バイト）。
`COMP_SIZE` | いいえ | `compact` でのエンコード後のサイズ。デフォルトは `SIZE`。
`qsl_write_unchecked`、`qsl_read_unchecked` | はい | 生ポインタを扱う `unsafe` なコーデック。
`qsl_comp_write_unchecked`、`qsl_comp_read_unchecked` | いいえ | `compact` 用のコーデック。デフォルトは通常のコーデック。
`qsl_validate(&[u8], compact)` | チェック付きデコードに必要 | チェック付きデコーダがリーダーを呼ぶ前に、バイト列をチェックします。
`qsl_validate_values(&self)` | チェック付きエンコードに必要 | `serialize_checked` がライターを呼ぶ前に、値をチェックします。
どちらのバリデータもデフォルトでは `InvalidSchema` を返すため、実装するまで、チェック付きメソッドはカスタム型を拒否します。ちょうど `SIZE` バイトを書き込むことと、有効な Rust の値を生成することは、利用者の `unsafe` コードの責任です。

スキーマがカスタム型を使う場合、生成ファイルは `use ::qsl::QslCustom;` で始まります。生成ファイルをインクルードするモジュールで、`QslCustom` を再度インポートしないでください。また、QSL はカスタム型のコーデックを見られないため、カスタム型を使うと[スキーマハッシュ](/guides/schema-identity/)は不完全になります。

## ネストしたリージョン {#nested-regions}

リージョンは、連続するフィールドを独立した `#[repr(C)]` 構造体にまとめます。

```qsl
&Parent[safety] {
    #region(aggressive_transmutate="Meta");
    u64 id;
    u32 value;
    str(4) tag;
    #end_region;
    str(3..10) name;
}
```

このスキーマから、`Meta { id: u64, value: u32, tag: [u8; 4] }` と `Parent { Meta: Meta, name: String }` が生成されます。親のフィールドには、リージョンの型名がそのまま付きます。リージョンのフィールドは、親にフラット化されません。

リージョンに含められるのは固定サイズのフィールド、つまり整数、固定長文字列、固定長配列、登録済みのラッパーだけです。オプションフィールドは拒否されます。リージョンは 64 階層までネストできますが、互いに交差させることはできません。`compact` または `qps` を持つ型はリージョンを持てません。
