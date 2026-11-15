---
title: "Первый проект и build.rs"
description: "Документация QSL: Первый проект и build.rs."
---

Пример собран и выполнен. Путь ../../proofs/qsl подходит приложению на два уровня ниже корня dev-qsl; при другом расположении его нужно изменить. Build- и runtime-зависимости должны указывать на одну реализацию.

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

`farray::&lt;u16&gt;(16, false)` задаёт ровно 16 элементов. Последний аргумент методов типов — optional. У числовых методов аргументы `(min,max,default,optional)`. Default относится к инициализации объекта; отсутствие optional при чтении остаётся отсутствием.

`safety` включает checked-десериализацию. `compact` выбирает wire-упаковку. `auto` разрешает перестановку физических объявлений, сохраняя wire-порядок. Эти настройки независимы.

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

Результат: wire = **34 B**, Self = **34 B** с auto. В парном бенчмарке такой wire-модели без auto Self равен 36 B. Уменьшение Self не означает автоматически меньшую задержку.

Описание полей текстом:

```qsl
&Packet[safety,compact,auto] {
    u8(0..7) prefix;
    array<u16>(16) values;
    u8 tail;
}
```

Обёртка для файлов:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    qsl::build_qsl_files(&["packet.qsl"], &["settings.deq"], out)?;
    Ok(())
}
```

Она записывает qsl_gen.rs. Программный пример самостоятельно выбрал qsl.rs. Имя в include! должно совпадать с записанным файлом.
