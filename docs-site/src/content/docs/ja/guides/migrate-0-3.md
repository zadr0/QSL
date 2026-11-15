---
title: 0.3.0 へのアップグレード
description: qsl 0.3.0 の変更点。qsl-core の分離と、文字列表現の整理。
qslResources: [{"label": "0.2.0 の移行ノート", "url": "https://github.com/zadr0/QSL/blob/main/docs/migration-0.2.0.en.md"}]
---

QSL 0.3.0 は共有ランタイムを新しいクレート `qsl-core` 0.1.0 に移し、古い文字列 API を削除しました。ビットマップ（v2）と通常の文字列のワイヤフォーマットは変わりません。

## バージョンをまとめて更新する

```toml title="Cargo.toml"
[dependencies]
qsl = "=0.4.0"

[build-dependencies]
qsl = "=0.4.0"
```

内部のクレートに直接依存している場合は、`qsl-lexer` と `qsl-interpreter` も 0.3.0 に更新してください。属性マクロは引き続き `qsl-macros-impl` 0.1.0 にあります。

## クレートの分割
クレート | 内容 | 依存先
---|---|---
`qsl-core` | エラー、スキーマモデル、共通の解析処理、I/O、UTF-8 コンテナ、検証、バッファ、低レベルのコーデック | ほかの QSL クレートには依存しない
`qsl-lexer` | レキサ、QSL と DEQ のパーサ、ファイルの読み込み、テンプレートの展開、スキーマビルダー | `qsl-core`
`qsl-interpreter` | Rust コードの生成 | `qsl-lexer`
`qsl` | ファサード、マクロ、互換性のあるエントリポイント | 上記すべてと `qsl-macros-impl`
古いモジュールパスは、再エクスポートとして引き続き使えます。`QslError` と `QslCustom` は、どのパスからインポートしても同じ 1 つの型です。

## 文字列

残る表現は次の 4 つです。
選択 | Rust の型
---|---
通常の文字列 | `String`
`aggressive`、上限が `STACK_LIMIT` 以内 | `StackString<N>`
`decompress=true`、上限が `STACK_LIMIT` 以内 | `InlineString<N>`
`lifetime` | `&str`
削除されたもの：`binary_opt=guard`、`binary_opt=true`、`Utf8Guard`、`Utf8CopyGuard`、バイト文字列モード。これらを使うスキーマはビルドに失敗します。各フィールドを、上の 4 つの表現のいずれかに切り替えてください。

新規作成（*new*）と再利用（*reuse*）は型ではなく、メソッドの選択です。既存の `String` にデコードする前に、十分な容量を確保してください。[文字列](/language/strings/)と[バッファと再利用](/guides/memory/#reading-into-existing-memory)を参照してください。

## トレイトのメソッドが unsafe に

`Serializable::serialize` と `BorrowSerializable::serialize` は `unsafe` になりました。値がスキーマに対して有効であることは、呼び出し側が保証します。代わりに `serialize_checked` を使ってください。RT プールを持つ型、または `qps` パッキングが有効な型に生成される `serialize()` は別のメソッドです。こちらは引き続き安全に呼び出せ、`Result` を返します。

## 0.1 から 0.2 へ

0.1 から移行する場合は、先に 0.2.0 の移行ノートを読んでください。主な変更点は、ビットマップのフォーマットの変更（v2）、Cargo フィーチャー `native` と Windows のソケットおよび IPC リージョンの削除、`serialize` が `Vec<u8>` を返すようになったことです。
