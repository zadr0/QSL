---
title: 버퍼와 재사용
description: 직접 준비한 버퍼에 쓰기, 기존 값에 디코딩하기, 정적 RT 슬롯 풀, 캐시된 출력 프레임.
---

생성기는 증명된 최대 프레임 크기로 출력 형식을 선택합니다. STACK_LIMIT 이하이면 경계를 포함하여 `StackOut<N>`을, 더 크거나 상한을 모르면 alloc::`Vec<u8>`을 사용합니다. 전송 헤더는 포함되지만 출력 객체 자체의 길이와 정렬 메타데이터는 제외됩니다.

String, Vec, Box는 no_std에서 alloc을 사용하며 할당자는 애플리케이션이 제공합니다. InlineString의 네이티브 크기가 STACK_LIMIT에 들어가면 객체 안에 저장하고, 아니면 Box를 사용합니다. 현재 값의 길이가 아닌 스키마 용량으로 결정합니다.

준비된 unsafe reuse는 호출 전에 충분한 용량을 요구하며 지원되는 String, Vec, `Box<InlineString>` 버퍼를 유지합니다. 이 경로에서는 버퍼를 확장하지 않습니다. 초기화된 객체와 초기화되지 않은 대상은 별도 계약입니다.

<a id="reading-into-existing-memory"></a>

## 예제 {#example}

```qsl
&Record[safety] { u64 id; }
```

```rust
let value = Record { id: 42 };
let mut buffer = [0u8; 64];
let used = value.serialize_checked_to(&mut buffer)?;
let mut destination = core::mem::MaybeUninit::<Record>::uninit();
let restored = Record::deserialize_checked_into(&buffer[..used], &mut destination)?;
assert_eq!(restored.id, 42);
```

[STACK_LIMIT](/language/deq/) · [String](/language/strings/) · [RT](/internals/runtime/)

## QSL_RT {#rt-slot-pool}

LocalRegions · StaticRegions · FramePool · FrameStorage. 1–64 slots; lease → Drop. PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.
