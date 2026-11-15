---
title: Rust 템플릿과 커스텀 타입
description: Rust 구조체를 필드 그룹으로 재사용하고, 검증되는 정수 래퍼를 등록하고, QslCustom을 구현하고, 중첩 리전을 선언합니다.
---

직접 만든 Rust 타입을 스키마에 가져오는 방법은 네 가지입니다. 각각 해결하는 문제가 다릅니다.
방식 | 쓰는 경우
---|---
`#[qsl::qsl_macro]` 템플릿 | 같은 필드 그룹이 여러 스키마에 반복됩니다.
`qsl::qsl_primitive!` | `DeviceId(u32)` 같은 정수 newtype이 있습니다.
`QslCustom` | 바이트를 완전히 제어해야 합니다.
`#region` | 고정된 네이티브 레이아웃을 가진 중첩 구조체가 필요합니다.
## Rust 템플릿

Rust 구조체를 템플릿으로 표시한 뒤, 그 이름을 필드 타입으로 씁니다.

```rust title="src/meta.rs"
#[qsl::qsl_macro]
pub struct Meta { pub id: u32, pub tick: u64 }
```

```qsl
&Packet { Meta meta; u16 code; }
```

생성된 `Packet`에는 평탄화된 필드 `meta_id`, `meta_tick`, `code`가 있습니다. `#[qsl::schem_macro]`는 이름만 다른 같은 마커입니다.

템플릿 필드에는 `u8`부터 `u64`, `i8`부터 `i64`, `String`, `&str`, `qsl::InlineString<N>`, 정수 고정 배열, 이들의 `Option<T>`, 다른 템플릿을 쓸 수 있습니다. 템플릿은 최대 64단계까지 중첩할 수 있습니다. 순환과 이름 충돌은 빌드 에러입니다. 템플릿 참조는 옵셔널이 될 수 없고 기본값도 가질 수 없습니다. 대신 템플릿의 개별 필드를 `Option<T>`로 만드십시오.

`build.rs`에서 `build_qsl_files_with_rust`로 Rust 파일 목록을 넘깁니다. 이 함수를 쓰려면 빌드 의존성의 `qsl`에 `rust-macros` 피처를 켜야 합니다.

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

## 검증되는 스칼라 래퍼 {#validated-scalar-wrappers}

`qsl::qsl_primitive!`는 `u8`/`i8`, `u16`/`i16`, `u32`/`i32`, `u64`/`i64`를 감싸고 값 범위를 가진 newtype을 선언합니다.

```rust title="primitive.rs"
qsl::qsl_primitive!(pub struct DeviceId(u32); range=(1,100););
```

매크로는 컴파일 타임에 래퍼의 크기와 정렬이 정수와 같은지, 정수가 오프셋 0에 있는지, 타입에 `Drop`이 없는지, 범위가 정수 타입에 들어가는지 검사합니다. 기본 코덱은 정수를 리틀 엔디언으로 씁니다. `compact` 코덱은 `value - min`을 범위를 담을 수 있는 가장 적은 바이트 수로 저장합니다. 값이 하나뿐인 범위는 0바이트를 차지합니다.

빌드 스크립트가 타입을 볼 수 있어야 하므로, 타입은 `build.rs`와 크레이트가 모두 포함하는 파일에 두십시오. 스키마 빌더에 타입을 등록한 뒤 코드를 생성합니다.

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

이 빌더 경로는 DEQ 파일을 읽지 않고 기본 설정을 씁니다. 등록한 래퍼는 배열 요소로도 쓸 수 있습니다. [중첩 리전](#nested-regions) 안에서 쓰려면 크레이트에서 `qsl::qsl_native_primitive!(DeviceId, u32);`도 호출하십시오.

## QslCustom {#qslcustom}

내장 코덱으로 해결되지 않을 때 `qsl::QslCustom`을 구현합니다.
항목 | 필수 | 의미
---|---|---
`SIZE` | 예 | 인코딩된 크기(바이트).
`COMP_SIZE` | 아니요 | `compact`일 때 인코딩된 크기. 기본값은 `SIZE`.
`qsl_write_unchecked`, `qsl_read_unchecked` | 예 | `unsafe` 원시 포인터 코덱.
`qsl_comp_write_unchecked`, `qsl_comp_read_unchecked` | 아니요 | `compact`용 코덱. 기본값은 일반 코덱.
`qsl_validate(&[u8], compact)` | 검증 디코딩에 필요 | 검증 디코더가 리더를 호출하기 전에 바이트를 검사합니다.
`qsl_validate_values(&self)` | 검증 인코딩에 필요 | `serialize_checked`가 라이터를 호출하기 전에 값을 검사합니다.
두 검증 함수는 기본적으로 `InvalidSchema`를 반환합니다. 따라서 직접 구현하기 전까지 검증 메서드는 커스텀 타입을 거부합니다. 정확히 `SIZE`바이트를 쓰고 유효한 Rust 값을 만드는 것은 사용자의 `unsafe` 코드가 책임집니다.

스키마가 커스텀 타입을 쓰면 생성된 파일은 `use ::qsl::QslCustom;`으로 시작합니다. 이 파일을 포함하는 모듈에서 `QslCustom`을 다시 import하지 마십시오. 커스텀 타입이 있으면 QSL이 그 코덱을 볼 수 없으므로 [스키마 해시](/guides/schema-identity/)도 불완전해집니다.

## 중첩 리전 {#nested-regions}

리전은 연속된 필드를 별도의 `#[repr(C)]` 구조체로 만듭니다.

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

이 스키마는 `Meta { id: u64, value: u32, tag: [u8; 4] }`와 `Parent { Meta: Meta, name: String }`을 생성합니다. 부모 구조체의 필드 이름은 리전 타입 이름을 따릅니다. 리전 필드는 부모로 평탄화되지 않습니다.

리전에는 고정 크기 필드만 넣을 수 있습니다. 정수, 고정 문자열, 고정 배열, 등록된 래퍼입니다. 옵셔널 필드는 거부됩니다. 리전은 최대 64단계까지 중첩할 수 있지만 서로 교차할 수는 없습니다. `compact`나 `qps`가 있는 타입에는 리전을 둘 수 없습니다.
