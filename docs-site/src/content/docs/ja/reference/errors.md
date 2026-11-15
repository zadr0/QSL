---
title: エラー
description: QslError の各バリアントと、それが返される条件。
---

生成されたメソッドは `qsl::QslResult<T>` を返します。これは `Result<T, QslError>` のエイリアスです。`QslError` には 11 個のバリアントがあり、`std::error::Error` を実装しています。`Display` の出力は `QSL error: <Variant>` の形式で、たとえば `QSL error: InvalidUtf8` となります。
バリアント | 返される条件
---|---
`UnexpectedEof` | フレームが終わる前に入力が終わった。
`BufferTooSmall` | 渡したバッファが、出力に対して小さすぎる。
`LengthOverflow` | 長さの計算がオーバーフローする。
`InvalidWidth` | 幅またはクォンタムがサポートされていない。
`InvalidValue` | 値、長さ、メタデータのフィールドが、許可された範囲の外にある。
`InvalidUtf8` | 文字列のバイト列が有効な UTF-8 ではない。
`InvalidSchema` | スキーマが無効であるか、その操作がスキーマに適用できない。`QslCustom` のバリデータのデフォルトの戻り値でもある。
`InvalidPadding` | パディングのビットまたはバイトがゼロではない。
`PoolExhausted` | すべての RT スロットが使用中。待機しない `try_acquire` からのみ返される。
`SizeLimitExceeded` | フレームまたはデコード後のデータが `MAX_DATA_SIZE` を超えた。
`AllocationFailed` | 失敗しうるメモリ確保が失敗した。
ファイル操作やソケット操作は、`std::io::Error` を返すこともあります。

検証が成功したということは、バイト列がスキーマの形に合っているということです。誰が送ったのか、どのスキーマで書かれたのかはわかりません。
