---
title: '배열과 비트 창'
description: '배열 길이는 원소 수이고 문자열 길이는 UTF-8 바이트 수입니다. InlineArray는 정해진 용량을 구조체 안에 두며 Vec는 별도 메모리를 할당합니다. STACK_LIMIT는 인라인 저장을 엄격히 제한합니다. Vec의 관리 정보를 페이로드로 복사하지 않습니다.'
---

배열 길이는 요소 수이며 문자열 길이는 UTF-8 바이트 수입니다. 가변 길이 `array<u16>(0..16)`은 0~16개, 고정 길이 `array<u16>(16)`은 정확히 16개를 요구합니다.

Vec은 alloc으로 별도 버퍼를 소유합니다. InlineArray는 객체 내부에 제한된 용량을 저장하며 고정 배열에는 `[T;N]`을 사용할 수 있습니다. STACK_LIMIT는 내장 저장소를 제한합니다. Vec 설명자의 크기는 페이로드 크기가 아닙니다.

$$
B_{\mathrm{array}}=H(n)+\sum_{i=1}^{n} w_i.
$$

H는 길이 메타데이터 비트 수, n은 요소 수, w는 각 요소의 인코딩 폭입니다. 정렬은 별도로 계산합니다. 길이는 프레임 밖 읽기를 허용하지 않습니다. 중첩은 최대 32단계이며 요소 범위와 길이를 별도로 검증합니다. 읽기 레지스터는 비트 위치에 따라 선택하며 항상 128비트가 아닙니다.

명시적 `aggressive[shift=true]`는 InlineArray에 맞는 NonZero 길이 단어를 선택할 수 있습니다. 기본값은 u32입니다. 물리 최적화는 wire를 바꾸지 않으며 None과 Some(empty)를 구분합니다.
[Read windows](/internals/optimizer/) · [Memory](/guides/memory/)

no_std는 std를 제외하며 alloc을 제외하지 않습니다. qsl-core는 항상 #![no_std]입니다. 기본 alloc 기능은 alloc String/Vec, SerOut, 힙 복원을 제공합니다. 할당자는 애플리케이션이 제공하며 QSL은 전역 할당자를 정의하지 않습니다. 기본 기능을 끄면 호출자 버퍼만 사용할 수 있습니다.
