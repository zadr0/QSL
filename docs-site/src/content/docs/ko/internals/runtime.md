---
title: '런타임과 버퍼'
description: 'SerOut과 준비된 버퍼로 할당 및 재사용을 제어합니다. QSL_RT는 크기 상한을 아는 적격 스키마에만 정적 슬롯을 만들며 기본값은 스키마마다 16개입니다. 모두 사용 중이면 숨은 할당 대신 오류를 반환합니다. 플랫폼 전용 소켓과 메모리 영역은 현재 런타임에서 제거되었습니다.'
---

no_std는 std를 제외하며 alloc을 제외하지 않습니다. qsl-core는 항상 #![no_std]입니다. 기본 alloc 기능은 alloc String/Vec, SerOut, 힙 복원을 제공합니다. 할당자는 애플리케이션이 제공하며 QSL은 전역 할당자를 정의하지 않습니다. 기본 기능을 끄면 호출자 버퍼만 사용할 수 있습니다.

코어 FramePool/FrameStorage를 구현하는 no_std 정적 영역. 풀이 차면 즉시 PoolExhausted 반환.
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString은 스택 또는 힙의 Box에서 같은 코덱을 유지합니다. 큰 힙 문자열은 목적지로 바로 복원합니다. StackString과 InlineArray는 객체 내부에 제한된 용량을 저장합니다. 명시적 aggressive[shift=true]는 NonZero 길이를 선택하여 None과 Some(empty)를 구분합니다.

빌드 스크립트와 테스트의 std는 대상 런타임에 포함되지 않습니다. 실제 생성 스키마를 할당자 없이 실행하고 별도 no_std 라이브러리로 alloc 컨테이너를 검증합니다. 코어의 두 구성과 qsl-rt를 wasm32-wasip2에서도 빌드합니다.

[Memory](/guides/memory/) · [Safety](/guides/safety/)

생성기는 증명된 최대 프레임 크기로 출력 형식을 선택합니다. STACK_LIMIT 이하이면 경계를 포함하여 `StackOut<N>`을, 더 크거나 상한을 모르면 alloc::`Vec<u8>`을 사용합니다. 전송 헤더는 포함되지만 출력 객체 자체의 길이와 정렬 메타데이터는 제외됩니다.
