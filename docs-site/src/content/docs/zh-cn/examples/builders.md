---
title: "构建器"
description: "QSL — 构建器"
---

在 build.rs 中声明字段，再通过 use_interpreter() 选择 Rust 后端。config 接收 DeqBuilder，schem 接收 QslSchemaBuilder。整数参数为最小值、最大值、默认值、optional；字符串参数为最小长度、最大长度、解压选项、optional。必填字段在可选字段之前。

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

生成器根据已证明的帧长度上限选择输出类型。上限不超过 STACK_LIMIT（包括相等）时使用 `StackOut<N>`；上限更大或未知时使用 alloc::`Vec<u8>`。传输头计入此限制，输出对象自身的长度和对齐元数据不计入。

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
