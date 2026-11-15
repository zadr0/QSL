---
title: Rust 模板与自定义类型
description: 把 Rust 结构体复用为字段组，注册带校验的整数包装类型，实现 QslCustom，以及声明嵌套区域。
---

QSL 提供四种方式把你自己的 Rust 类型引入 schema，它们解决的问题各不相同。
机制 | 适用场景
---|---
`#[qsl::qsl_macro]` 模板 | 一组字段在多个 schema 中重复出现。
`qsl::qsl_primitive!` | 你有一个基于整数的 newtype，例如 `DeviceId(u32)`。
`QslCustom` | 需要完全控制字节。
`#region` | 需要一个具有固定原生布局的嵌套结构体。
## Rust 模板

把 Rust 结构体标记为模板，然后把它的名字用作字段类型：

```rust title="src/meta.rs"
#[qsl::qsl_macro]
pub struct Meta { pub id: u32, pub tick: u64 }
```

```qsl
&Packet { Meta meta; u16 code; }
```

生成的 `Packet` 包含展平后的字段 `meta_id`、`meta_tick` 和 `code`。`#[qsl::schem_macro]` 是同一个标记的另一个名字。

模板字段可以是 `u8` 到 `u64`、`i8` 到 `i64`、`String`、`&str`、`qsl::InlineString<N>`、整数的固定数组、以上类型的 `Option<T>`，以及其他模板。模板最多嵌套 64 层。循环引用和名称冲突都是构建错误。对模板的引用不能是可选的，也不能有默认值；需要时，把模板中的单个字段声明为 `Option<T>`。

在 `build.rs` 中用 `build_qsl_files_with_rust` 列出这些 Rust 文件。这需要在构建依赖中启用 `qsl` 的 `rust-macros` feature：

```toml title="Cargo.toml"
[build-dependencies]
qsl = { version = "=0.4.0", features = ["rust-macros"] }
```

```rust title="build.rs"
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var("OUT_DIR")?;
    qsl::build_qsl_files_with_rust(&["packet.qsl"], &[], &["src/meta.rs"], &out)?;
    Ok(())
}
```

## 带校验的标量包装类型 {#validated-scalar-wrappers}

`qsl::qsl_primitive!` 声明一个基于 `u8`/`i8`、`u16`/`i16`、`u32`/`i32` 或 `u64`/`i64`、带取值范围的 newtype：

```rust title="primitive.rs"
qsl::qsl_primitive!(pub struct DeviceId(u32); range=(1,100););
```

该宏在编译时检查：包装类型的大小和对齐与整数相同，整数位于偏移 0 处，类型没有实现 `Drop`，范围在整数的表示范围内。默认编解码器以小端序写入整数。`compact` 编解码器用能容纳该范围的最少整字节数存储 `value - min`。只含一个值的范围占 0 字节。

构建脚本必须能看到这个类型，所以要把它放在 `build.rs` 和你的 crate 都会引入的文件中。先用 schema 构建器注册它，再生成代码：

```rust title="build.rs"
mod primitive {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/primitive.rs"));
}
use qsl::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rust = QslSchemaBuilder::new()
        .with_primitive::<primitive::DeviceId>("DeviceId")?
        .with_source("&Packet[safety]{DeviceId id; DeviceId? extra;}")?
        .use_interpreter()?;
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("qsl_gen.rs");
    std::fs::write(out, rust)?;
    Ok(())
}
```

这种构建器方式不读取 DEQ 文件，使用默认设置。已注册的包装类型也可以作为数组元素。要在[嵌套区域](#nested-regions)中使用它，还需要在你的 crate 中调用 `qsl::qsl_native_primitive!(DeviceId, u32);`。

## QslCustom {#qslcustom}

内置编解码器都不合适时，实现 `qsl::QslCustom`：
项 | 是否必需 | 含义
---|---|---
`SIZE` | 是 | 编码后的字节数。
`COMP_SIZE` | 否 | 使用 `compact` 时编码后的字节数，默认等于 `SIZE`。
`qsl_write_unchecked`、`qsl_read_unchecked` | 是 | 基于裸指针的 `unsafe` 编解码函数。
`qsl_comp_write_unchecked`、`qsl_comp_read_unchecked` | 否 | `compact` 下的编解码函数，默认使用普通编解码函数。
`qsl_validate(&[u8], compact)` | 带检查的解码需要 | 在带检查的解码器调用你的读取函数之前检查字节。
`qsl_validate_values(&self)` | 带检查的编码需要 | 在 `serialize_checked` 调用你的写入函数之前检查值。
两个校验函数默认都返回 `InvalidSchema`，因此在你实现它们之前，带检查的方法会拒绝该自定义类型。你的 `unsafe` 代码要负责恰好写入 `SIZE` 字节，并产生有效的 Rust 值。

Schema 使用自定义类型时，生成的文件以 `use ::qsl::QslCustom;` 开头。不要在引入该文件的模块中再次导入 `QslCustom`。自定义类型还会让 [schema 哈希](/guides/schema-identity/)不完整，因为 QSL 看不到它的编解码实现。

## 嵌套区域 {#nested-regions}

区域把一段连续的字段变成独立的 `#[repr(C)]` 结构体：

```qsl
&Parent[safety] {
    #region(aggressive_transmutate="Meta");
    u64 id;
    u32 value;
    str(4) tag;
    #end_region;
    str(3..10) name;
}
```

这会生成 `Meta { id: u64, value: u32, tag: [u8; 4] }` 和 `Parent { Meta: Meta, name: String }`。父结构体中的这个字段以区域类型命名。区域中的字段不会展平到父结构体中。

区域只能包含固定大小的字段：整数、固定长度字符串、固定数组和已注册的包装类型。可选字段会被拒绝。区域最多嵌套 64 层，但不能相互交叉。带 `compact` 或 `qps` 的类型不能包含区域。
