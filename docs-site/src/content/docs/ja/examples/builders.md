---
title: "ビルダー"
description: "QSL — ビルダー"
---

build.rs でフィールドを宣言し、use_interpreter() で Rust バックエンドを選びます。config は DeqBuilder、schem は QslSchemaBuilder を受け取ります。整数の引数は最小値、最大値、既定値、optional、文字列は最小長、最大長、展開指定、optional です。必須フィールドを先に置きます。

```rust title="build.rs"
use qsl::{InterpreterBuilderExt, QslBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = QslBuilder::new()
        .config(|c| { c.stack_limit(64).max_data_size(65535); })
        .schem(|s| {
            s.name("Packet").safety().aggresive(false, false);
            s.field("id").u32(0, 1_000_000, 0, false);
            s.field("name").str(1, 32, false, false);
            s.field("status").u8(0, 15, 0, true);
        })
        .build()?;
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    std::fs::write(out.join("qsl_gen.rs"), schemas.use_interpreter()?)?;
    Ok(())
}
```

生成器は証明されたフレームの最大サイズから出力型を選びます。STACK_LIMIT 以下なら等号を含め `StackOut<N>`、より大きい場合や上限が不明な場合は alloc::`Vec<u8>` を使います。送信ヘッダーは含み、出力オブジェクト自体の長さや整列メタデータは含みません。

```rust title="main.rs"
include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));

fn main() -> qsl::QslResult<()> {
    let packet = Packet::try_new(42, qsl::StackString::try_from_str("QSL")?, Some(7))?;
    let frame: qsl::StackOut<42> = packet.serialize_checked()?;
    let restored = Packet::deserialize_checked(&frame)?;
    assert_eq!((restored.id, restored.status()), (42, Some(7)));
    Ok(())
}
```

[Builder API](/language/deq/) · [String](/language/strings/)
