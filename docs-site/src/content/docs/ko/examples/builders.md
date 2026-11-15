---
title: "빌더"
description: "QSL — 빌더"
---

build.rs에서 필드를 선언하고 use_interpreter()로 Rust 백엔드를 선택합니다. config는 DeqBuilder, schem은 QslSchemaBuilder를 받습니다. 정수 인수는 최소값, 최대값, 기본값, optional이며 문자열은 최소 길이, 최대 길이, 압축 해제, optional입니다. 필수 필드가 먼저 옵니다.

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

생성기는 증명된 최대 프레임 크기로 출력 형식을 선택합니다. STACK_LIMIT 이하이면 경계를 포함하여 `StackOut<N>`을, 더 크거나 상한을 모르면 alloc::`Vec<u8>`을 사용합니다. 전송 헤더는 포함되지만 출력 객체 자체의 길이와 정렬 메타데이터는 제외됩니다.

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
