---
title: DEQ 配置
description: DEQ 文件存放带类型的常量和作用于整个构建的设置，例如 MAX_DATA_SIZE、STACK_LIMIT 和 RT 池。
---

`.deq` 文件包含两类指令：可在 schema 边界中使用的带类型常量，以及代码生成的设置。DEQ 文件通过 `build_qsl_files` 的第二个参数传入：

```rust title="build.rs"
qsl::build_qsl_files(&["schemas/packet.qsl"], &["schemas/config.deq"], out)?;
```

## 常量

```deq title="config.deq"
!define MAX_NAME: u16 = 100;
```

类型可以是 `u8` 到 `u64`、`i8` 到 `i64`、`usize` 或 `isize`，值必须在该类型的表示范围内。表达式支持的运算符与 schema 相同，包括作用于整数类型的 `sizeof(T)`。后定义的常量可以用 `!NAME` 引用先定义的常量：

```deq title="config.deq"
!define HEADER: usize = sizeof(u64) + 2;
!define FRAME: usize = !HEADER * 4;
```

Schema 中直接用名字引用常量：

```qsl
&User[safety] { u64 id; str(1..MAX_NAME) name; }
```

## 设置

```deq title="config.deq"
!config MAX_DATA_SIZE = 65535;
!config USE_HASH = xxHash;
```
设置 | 默认值 | 含义
---|---|---
`MAX_DATA_SIZE` | `65535` | 帧、解压还原的数据以及解码值总大小的上限，单位为字节。取值范围为 1 到 `isize::MAX`。
`STACK_LIMIT` | `256` | 每个字符串或数组缓冲区的内联容量预算，单位为字节。上界更大时退回使用 `String` 或 `Vec`。取值范围为 `0` 到 `u32::MAX`。
`USE_HASH` | `xxHash` | Schema 哈希算法。`xxHash` 即 XXH3-64，`xxHash128` 即 XXH3-128。也接受 `XXH3_64` 和 `XXH3_128`。
`QSL_RT` | `false` | 设为 `true` 时，为最大帧大小有限的类型生成静态槽位池。见[缓冲区与复用](/guides/memory/#rt-slot-pool)。
`QSL_RT_DEFAULT` | `16` | 每个类型的槽位数，1 到 64。计的是槽位数，不是字节数。
设置作用于整个构建。同一个名字设置两次是错误，即使在不同文件中也一样。未知的指令和设置名同样是错误。

DEQ 文件中不包含文件路径，由 `build.rs` 决定编译哪些文件。
