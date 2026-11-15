---
title: 属性
description: QSL のすべての型属性とフィールド修飾子について、生成されるものと、ワイヤフォーマットを変えるかどうか。
---

型属性は、型名の後の角括弧に `&Packet[safety,compact]` のように書きます。フィールド修飾子は、フィールド名の後でパイプに挟んで `str(1..100) name;` のように書きます。どれも省略でき、それぞれ 1 回だけ指定できます。

**ワイヤ**の列は、その属性がバイト列を変えるかどうか、つまり[スキーマハッシュ](/guides/schema-identity/)を変えるかどうかを示します。変える場合は、両側のピアで同じ設定を使う必要があります。

## 一覧
属性 | 対象 | ワイヤ | 効果
---|---|---|---
[`safety`](#safety) | 型 | いいえ | チェック付きデコードメソッドを生成します。
[`compact`](#compact) | 型 | はい | 範囲付きの値を最小のビット数で格納します。
[`alignment`](#alignment) | 型 | `compact` との併用時 | compact の各フィールドをバイト境界から開始します。
[`optionals=bitmap`](#optionalsbitmap) | 型 | はい | オプションフィールドを対象とする存在マスク。
[`predict_seq`](#predict_seq) | 型 | はい | ペイロード長から存在情報を導出します。
[`opt_table`](#opt_table) | 型 | `legacy` のみ | オプションフィールドのリーダーの静的テーブル。
[`qps`、`quant`](#qps-quant) | 型 | はい | 数値フィールドの可逆パッキング。
[`aggressive`](#aggressive) | 型 | いいえ | 上限付きの文字列と配列をインラインで格納します。
[`lifetime`](#lifetime) | 型 | いいえ | 文字列とバイト列を入力から借用します。
[`transmutate`](#transmutate) | 型 | いいえ | ネイティブレイアウトをワイヤレイアウトと一致させます。
[`inline`](#inline) | 型 | いいえ | 生成メソッドの `#[inline]` ポリシー。
[`slots=N`](#slotsn) | 型 | いいえ | この型の RT スロット数。
[`cache=N`、`loyal_cache`](#cachen-loyal_cache) | 型 | いいえ | 呼び出し側の `CachedFrame` に出力します。
[`state_bits=Qn`](#state_bitsqn) | 型 | はい | 旧来の存在インデックスのフォーマット。
[`decompress=true`](#decompresstrue) | フィールド | はい | 1 つの文字列に対する上限付き LZ 圧縮。
[`default=N`](#defaultn) | フィールド | いいえ | `impl Default` を生成します。
## 型属性

### safety {#safety}

`deserialize_checked` と `deserialize_checked_into` を生成します。どちらも信頼できない入力をデコードするためのメソッドです。値を返す前に、上下限、範囲、UTF-8、オプションフィールドのメタデータ、パディング、圧縮データの展開、`MAX_DATA_SIZE` による合計サイズの上限をチェックします。チェックなしのメソッドも引き続き使えます。`serialize_checked`、`serialize_checked_to`、`validate`、`validate_values` は、`safety` の有無にかかわらず生成されます。[入力の検証](/guides/safety/)を参照してください。

### compact {#compact}

範囲付きの各整数を `x - min` として `bit_length(max - min)` ビットで格納し、バイト境界をまたいで詰めます。`u8(0..7)`、`u8(0..31)`、`u16(0..1023)` は、それぞれ 3、5、10 ビットになります。文字列と配列の長さも、その上下限の中で同じ方法で格納されます。`opt_table`、`optionals=bitmap` と併用できます。`predict_seq`、`state_bits`、`qps` とは組み合わせられません。[フレームを小さくする](/guides/smaller-frames/)を参照してください。

### alignment {#alignment}

`compact` と併用すると、すべてのフィールドをバイト境界から開始します。フレームが大きくなることがあります。`compact` がない場合は、フォーマットを変えません。バッファのメモリアラインメントとは関係ありません。`qps` とは組み合わせられません。

### optionals=bitmap {#optionalsbitmap}

オプションフィールドごとに 1 ビットのマスクを書き込みます。マスクの幅は、オプションフィールドの数に応じて 0〜16 バイトです。最大 128 個のオプションフィールドに対応します。0.2.0 より前に使われていたマスクとは互換性がありません。`state_bits`、`opt_table=legacy`、`qps` とは組み合わせられません。[オプションフィールド](/language/optionals/#bitmap)を参照してください。

### predict_seq {#predict_seq}

どのオプションフィールドが存在するかを、ペイロード長から推定します。存在するフィールドのどの組み合わせでも長さが異なることが条件です。`optionals=bitmap` が指定されている場合は無視されます。`compact` とは組み合わせられません。[オプションフィールド](/language/optionals/#length-index-predict_seq)を参照してください。

### opt_table {#opt_table}

`opt_table`（または `opt_table=static`）は、オプションフィールドのリーダーを `match` ではなく静的テーブル経由でディスパッチします。バイト列もハッシュも変わりません。`predict_seq` と組み合わせると、0.2.0 より前に `[opt_table]` が生成していた長さグラフのフォーマットになります。

`opt_table=legacy` は別のフォーマットです。必須フィールドの後に、個数を表す 1 バイトと、オプションフィールドごとにリトルエンディアンの `u32` を 1 つ書き込みます。`u32` の値は、フィールドが存在しない場合は `0`、存在する場合はオプションペイロード内での値のオフセットに 1 を足したものです。通常の `opt_table`、`predict_seq`、`state_bits`、`optionals=bitmap`、`qps` とは組み合わせられません。

### qps、quant {#qps-quant}

`qps` は `qps=Q16` を意味します。`qps=Q2`、`Q4`、`Q8`、`Q16` でクォンタムを指定します。`quant=Qn` は別名です。値なしの `quant` はエラーで、両方の名前を使うこともエラーです。パックするのは数値フィールドだけです。文字列、配列、その長さはそれぞれのコーデックのままで、数値フィールドはそれらより前に置く必要があります。`compact`、`alignment`、`optionals=bitmap`、`opt_table=legacy` とは組み合わせられません。

RT プールがない場合、パッキングが有効な型には、`QslResult<QuantizedOut>` を返す安全な `serialize()` が生成されます。[フレームを小さくする](/guides/smaller-frames/)を参照してください。

### aggressive {#aggressive}

Rust 側のストレージを変えます。バイト列や、受け付ける入力は変わりません。

- 最大値が `STACK_LIMIT` に収まる場合、`str(min..max)` は `StackString<max>` に、`array<T>(min..max)` は `InlineArray<T, max>` になります。これより大きいフィールドは `String` または `Vec` のままです。
- オプションの数値フィールドは、範囲を表せる最小幅の `NonZero` 整数として格納されます。ただし、その幅がフィールドの型より広くならない場合に限ります。

### lifetime {#lifetime}

型にライフタイム `'a` を付け、入力から借用します。文字列は `&'a str` に、`array<u8>` は `&'a [u8]` になります。入力は値より長く生存する必要があります。ほかの要素型の配列と圧縮文字列は借用できず、ビルドに失敗します。借用するものがない `lifetime` 型もビルドに失敗します。

### transmutate {#transmutate}

型のネイティブレイアウトをワイヤレイアウトと一致させます。すべてのフィールドが固定サイズである必要があります。`compact`、`qps`、オプションフィールド、範囲付きの文字列や配列はビルドエラーです。ワイヤサイズを構造体のサイズと等しくする必要があるので、Rust がパディングを入れる位置には `padding(N)` フィールドを追加してください。

`lifetime` がない場合、`deserialize_transmutate_copy` は、`NATIVE_WIRE_COMPATIBLE` が true ならフレームを所有値に 1 ブロックでコピーし、そうでなければフィールドごとにデコードします。`[safety]` があれば安全なメソッドになり、結果を検証します。`[safety]` がなければ `unsafe` です。

`lifetime` がある場合、`deserialize_overlay` は入力を指す `&Self` を返します。ビルド時に、すべてのワイヤ上のオフセットがネイティブのオフセットと等しいことをチェックし、ビッグエンディアンのターゲットではビルドに失敗します。`[safety]` がある場合、`deserialize_overlay` は `NATIVE_WIRE_COMPATIBLE`、入力の長さとアラインメントをチェックし、その後に値をチェックします。`deserialize_checked` も同じ処理を行います。`[safety]` がない場合、`deserialize_overlay` は `unsafe` で、何もチェックしません。

どちらのメソッドにも、チェックを省略する `_unchecked` 版があります。

### inline {#inline}

`inline`（`inline=hint` と同じ）、`inline=always`、`inline=never` は、生成メソッドに対応する属性を付けます。指定しない場合、生成メソッドに inline 属性は付かず、コンパイラが判断します。`always` はコードサイズを増やすことがあり、高速化を保証するものではありません。

### slots=N {#slotsn}

この型の RT スロット数を、1〜64 の範囲で上書きします。`!config QSL_RT = true;` を指定した場合にだけ効果があります。[RT スロットプール](/guides/memory/#rt-slot-pool)を参照してください。

### cache=N、loyal_cache {#cachen-loyal_cache}

`CACHE_BYTES` と `serialize_cached` を生成します。`serialize_cached` は、呼び出し側が用意した `qsl::modernization::CachedFrame<N>` に書き込みます。`loyal_cache` は `CACHE_BYTES` を最大フレームサイズに設定します。型の最大フレームサイズが有限である必要があり、`N` がそれより小さいとビルドに失敗します。この属性は出力用のメモリを準備するもので、値をキャッシュするものではありません。[キャッシュした出力フレーム](/guides/memory/#cached-output-frames)を参照してください。

### state_bits=Qn {#state_bitsqn}

`Q1`（デフォルト）、`Q2`、`Q4`、`Q8`、`Q16` のいずれかです。旧来の存在インデックスのフォーマットを選択します。圧縮モードではありません。`optionals=bitmap`、`compact`、`opt_table=legacy` とは組み合わせられません。

## フィールド修飾子

### binary_opt {#binary_opt}

binary_opt はすべて廃止されました。&str の借用には [lifetime]、有界 StackString には [aggressive]、InlineString には decompress=true を使います。STACK_LIMIT は全文字列と配列に適用されます。

### decompress=true {#decompresstrue}

文字列を QSL の上限付き LZ コーデックでエンコードします。32 バイト未満の文字列と、圧縮しても短くならない文字列は、そのまま格納されます。デコードした各トークンは、復元済みのサイズ、書き込み先の容量、`MAX_DATA_SIZE` と照合されます。文字列には 65535 バイト以下の上限が必要です。範囲付きの文字列は `InlineString<max>` になり、`STACK_LIMIT` を超える場合は `String` になります。`lifetime` や `lifetime` とは組み合わせられません。[圧縮文字列](/language/strings/#compressed-strings)を参照してください。

### default=N {#defaultn}

型に `impl Default` を生成します。このフィールドの初期値は `N` で、オプションの場合は `Some(N)` です。ほかのフィールドは、それぞれの有効な最小値から始まります。`N` はフィールドの範囲内である必要があり、フィールドは数値型である必要があります。カスタム型のフィールドも持つ型では、ビルドに失敗します。バイト列もハッシュも変わりません。
