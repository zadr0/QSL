---
title: "Билдеры"
description: "QSL — Билдеры"
---

Поля задаются в build.rs, затем use_interpreter() выбирает генерацию Rust. В config передаётся DeqBuilder, в schem — QslSchemaBuilder. Аргументы чисел: минимум, максимум, default и optional. Аргументы строки: минимум, максимум, decompress и optional. Обязательные поля располагаются перед опциональными.

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

Генератор выбирает тип выхода по доказанному максимуму кадра. Если он не превышает STACK_LIMIT, включая равенство, обычная сериализация возвращает `StackOut<N>`; больший или неизвестный максимум выбирает alloc::`Vec<u8>`. Передаваемые заголовки входят в порог; служебная длина и выравнивание самого выходного объекта — нет.

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
