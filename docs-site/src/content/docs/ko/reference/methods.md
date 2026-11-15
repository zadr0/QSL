---
title: 생성되는 메서드
description: QSL이 모든 스키마 타입에 생성하는 메서드와 상수를, 호출자에게 요구하는 조건별로 정리했습니다.
qslResources: [{"label": "docs.rs", "url": "https://docs.rs/qsl/0.3.0/qsl/"}]
---

각 선언은 아래 메서드를 가진 Rust 타입을 만듭니다. 어떤 메서드가 생기는지는 스키마의 속성에 따라 다릅니다. 런타임 크레이트의 공개 아이템 전체 목록은 docs.rs를 참고하십시오.

## 검증 메서드: 신뢰할 수 없는 데이터에도 안전

타입에 `[safety]`가 있을 때만 생성됩니다.
메서드 | 시그니처(단순화) | 비고
---|---|---
`deserialize_checked` | `fn(&[u8]) -> QslResult<Self>` | 디코딩하면서 검증합니다. `lifetime`이 있으면 입력은 `&'a [u8]`입니다.
`deserialize_checked_into` | `fn(&[u8], &mut MaybeUninit<Self>) -> QslResult<&mut Self>` | 사용자의 메모리에 디코딩합니다. 에러가 나면 대상은 초기화되지 않은 상태로 남습니다.
모든 타입에 생성됩니다.
메서드 | 시그니처(단순화) | 비고
---|---|---
`validate` | `fn(&[u8]) -> QslResult<()>` | 값을 만들지 않고 프레임 전체를 검사합니다. `deserialize_checked` 전에 호출할 필요는 없습니다.
`validate_values` | `fn(&self) -> QslResult<()>` | 범위, 길이, 고정 문자열의 UTF-8, `MAX_DATA_SIZE` 한도를 검사합니다.
`serialize_checked` | `fn(&self) -> QslResult<`Vec<u8>`>` | 값을 검사한 뒤 인코딩합니다.
`serialize_checked_to` | `fn(&self, &mut [u8]) -> QslResult<usize>` | 크기가 `serialized_size()`바이트 이상인 슬라이스에 인코딩하고, 쓴 바이트 수를 반환합니다.
옵셔널 숫자 필드가 있는 타입에는 다음도 생깁니다.
메서드 | 시그니처(단순화) | 비고
---|---|---
`try_new` | `fn(field, ...) -> QslResult<Self>` | 모든 필드를 선언 순서대로 받고(옵셔널 필드는 `Option<T>`로) 값을 검사합니다.
`name` | `fn(&self) -> Option<T>` | 옵셔널 필드 `name`을 읽습니다.
`set_name` | `fn(&mut self, Option<T>) -> QslResult<()>` | 값을 바꿉니다. 범위를 벗어나면 `InvalidValue`.
[Rust에서의 옵셔널 필드](/language/declarations/#optional-fields-in-rust) 참고.

## 신뢰 입력용 메서드: 호출자가 유효성을 보장

`qsl::Serializable` 트레이트의 메서드입니다. `lifetime`이 있는 타입은 대신 `qsl::BorrowSerializable<'a>`를 구현하며, 이 트레이트에서 디코더 이름은 `deserialize_borrowed`, `deserialize_borrowed_into`, `deserialize_borrowed_unsafe`입니다. 두 트레이트 모두에서 `serialized_size`와 `get_size`를 제외한 모든 메서드는 `unsafe`입니다.
메서드 | 호출자가 보장할 것
---|---
`serialize`, `serialize_to`, `serialize_into_unsafe`, `serialize_unsafe` | 값이 스키마에 유효하고, 대상에 `serialized_size()`바이트를 쓸 수 있습니다.
`deserialize`, `deserialize_unsafe` | 입력이 완전하고 유효한 프레임 하나입니다.
`deserialize_into(ptr, len, out)` | 유효한 프레임. `out`은 쓰기 가능하고 정렬되어 있으며 입력과 겹치지 않습니다. `out`의 이전 값은 drop되지 않습니다.
`deserialize_reuse(input, &mut value)` | 유효한 프레임. 재사용하는 모든 문자열 버퍼의 용량이 충분합니다. 레이아웃이 허용하면 버퍼를 유지합니다.
`deserialize_prepared_into(ptr, len, &mut value)` | 같은 조건을 원시 포인터와 길이로.
안전한 트레이트 메서드는 다음과 같습니다.
메서드 | 반환값
---|---
`serialized_size(&self)` | 이 값을 위해 준비할 공간. 정확한 프레임 길이이며, 압축 문자열이 있는 값에서는 상한입니다.
`get_size()` | 가능한 가장 작은 프레임 길이.
## 특수 메서드
메서드 | 생성 조건
---|---
`serialize()` | RT 풀이 있으면 안전하며, 값을 검사하고 빈 슬롯을 기다린 뒤 리스를 반환합니다. RT 풀 없이 `qps` 패킹이 활성화된 타입에서는 안전하며 `QuantizedOut`을 반환합니다. [RT 슬롯 풀](/guides/memory/#rt-slot-pool) 참고.
`serialize_quantized_to` | `qps` 활성화. 값과 용량을 검사한 뒤 패킹 커널을 실행합니다.
`quant_write_prepared`, `quant_read_padded` | `qps` 활성화. `qsl::payload_quant::QuantizedRecord`의 메서드. `unsafe`: 호출자가 범위, 패딩, 서로 겹치지 않고 정렬된 메모리를 보장합니다.
`serialize_qps_to`, `deserialize_qps_into`, `deserialize_qps` | `ByteQps` 어댑터. `[safety]`와 `qps`가 있고 문자열이나 배열이 없는 타입에만 생성됩니다.
`serialize_cached` | `cache=N` 또는 `loyal_cache`. `CachedFrame`에 씁니다.
`deserialize_transmutate_copy`, `deserialize_overlay` | [`transmutate`](/reference/attributes/#transmutate). 각각 검사를 건너뛰는 `_unchecked` 변형이 있습니다.
## 상수
상수 | 의미
---|---
`SCHEMA_HASH_STRICT: u128` | 이름을 포함한 해시. [스키마 식별](/guides/schema-identity/) 참고.
`SCHEMA_HASH_LIGHT: u128` | 이름을 제외한 해시.
`SCHEMA_HASHES` | 두 해시와 알고리즘, 버전, 완전성 정보. `<T as qsl::SchemaIdentity>::SCHEMA_HASHES`로도 접근할 수 있습니다.
`MAX_DATA_SIZE` | 검증 메서드가 쓰는 한도.
`NATIVE_BYTES`, `NATIVE_ALIGNMENT` | 생성된 구조체의 `size_of`와 `align_of`.
`WIRE_MAX_BYTES: Option<usize>` | 최대 프레임 크기. 상한이 없으면 `None`.
`CACHE_BYTES` | `serialize_cached`의 용량. `cache=N`이나 `loyal_cache`가 있을 때만.
`NATIVE_WIRE_COMPATIBLE` | 네이티브 레이아웃이 와이어 레이아웃과 같은지 여부. `transmutate`가 있을 때만.
`QUANT_PADDING_BYTES` | 준비된(prepared) `qps` 읽기에 필요한 추가 읽기 가능 바이트 수. 현재 `0`.
`REUSES_STORAGE`는 `Serializable` 트레이트의 상수입니다. `deserialize_reuse`가 버퍼를 제자리에서 갱신하면 `<T as qsl::Serializable>::REUSES_STORAGE`가 `true`입니다.

생성기는 증명된 최대 프레임 크기로 출력 형식을 선택합니다. STACK_LIMIT 이하이면 경계를 포함하여 `StackOut<N>`을, 더 크거나 상한을 모르면 alloc::`Vec<u8>`을 사용합니다. 전송 헤더는 포함되지만 출력 객체 자체의 길이와 정렬 메타데이터는 제외됩니다.
