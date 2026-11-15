---
title: "Builders"
description: "QSL — Builders"
---

Declare fields in build.rs, then select the Rust backend with use_interpreter(). config receives DeqBuilder; schem receives QslSchemaBuilder. Integer arguments are minimum, maximum, default and optional. String arguments are minimum, maximum, decompression and optional. Required fields precede optional fields.

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

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.

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
