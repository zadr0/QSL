---
title: 스키마 식별
description: strict 해시와 light 해시, 해시에 들어가는 요소, 핸드셰이크에서 해시를 비교하는 방법.
---

QSL은 스키마를 분석하면서 타입마다 해시 두 개를 계산하고, 생성된 코드에 상수로 저장합니다. 인코딩과 디코딩 중에는 해시를 계산하지 않으며, 프레임에도 해시가 들어가지 않습니다.

```rust
let strict: u128 = Packet::SCHEMA_HASH_STRICT;
let light: u128 = Packet::SCHEMA_HASH_LIGHT;
let h = Packet::SCHEMA_HASHES; // <Packet as qsl::SchemaIdentity>::SCHEMA_HASHES로도 접근 가능
println!("{} strict={} light={}", h.algorithm.name(), h.strict_hex(), h.light_hex());
```
해시 | 포함 범위
---|---
Strict | 타입·필드 이름, 필드 순서, 타입, 범위, 옵셔널 여부, 포맷 옵션.
Light | 위와 같되 타입·필드 이름은 제외.
`&A { u32 id; }`와 `&B { u32 sequence; }`는 light 해시가 같고 strict 해시가 다릅니다. 타입이 다른 두 필드의 순서를 바꾸면 두 해시가 모두 바뀝니다.

`SCHEMA_HASHES`는 `qsl::SchemaHashes` 값입니다.
필드 | 의미
---|---
`version` | 해시 입력 포맷의 버전. 현재 1.
`algorithm` | `Xxh3_64` 또는 `Xxh3_128`.
`strict`, `light` | 해시 값. XXH3-64에서는 상위 64비트가 0입니다.
`complete` | 타입이 [`QslCustom`](/guides/custom-types/#qslcustom) 타입을 쓰면 `false`. 이때 해시는 커스텀 타입의 Rust 경로와 크기만 반영하고 코덱은 반영하지 않습니다.
해시와 함께 `version`, `algorithm`, `complete`도 비교하십시오. `strict_hex()`와 `light_hex()`는 알고리즘에 따라 해시를 16자리 또는 32자리 16진수로 표시합니다.

## 알고리즘

기본값은 XXH3-64입니다. 128비트를 쓰려면 [DEQ 파일](/language/deq/)에서 설정합니다.

```deq
!config USE_HASH = xxHash128;
```

이 설정은 빌드 전체에 적용됩니다. 두 알고리즘 모두 암호학적 해시가 아닙니다. 불일치는 감지하지만 변조는 감지하지 못합니다.

## 해시를 바꾸는 것

- 상수를 계산하고 Rust 템플릿을 전개한 뒤의 필드 시퀀스.
- 타입, 값 범위, 문자열·배열 경계, 어떤 필드가 옵셔널인지.
- 와이어 포맷 옵션: `compact`, `compact`와 함께 쓴 `alignment`, `optionals=bitmap`, `predict_seq`, `opt_table`과 함께 쓴 `predict_seq`, `opt_table=legacy`, `state_bits`.
- 문자열 코덱: `decompress=true`와 `decompress=true`.
- `qps`와 그 퀀텀. `[qps]`, `[qps=Q16]`, `[quant=Q16]`은 같습니다.
- 기본 옵셔널 필드 포맷에서는 옵셔널 필드의 태그와 태그 폭.

## 해시를 바꾸지 않는 것

`safety`, `lifetime`, `inline`과 그 변형, `default`, 단독으로 쓴 `opt_table`, `compact` 없이 쓴 `alignment`, `aggressive`, `transmutate`, `cache`와 `loyal_cache`, `QSL_RT`와 슬롯 수. 이들은 바이트가 아니라 생성되는 Rust 코드나 메모리 처리를 바꿉니다.

`String`, `&str`, `StackString`, `lifetime`는 경계가 같으면 같은 바이트를 만들므로 해시도 같습니다.

## 활용 방법

해시는 포맷을 보수적으로 기술합니다. 서로 다른 두 스키마가 같은 바이트를 디코딩할 수 없다는 것까지 증명하지는 않습니다. 양쪽이 일치하는지 확인하려면 데이터를 보내기 전에, 직접 만든 핸드셰이크에서 해시를 교환하십시오. QSL은 핸드셰이크를 제공하지 않습니다.
