---
title: 0.3.0으로 업그레이드
description: qsl 0.3.0의 변경 사항 - qsl-core 분리와 줄어든 문자열 표현.
qslResources: [{"label": "0.2.0 노트", "url": "https://github.com/zadr0/QSL/blob/main/docs/migration-0.2.0.en.md"}]
---

QSL 0.3.0은 공용 런타임을 새 크레이트 `qsl-core` 0.1.0으로 옮기고, 오래된 문자열 API를 제거합니다. 비트맵(v2)과 일반 문자열의 와이어 포맷은 바뀌지 않습니다.

## 버전을 함께 올리기

```toml title="Cargo.toml"
[dependencies]
qsl = "=0.4.0"

[build-dependencies]
qsl = "=0.4.0"
```

내부 크레이트에 직접 의존한다면 `qsl-lexer`와 `qsl-interpreter`도 0.3.0으로 올리십시오. 속성 매크로는 그대로 `qsl-macros-impl` 0.1.0에 있습니다.

## 크레이트 분리
크레이트 | 내용 | 의존 대상
---|---|---
`qsl-core` | 에러, 스키마 모델, 공용 분석, I/O, UTF-8 컨테이너, 검증, 버퍼, 저수준 코덱 | 다른 QSL 크레이트 없음
`qsl-lexer` | 렉서, QSL·DEQ 파서, 파일 로딩, 템플릿 전개, 스키마 빌더 | `qsl-core`
`qsl-interpreter` | Rust 코드 생성 | `qsl-lexer`
`qsl` | 퍼사드, 매크로, 호환 진입점 | 위의 모든 크레이트와 `qsl-macros-impl`
기존 모듈 경로는 다시 내보내기(re-export)로 계속 동작합니다. `QslError`와 `QslCustom`은 어느 경로로 import하든 각각 하나의 타입입니다.

## 문자열

남은 표현은 네 가지입니다.
선택 | Rust 타입
---|---
일반 문자열 | `String`
`aggressive`, 상한이 `STACK_LIMIT` 이내 | `StackString<N>`
`decompress=true`, 경계가 `STACK_LIMIT` 이내 | `InlineString<N>`
`lifetime` | `&str`
제거됨: `binary_opt=guard`, `binary_opt=true`, `Utf8Guard`, `Utf8CopyGuard`, 바이트 문자열 모드. 이를 쓰는 스키마는 빌드에 실패합니다. 각 필드를 위 네 가지 표현 중 하나로 바꾸십시오.

새 값 생성(*new*)과 재사용(*reuse*)은 타입이 아니라 메서드 선택입니다. 기존 `String`에 디코딩하기 전에 충분한 용량을 확보하십시오. [문자열](/language/strings/)과 [버퍼와 재사용](/guides/memory/#reading-into-existing-memory)을 참고하십시오.

## 트레이트 메서드가 unsafe로 바뀜

`Serializable::serialize`와 `BorrowSerializable::serialize`는 이제 `unsafe`입니다. 값이 스키마에 유효하다는 것을 호출자가 보장해야 합니다. 대신 `serialize_checked`를 쓰십시오. RT 풀이 있거나 `qps` 패킹이 활성화된 타입에서 생성되는 `serialize()`는 별개의 메서드입니다. 이 메서드는 여전히 안전하며 `Result`를 반환합니다.

## 0.1에서 0.2로

0.1에서 업그레이드한다면 먼저 0.2.0 노트를 읽으십시오. 주요 변경점은 다음과 같습니다. 비트맵 포맷이 바뀌었고(v2), `native` Cargo 피처와 Windows 소켓·IPC 리전이 제거되었으며, `serialize`가 이제 `Vec<u8>`을 반환합니다.
