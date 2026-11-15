---
title: 生成されるメソッド
description: QSL がすべてのスキーマ型に生成するメソッドと定数を、呼び出し側に求める条件ごとにまとめます。
qslResources: [{"label": "docs.rs", "url": "https://docs.rs/qsl/0.3.0/qsl/"}]
---

各宣言から、以下のメソッドを持つ Rust の型が生成されます。どのメソッドが存在するかは、スキーマの属性によって決まります。ランタイムクレートの公開アイテムの一覧は、docs.rs を参照してください。

## チェック付き：信頼できないデータに対して安全

型が `[safety]` を持つ場合にだけ生成されます。
メソッド | シグネチャ（簡略化） | 備考
---|---|---
`deserialize_checked` | `fn(&[u8]) -> QslResult<Self>` | デコードしながら検証します。`lifetime` がある場合、入力は `&'a [u8]` です。
`deserialize_checked_into` | `fn(&[u8], &mut MaybeUninit<Self>) -> QslResult<&mut Self>` | 呼び出し側のメモリにデコードします。エラー時、書き込み先は未初期化のままです。
すべての型に生成されます。
メソッド | シグネチャ（簡略化） | 備考
---|---|---
`validate` | `fn(&[u8]) -> QslResult<()>` | 値を構築せずに、フレーム全体をチェックします。`deserialize_checked` の前に呼ぶ必要はありません。
`validate_values` | `fn(&self) -> QslResult<()>` | 範囲、長さ、固定長文字列の UTF-8、`MAX_DATA_SIZE` のサイズ上限をチェックします。
`serialize_checked` | `fn(&self) -> QslResult<`Vec<u8>`>` | 値をチェックしてからエンコードします。
`serialize_checked_to` | `fn(&self, &mut [u8]) -> QslResult<usize>` | `serialized_size()` バイト以上のスライスにエンコードし、書き込んだバイト数を返します。
オプションの数値フィールドを持つ型には、さらに次のメソッドが生成されます。
メソッド | シグネチャ（簡略化） | 備考
---|---|---
`try_new` | `fn(field, ...) -> QslResult<Self>` | すべてのフィールドを宣言順に受け取り（オプションのものは `Option<T>` として）、値をチェックします。
`name` | `fn(&self) -> Option<T>` | オプションフィールド `name` を読み取ります。
`set_name` | `fn(&mut self, Option<T>) -> QslResult<()>` | 値を変更します。範囲外の値には `InvalidValue` を返します。
[Rust でのオプションフィールド](/language/declarations/#optional-fields-in-rust)を参照してください。

## 信頼済み：有効性は呼び出し側が保証

これらは `qsl::Serializable` トレイトのメソッドです。`lifetime` を持つ型は代わりに `qsl::BorrowSerializable<'a>` を実装し、デコーダの名前は `deserialize_borrowed`、`deserialize_borrowed_into`、`deserialize_borrowed_unsafe` になります。どちらのトレイトでも、`serialized_size` と `get_size` を除くすべてのメソッドは `unsafe` です。
メソッド | 呼び出し側が保証すること
---|---
`serialize`、`serialize_to`、`serialize_into_unsafe`、`serialize_unsafe` | 値がスキーマに対して有効であること。書き込み先に、書き込み可能な `serialized_size()` バイトがあること。
`deserialize`、`deserialize_unsafe` | 入力が、完全で有効なフレーム 1 つであること。
`deserialize_into(ptr, len, out)` | 有効なフレームであること。`out` が書き込み可能で、アラインされていて、入力と重ならないこと。`out` にあった古い値はドロップされません。
`deserialize_reuse(input, &mut value)` | 有効なフレームであること。再利用するすべての文字列バッファに十分な容量があること。レイアウトが許す範囲でバッファを維持します。
`deserialize_prepared_into(ptr, len, &mut value)` | 上と同じ。生ポインタと長さを使います。
安全なトレイトメソッドは次のとおりです。
メソッド | 戻り値
---|---
`serialized_size(&self)` | この値のために用意すべき領域のサイズ。正確なフレーム長で、圧縮文字列を含む値では上限値。
`get_size()` | ありうる最小のフレーム長。
## 特殊なメソッド
メソッド | 生成される条件
---|---
`serialize()` | RT プールがある場合：安全。値をチェックし、空きスロットを待ってリースを返します。`qps` パッキングが有効で RT プールがない型の場合：安全。`QuantizedOut` を返します。[RT スロットプール](/guides/memory/#rt-slot-pool)を参照してください。
`serialize_quantized_to` | `qps` が有効な場合。値と容量をチェックしてから、パッキングのカーネルを実行します。
`quant_write_prepared`、`quant_read_padded` | `qps` が有効な場合。`qsl::payload_quant::QuantizedRecord` のメソッドです。`unsafe` で、範囲、パディング、互いに重ならずアラインされたメモリを呼び出し側が保証します。
`serialize_qps_to`、`deserialize_qps_into`、`deserialize_qps` | `ByteQps` のアダプタ。`[safety]` と `qps` を持ち、文字列も配列も持たない型だけに生成されます。
`serialize_cached` | `cache=N` または `loyal_cache`。`CachedFrame` に書き込みます。
`deserialize_transmutate_copy`、`deserialize_overlay` | [`transmutate`](/reference/attributes/#transmutate)。どちらにも、チェックを省略する `_unchecked` 版があります。
## 定数
定数 | 意味
---|---
`SCHEMA_HASH_STRICT: u128` | 名前を含むハッシュ。[スキーマの識別](/guides/schema-identity/)を参照してください。
`SCHEMA_HASH_LIGHT: u128` | 名前を含まないハッシュ。
`SCHEMA_HASHES` | 両方のハッシュと、アルゴリズム、バージョン、完全性。`<T as qsl::SchemaIdentity>::SCHEMA_HASHES` でも取得できます。
`MAX_DATA_SIZE` | チェック付きメソッドが使うサイズ上限。
`NATIVE_BYTES`、`NATIVE_ALIGNMENT` | 生成された構造体の `size_of` と `align_of`。
`WIRE_MAX_BYTES: Option<usize>` | 最大フレームサイズ。上限がない場合は `None`。
`CACHE_BYTES` | `serialize_cached` 用の容量。`cache=N` または `loyal_cache` の場合のみ。
`NATIVE_WIRE_COMPATIBLE` | ネイティブレイアウトがワイヤレイアウトと等しいかどうか。`transmutate` の場合のみ。
`QUANT_PADDING_BYTES` | 準備済みの `qps` 読み取りに必要な、追加で読み取り可能なバイト数。現在は `0`。
`REUSES_STORAGE` は `Serializable` トレイトの定数です。`deserialize_reuse` がバッファをその場で更新する場合、`<T as qsl::Serializable>::REUSES_STORAGE` は `true` です。

生成器は証明されたフレームの最大サイズから出力型を選びます。STACK_LIMIT 以下なら等号を含め `StackOut<N>`、より大きい場合や上限が不明な場合は alloc::`Vec<u8>` を使います。送信ヘッダーは含み、出力オブジェクト自体の長さや整列メタデータは含みません。
