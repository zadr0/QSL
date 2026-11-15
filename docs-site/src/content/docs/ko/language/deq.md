---
title: DEQ 설정
description: DEQ 파일에는 타입이 있는 상수와 MAX_DATA_SIZE, STACK_LIMIT, RT 풀 같은 빌드 전체 설정이 들어갑니다.
---

`.deq` 파일에는 두 종류의 지시문이 들어갑니다. 스키마의 경계에 쓸 수 있는 타입 있는 상수, 그리고 코드 생성 설정입니다. DEQ 파일은 `build_qsl_files`의 두 번째 인자로 넘깁니다.

```rust title="build.rs"
qsl::build_qsl_files(&["schemas/packet.qsl"], &["schemas/config.deq"], out)?;
```

## 상수

```deq title="config.deq"
!define MAX_NAME: u16 = 100;
```

타입은 `u8`부터 `u64`, `i8`부터 `i64`, `usize`, `isize` 중 하나이고, 값은 그 타입에 들어가야 합니다. 표현식은 스키마와 같은 연산자를 쓰며, 정수 타입에 대한 `sizeof(T)`도 쓸 수 있습니다. 뒤에 오는 상수는 앞의 상수를 `!NAME`으로 참조할 수 있습니다.

```deq title="config.deq"
!define HEADER: usize = sizeof(u64) + 2;
!define FRAME: usize = !HEADER * 4;
```

스키마에서는 상수를 이름으로 씁니다.

```qsl
&User[safety] { u64 id; str(1..MAX_NAME) name; }
```

## 설정

```deq title="config.deq"
!config MAX_DATA_SIZE = 65535;
!config USE_HASH = xxHash;
```
설정 | 기본값 | 의미
---|---|---
`MAX_DATA_SIZE` | `65535` | 프레임, 압축을 풀어 복원한 데이터, 디코딩된 값 전체 크기의 상한(바이트). 1부터 `isize::MAX`까지.
`STACK_LIMIT` | `256` | 문자열·배열 버퍼 하나당 인라인 용량 한도(바이트). 경계가 이보다 크면 `String`이나 `Vec`으로 대체됩니다. `0`부터 `u32::MAX`까지.
`USE_HASH` | `xxHash` | 스키마 해시 알고리즘. `xxHash`는 XXH3-64, `xxHash128`은 XXH3-128입니다. `XXH3_64`와 `XXH3_128`도 받습니다.
`QSL_RT` | `false` | `true`이면 최대 프레임 크기가 유한한 타입에 정적 슬롯 풀을 생성합니다. [버퍼와 재사용](/guides/memory/#rt-slot-pool) 참고.
`QSL_RT_DEFAULT` | `16` | 타입당 슬롯 수, 1~64. 바이트가 아니라 슬롯 개수입니다.
설정은 빌드 전체에 적용됩니다. 같은 이름을 두 번 설정하면 서로 다른 파일에서 설정했더라도 에러입니다. 알 수 없는 지시문과 알 수 없는 설정 이름도 에러입니다.

DEQ 파일에는 파일 경로가 들어가지 않습니다. 어떤 파일을 컴파일할지는 `build.rs`가 정합니다.
