---
title: 속성
description: QSL의 모든 타입 속성과 필드 수정자, 각각이 생성하는 코드, 와이어 포맷을 바꾸는지 여부.
---

타입 속성은 이름 뒤 대괄호 안에 씁니다: `&Packet[safety,compact]`. 필드 수정자는 필드 이름 뒤 파이프 사이에 씁니다: `str(1..100) name;`. 모두 생략할 수 있으며, 각각 한 번만 쓸 수 있습니다.

**와이어** 열은 속성이 바이트를, 따라서 [스키마 해시](/guides/schema-identity/)를 바꾸는지 나타냅니다. 바꾼다면 양쪽 피어가 같은 설정을 써야 합니다.

## 요약
속성 | 위치 | 와이어 | 효과
---|---|---|---
[`safety`](#safety) | 타입 | 아니요 | 검증 디코드 메서드를 생성합니다.
[`compact`](#compact) | 타입 | 예 | 범위가 있는 값을 최소 비트 수로 저장합니다.
[`alignment`](#alignment) | 타입 | `compact`와 함께일 때 | compact 필드를 각각 바이트 경계에서 시작합니다.
[`optionals=bitmap`](#optionalsbitmap) | 타입 | 예 | 옵셔널 필드에 대한 존재 마스크.
[`predict_seq`](#predict_seq) | 타입 | 예 | 페이로드 길이에서 존재 여부를 유도합니다.
[`opt_table`](#opt_table) | 타입 | `legacy`만 | 옵셔널 필드 리더의 정적 테이블.
[`qps`, `quant`](#qps-quant) | 타입 | 예 | 숫자 필드의 가역 패킹.
[`aggressive`](#aggressive) | 타입 | 아니요 | 경계가 있는 문자열과 배열을 인라인으로 저장합니다.
[`lifetime`](#lifetime) | 타입 | 아니요 | 입력에서 문자열과 바이트를 빌립니다.
[`transmutate`](#transmutate) | 타입 | 아니요 | 네이티브 레이아웃을 와이어 레이아웃과 같게 만듭니다.
[`inline`](#inline) | 타입 | 아니요 | 생성된 메서드의 `#[inline]` 정책.
[`slots=N`](#slotsn) | 타입 | 아니요 | 이 타입의 RT 슬롯 수.
[`cache=N`, `loyal_cache`](#cachen-loyal_cache) | 타입 | 아니요 | 호출자의 `CachedFrame`에 출력합니다.
[`state_bits=Qn`](#state_bitsqn) | 타입 | 예 | 레거시 존재 인덱스 포맷.
[`decompress=true`](#decompresstrue) | 필드 | 예 | 문자열 하나에 대한 크기 제한 LZ 압축.
[`default=N`](#defaultn) | 필드 | 아니요 | `impl Default`를 생성합니다.
## 타입 속성

### safety {#safety}

신뢰할 수 없는 입력을 디코딩하는 `deserialize_checked`와 `deserialize_checked_into`를 생성합니다. 이 메서드는 값을 반환하기 전에 경계, 범위, UTF-8, 옵셔널 필드 메타데이터, 패딩, 압축 해제, 전체 `MAX_DATA_SIZE` 한도를 검사합니다. 비검증 메서드도 그대로 남습니다. `serialize_checked`, `serialize_checked_to`, `validate`, `validate_values`는 `safety` 유무와 관계없이 생성됩니다. [입력 검증](/guides/safety/) 참고.

### compact {#compact}

범위가 있는 각 정수를 `x - min`으로 바꿔 `bit_length(max - min)`비트에 저장하며, 바이트 경계를 넘어 이어 붙입니다. `u8(0..7)`, `u8(0..31)`, `u16(0..1023)`은 각각 3, 5, 10비트를 차지합니다. 문자열과 배열의 길이도 경계 안에서 같은 방식으로 저장됩니다. `opt_table`, `optionals=bitmap`과 함께 쓸 수 있습니다. `predict_seq`, `state_bits`, `qps`와는 함께 쓸 수 없습니다. [프레임 줄이기](/guides/smaller-frames/) 참고.

### alignment {#alignment}

`compact`와 함께 쓰면 모든 필드를 바이트 경계에서 시작합니다. 프레임이 커질 수 있습니다. `compact` 없이 쓰면 포맷을 바꾸지 않습니다. 버퍼의 메모리 정렬과는 관계가 없습니다. `qps`와 함께 쓸 수 없습니다.

### optionals=bitmap {#optionalsbitmap}

옵셔널 필드마다 1비트인 마스크를 씁니다. 마스크 폭은 옵셔널 필드 수에 따라 0~16바이트입니다. 옵셔널 필드를 최대 128개까지 지원합니다. 0.2.0 이전에 쓰던 마스크와 호환되지 않습니다. `state_bits`, `opt_table=legacy`, `qps`와 함께 쓸 수 없습니다. [옵셔널 필드](/language/optionals/#bitmap) 참고.

### predict_seq {#predict_seq}

페이로드 길이로부터 어떤 옵셔널 필드가 있는지 추론합니다. 존재하는 필드의 모든 조합이 서로 다른 길이를 내야 합니다. `optionals=bitmap`이 설정되어 있으면 무시됩니다. `compact`와 함께 쓸 수 없습니다. [옵셔널 필드](/language/optionals/#length-index-predict_seq) 참고.

### opt_table {#opt_table}

`opt_table`(또는 `opt_table=static`)은 옵셔널 필드 리더를 `match` 대신 정적 테이블로 디스패치합니다. 바이트도 해시도 같습니다. `predict_seq`와 함께 쓰면 0.2.0 이전에 `[opt_table]`이 만들던 길이 그래프 포맷을 선택합니다.

`opt_table=legacy`는 다른 포맷입니다. 필수 필드 뒤에 개수 바이트 하나와, 옵셔널 필드마다 리틀 엔디언 `u32` 하나를 씁니다. 필드가 없으면 `0`, 있으면 옵셔널 페이로드 안에서 값의 오프셋에 1을 더한 값입니다. 일반 `opt_table`, `predict_seq`, `state_bits`, `optionals=bitmap`, `qps`와 함께 쓸 수 없습니다.

### qps, quant {#qps-quant}

`qps`는 `qps=Q16`을 뜻합니다. `qps=Q2`, `Q4`, `Q8`, `Q16`으로 퀀텀을 정합니다. `quant=Qn`은 별칭입니다. 값 없이 `quant`만 쓰면 에러이고, 두 이름을 함께 써도 에러입니다. 숫자 필드만 패킹합니다. 문자열, 배열, 그 길이는 각자의 코덱을 유지하며, 숫자 필드가 그보다 앞에 와야 합니다. `compact`, `alignment`, `optionals=bitmap`, `opt_table=legacy`와 함께 쓸 수 없습니다.

RT 풀이 없으면, 패킹이 활성화된 타입에는 `QslResult<QuantizedOut>`을 반환하는 안전한 `serialize()`가 생깁니다. [프레임 줄이기](/guides/smaller-frames/) 참고.

### aggressive {#aggressive}

바이트나 허용되는 입력이 아니라 Rust 저장 방식을 바꿉니다.

- 최댓값이 `STACK_LIMIT`에 들어가면 `str(min..max)`는 `StackString<max>`가 되고 `array<T>(min..max)`는 `InlineArray<T, max>`가 됩니다. 더 큰 필드는 `String`이나 `Vec`으로 남습니다.
- 옵셔널 숫자 필드는 범위를 담을 수 있는 가장 작은 폭의 `NonZero` 정수로 저장됩니다. 단, 그 폭이 필드 타입보다 넓지 않을 때만입니다.

### lifetime {#lifetime}

타입에 라이프타임 `'a`를 붙이고 입력에서 빌려 옵니다. 문자열은 `&'a str`이 되고 `array<u8>`은 `&'a [u8]`이 됩니다. 입력은 값보다 오래 살아야 합니다. 다른 요소 타입의 배열과 압축 문자열은 빌릴 수 없어 빌드에 실패하며, 빌릴 것이 하나도 없는 `lifetime` 타입도 마찬가지입니다.

### transmutate {#transmutate}

타입의 네이티브 레이아웃을 와이어 레이아웃과 같게 만듭니다. 모든 필드는 고정 크기여야 합니다. `compact`, `qps`, 옵셔널 필드, 범위가 있는 문자열이나 배열은 빌드 에러입니다. 와이어 크기가 구조체 크기와 같아야 하므로, Rust가 패딩을 넣는 자리에는 `padding(N)` 필드를 추가하십시오.

`lifetime`이 없으면 `deserialize_transmutate_copy`는 `NATIVE_WIRE_COMPATIBLE`이 true일 때 프레임을 한 블록으로 복사해 소유한 값을 만들고, 그렇지 않으면 필드 단위로 디코딩합니다. `[safety]`가 있으면 이 메서드는 안전하며 결과를 검증합니다. `[safety]`가 없으면 `unsafe`입니다.

`lifetime`이 있으면 `deserialize_overlay`는 입력 안을 가리키는 `&Self`를 반환합니다. 빌드 시에 모든 와이어 오프셋이 네이티브 오프셋과 같은지 검사하며, 빅 엔디언 타깃에서는 빌드에 실패합니다. `[safety]`가 있으면 이 메서드는 `NATIVE_WIRE_COMPATIBLE`, 입력 길이와 정렬을 검사한 다음 값을 검사하며, `deserialize_checked`도 같은 일을 합니다. `[safety]`가 없으면 `deserialize_overlay`는 `unsafe`이고 아무것도 검사하지 않습니다.

두 메서드 모두 검사를 건너뛰는 `_unchecked` 변형이 있습니다.

### inline {#inline}

`inline`(`inline=hint`와 같음), `inline=always`, `inline=never`는 생성된 메서드에 해당 속성을 붙입니다. 지정하지 않으면 생성된 메서드에는 inline 속성이 없고 컴파일러가 결정합니다. `always`는 코드 크기를 키울 수 있으며, 속도 향상을 보장하지 않습니다.

### slots=N {#slotsn}

이 타입의 RT 슬롯 수를 1~64로 재정의합니다. `!config QSL_RT = true;`일 때만 효과가 있습니다. [RT 슬롯 풀](/guides/memory/#rt-slot-pool) 참고.

### cache=N, loyal_cache {#cachen-loyal_cache}

`CACHE_BYTES`와, 호출자가 제공한 `qsl::modernization::CachedFrame<N>`에 쓰는 `serialize_cached`를 생성합니다. `loyal_cache`는 `CACHE_BYTES`를 최대 프레임 크기로 정합니다. 타입의 최대 프레임 크기가 유한해야 하며, `N`이 그보다 작으면 빌드에 실패합니다. 이 속성은 출력 메모리를 준비할 뿐 값을 캐시하지 않습니다. [캐시된 출력 프레임](/guides/memory/#cached-output-frames) 참고.

### state_bits=Qn {#state_bitsqn}

`Q1`(기본값), `Q2`, `Q4`, `Q8`, `Q16` 중 하나입니다. 레거시 존재 인덱스 포맷을 선택합니다. 압축 모드가 아닙니다. `optionals=bitmap`, `compact`, `opt_table=legacy`와 함께 쓸 수 없습니다.

## 필드 수정자

### binary_opt {#binary_opt}

binary_opt는 모두 제거되었습니다. &str 빌림은 [lifetime], 제한된 StackString은 [aggressive], InlineString은 decompress=true를 사용합니다. STACK_LIMIT는 모든 문자열과 배열에 적용됩니다.

### decompress=true {#decompresstrue}

QSL의 크기 제한 LZ 코덱으로 문자열을 인코딩합니다. 32바이트보다 짧은 문자열과 압축해도 짧아지지 않는 문자열은 원본 그대로 저장됩니다. 디코딩한 토큰마다 복원된 크기, 대상 용량, `MAX_DATA_SIZE`를 검사합니다. 문자열의 상한이 65535바이트 이하여야 합니다. 범위가 있는 문자열은 `InlineString<max>`가 되고, `STACK_LIMIT`을 넘으면 `String`이 됩니다. `lifetime`이나 `lifetime`와 함께 쓸 수 없습니다. [압축 문자열](/language/strings/#compressed-strings) 참고.

### default=N {#defaultn}

타입에 `impl Default`를 생성합니다. 이 필드는 `N`으로 시작하며, 옵셔널이면 `Some(N)`으로 시작합니다. 나머지 필드는 가장 작은 유효값으로 시작합니다. `N`은 필드 범위 안에 있어야 하고, 필드는 숫자여야 합니다. 커스텀 타입 필드도 가진 타입은 빌드에 실패합니다. 바이트와 해시는 바뀌지 않습니다.
