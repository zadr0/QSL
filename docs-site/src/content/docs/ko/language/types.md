---
title: 타입
description: QSL 내장 타입, 각 타입이 생성하는 Rust 타입, 기본 와이어 인코딩.
---

## 내장 타입
스키마 | Rust 타입 | 비고
---|---|---
`u8` `u16` `u32` `u64` `i8` `i16` `i32` `i64` | 같은 기본 타입 |
정수 `T`에 대한 `T(A..B)` | 같은 기본 타입 | 값이 범위 안에 있어야 합니다. `compact`와 `qps`로 와이어 크기를 줄일 수 있습니다.
`flags(N)` | `N`에 따라 `u8`, `u16`, `u32`, `u64` 중 하나 | `N`은 1~64. `N`번째보다 높은 비트는 0이어야 합니다.
`str`, `string`, `String` | `String` | 가변 길이. `MAX_DATA_SIZE`로만 제한됩니다.
`str(A..B)` | `String` 또는 다른 문자열 타입 | UTF-8 바이트 단위, 양 끝을 포함하는 경계. [문자열](/language/strings/) 참고.
`str(N)` | `[u8; N]` | 정확히 `N`바이트의 UTF-8, 와이어에 길이 없음. `N`이 `STACK_LIMIT`보다 크면 `String`.
`array<T>`, `vec<T>` | `Vec<T>` | `T`는 내장 정수 타입이거나 [등록된 프리미티브](/guides/custom-types/#validated-scalar-wrappers)입니다.
`array<T>(N)` | `[T; N]` | 정확히 `N`개 요소, 와이어에 개수 없음. 배열이 `STACK_LIMIT`바이트보다 크면 `Vec<T>`.
`array<T>(A..B)` | `Vec<T>` | 양 끝을 포함하는 요소 개수. `aggressive`이고 `STACK_LIMIT`에 들어가면 `InlineArray<T, B>`.
`padding(N)` | `()` | 와이어에 값이 0인 바이트 `N`개.
`T?` | `Option<T>` | 옵셔널 필드. 숫자 옵셔널 필드는 접근자 메서드를 씁니다. [선언](/language/declarations/#optional-fields-in-rust) 참고.
그 밖의 이름 | 사용자의 Rust 타입 | `QslCustom`을 구현해야 합니다. [커스텀 타입](/guides/custom-types/) 참고.
`lifetime` 속성이 있으면 문자열과 `u8` 배열은 입력에서 `&'a str`, `&'a [u8]`로 빌려 옵니다.

## 기본 와이어 인코딩

포맷을 바꾸는 속성이 없을 때는 다음과 같습니다.

- **정수**는 네이티브 폭의 리틀 엔디언입니다. `u16(0..1023)`은 2바이트입니다.
- **문자열**은 `u32` 바이트 길이 뒤에 UTF-8 바이트가 옵니다. 고정 길이 `str(N)`에는 길이가 없습니다.
- **동적 배열**은 `u32` 요소 개수 뒤에 요소가 옵니다. 고정 배열에는 개수가 없습니다.
- **필수 필드**는 선언 순서를 따릅니다.
- **옵셔널 필드**는 값 앞에 존재 여부 메타데이터를 추가합니다. [옵셔널 필드](/language/optionals/) 참고.

프레임에는 타입 정보가 없습니다. 양쪽 모두 스키마를 알아야 합니다.

## 와이어 크기와 객체 크기

둘은 서로 독립적입니다.

```qsl
&Reading[compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

와이어에서는 3 + 5 + 10 = 18비트이고, 3바이트로 기록됩니다. 메모리에서 `Reading`은 `u8`, `u8`, `u16`을 담은 `#[repr(C)]` 구조체로 4바이트입니다. 이 크기는 `Reading::NATIVE_BYTES`에 들어 있습니다.
