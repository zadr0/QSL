---
title: 입력 검증
description: QSL의 [safety]가 생성하는 메서드, 검증 메서드가 확인하는 항목, 호출자의 책임으로 남는 부분.
---

소켓, 파일, 다른 프로세스에서 온 바이트는 신뢰할 수 없습니다. 이런 바이트는 검증 메서드로 디코딩하십시오. 검증 메서드는 `safety` 속성이 있는 타입에만 생성됩니다.

```qsl
&Packet[safety] {
    u32 id;
    str(1..100) text;
}
```

```rust
let bytes = packet.serialize_checked()?;
let decoded = Packet::deserialize_checked(&bytes)?;

// 또는 이미 가진 메모리에 디코딩:
let mut slot = std::mem::MaybeUninit::uninit();
let decoded: &mut Packet = Packet::deserialize_checked_into(&bytes, &mut slot)?;
```

## 검증 디코딩이 확인하는 것

생성된 디코더는 값을 반환하기 전에 다음을 검사합니다.

- 프레임이 가장 작은 유효 프레임 이상의 길이이고, 모든 필드를 담고 있으며, 스키마가 정한 위치에서 끝나는지
- 모든 숫자가 선언된 범위 안에 있는지
- 모든 문자열과 배열의 길이가 경계 안에 있는지
- 문자열이 유효한 UTF-8인지
- 옵셔널 필드 메타데이터가 스키마와 일치하는지
- 패딩 비트와 패딩 바이트가 0인지
- 압축 문자열을 복원한 크기가 경계를 넘지 않는지
- 디코딩된 값의 전체 크기가 `MAX_DATA_SIZE` 이내인지

검증과 디코딩은 한 패스에서 일어납니다. `validate`를 먼저 호출할 필요는 없습니다. 디코딩이 실패하면 `deserialize_checked_into`는 `slot`을 초기화되지 않은 상태로 둡니다.

## 크기 제한

`MAX_DATA_SIZE`(기본값 65535바이트)는 입력 프레임과 디코딩된 페이로드의 전체 크기를 제한합니다. 프레임에서 읽은 길이는 메모리를 확보하기 전에 각자의 경계와 이 한도에 비추어 검사됩니다. 한도는 [DEQ 파일](/language/deq/)에서 높이거나 낮춥니다.

저장소의 safety 예제는 다음 타입에 대해 잘못된 프레임 세 개를 `validate`에 넘깁니다.

```qsl
&Bounded[safety] { u8(1..3) kind; str(1..100) name; }
```
입력 | 결과
---|---
`09 01 00 00 00 61` | `InvalidValue`: `kind`가 9로, `1..3`을 벗어납니다.
`01 ff ff ff ff` | `UnexpectedEof`: 5바이트는 가장 작은 유효 프레임인 6바이트보다 짧습니다.
`01 01 00 00 00 ff` | `InvalidUtf8`: `ff`는 UTF-8이 아닙니다.
`01 ff ff ff ff 61`처럼 프레임 크기는 유효하지만 길이가 위조된 경우는 `InvalidValue`로 실패합니다. 바이트를 하나라도 복사하기 전에 길이를 경계 `100`과 비교하기 때문입니다.

## 검증 인코딩

`serialize_checked`는 값을 스키마(범위, 길이, 고정 크기 문자열의 UTF-8, `MAX_DATA_SIZE`)에 비추어 검사하고 새 `Vec<u8>`을 반환합니다. `serialize_checked_to`는 사용자가 준 슬라이스에 쓰고 쓴 바이트 수를 반환하며, 공간이 부족하면 `BufferTooSmall`을 반환합니다. 마지막 크기 검사가 실패하면 슬라이스의 일부가 이미 쓰였을 수 있습니다. 두 메서드는 `[safety]` 유무와 관계없이 모든 타입에 생성됩니다.

## 비검증 메서드

`[safety]`는 검증 메서드를 추가할 뿐, 비검증 메서드를 없애지 않습니다. `deserialize_into` 같은 `qsl::Serializable` 트레이트의 메서드와 원시 포인터 리더는 다음을 전제합니다.

- 프레임이 완전하고 이 스키마에 유효합니다.
- 대상 메모리가 충분히 크고, 올바르게 정렬되어 있으며, 입력과 겹치지 않습니다.
- 빌린 입력이 결과보다 오래 삽니다.

이 메서드는 직접 만든 프레임이나 이미 검증한 프레임에만 쓰십시오. 모두 `unsafe`이며, 0.3.0부터는 `Serializable::serialize`와 `BorrowSerializable::serialize`도 마찬가지입니다. RT 풀이 있거나 `qps` 패킹이 활성화된 타입에서 생성되는 `serialize()`는 별개의 안전한 메서드로, `Result`를 반환합니다.

## 검증의 한계

- 검증은 바이트가 스키마에 맞는 형태라는 것만 증명합니다. 어떤 스키마가 그 바이트를 만들었는지는 증명하지 않습니다. 레이아웃이 같은 두 스키마는 같은 바이트를 받아들입니다. 핸드셰이크에서 [스키마 해시](/guides/schema-identity/)나 직접 정한 구분자를 쓰십시오.
- 송신자를 인증하지 않습니다.
- [커스텀 타입](/guides/custom-types/)의 안전성은 그 타입의 `qsl_validate` 구현에 달려 있습니다. 기본 구현은 모든 입력을 거부합니다.

생성기는 증명된 최대 프레임 크기로 출력 형식을 선택합니다. STACK_LIMIT 이하이면 경계를 포함하여 `StackOut<N>`을, 더 크거나 상한을 모르면 alloc::`Vec<u8>`을 사용합니다. 전송 헤더는 포함되지만 출력 객체 자체의 길이와 정렬 메타데이터는 제외됩니다.
