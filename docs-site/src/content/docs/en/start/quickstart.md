---
title: "First project and build.rs"
description: "QSL documentation: First project and build.rs."
---

This example was compiled and executed. The relative dependency path fits an application two directories below dev-qsl; adjust it for another location. Build and application dependencies must target the same implementation.

### Cargo.toml

```toml
[package]
name="qsl-array-usage-example"
version="0.0.0"
edition="2021"
[dependencies]
qsl={path="../../proofs/qsl"}
[build-dependencies]
qsl={path="../../proofs/qsl"}
[workspace]
```

### build.rs

```rust
use qsl::{InterpreterBuilderExt, QslBuilder};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QslBuilder::new();
    builder.config(|c| {
        c.max_data_size(65535).stack_limit(256);
    });
    builder.schem(|s| {
        s.name("Packet").safety().compact().auto();
        s.field("prefix").u8(0, 7, 0, false);
        s.field("values").farray::<u16>(16, false);
        s.field("tail").u8(0, 255, 0, false);
    });
    let schemas = builder.build()?;
    let rust = schemas.use_interpreter()?;
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    std::fs::write(out.join("qsl.rs"), rust)?;
    Ok(())
}
```

farray::&lt;u16&gt;(16,false) declares exactly 16 elements. Type methods take optional as the final argument; numeric methods use (min,max,default,optional). Default initializes an object and never fills an absent optional during decoding.

Safety enables checked decoding, compact selects wire packing, and auto permits physical declaration reordering while retaining wire order. These settings are independent.

### src/main.rs

```rust
#![allow(dead_code, unused_mut)]
use std::mem::MaybeUninit;
include!(concat!(env!("OUT_DIR"), "/qsl.rs"));
fn main() -> qsl::QslResult<()> {
    let packet = Packet { prefix: 5, values: [42; 16], tail: 7 };
    let mut buffer = [0u8; 64];
    let used = packet.serialize_checked_to(&mut buffer)?;
    let mut output = MaybeUninit::<Packet>::uninit();
    let decoded = Packet::deserialize_checked_into(&buffer[..used], &mut output)?;
    assert_eq!(decoded.prefix, 5);
    assert_eq!(decoded.values, [42; 16]);
    assert_eq!(decoded.tail, 7);
    assert_eq!(used, 34);
    assert_eq!(std::mem::size_of::<Packet>(), 34);
    println!("Array example passed: wire={used} B, Self={} B", std::mem::size_of::<Packet>());
    Ok(())
}
```

Verified output: wire **34 B**, Self **34 B** with auto. The measured version of this wire model without auto has a 36 B Self. Smaller Self does not itself prove lower latency.

Textual field description:

```qsl
&Packet[safety,compact,auto] {
    u8(0..7) prefix;
    array<u16>(16) values;
    u8 tail;
}
```

For files, select QslSchemaBuilder::from_files(...).use_interpreter(), or use:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    qsl::build_qsl_files(&["packet.qsl"], &["settings.deq"], out)?;
    Ok(())
}
```

The wrapper writes qsl_gen.rs. The programmatic example chose qsl.rs itself. The include! filename must match the generated file.
