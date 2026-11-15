---
title: 문자열과 소유권
description: QSL 문자열이 될 수 있는 Rust 타입, 각 타입이 선택되는 조건, 디코딩이 메모리를 다루는 방식.
---

UTF-8은 바이트로 셉니다. «Яa»는 3 B입니다. String은 동적 메모리를 소유하며 &str은 살아 있는 입력을 빌립니다. StackString과 InlineString은 용량이 제한됩니다. QSL-LZ는 실제로 작아질 때만 압축하고 나머지는 raw로 저장합니다. 그래프에서 분포, 리터럴, 뒤쪽 참조를 확인합니다.

## 저장 방식과 길이

InlineString은 스택 또는 힙의 Box에서 같은 코덱을 유지합니다. 큰 힙 문자열은 목적지로 바로 복원합니다. StackString과 InlineArray는 객체 내부에 제한된 용량을 저장합니다. 명시적 aggressive[shift=true]는 NonZero 길이를 선택하여 None과 Some(empty)를 구분합니다.

## QSL-LZ {#compressed-strings}

확장 token, 역참조 거리, UTF-8과 합산 예산을 각각 검사합니다. 저장 길이 헤더가 더 큰 복원 출력을 허용하지 않습니다. 길이는 Unicode 문자 수가 아닌 UTF-8 바이트 수입니다.
[Arrays](/language/arrays/) · [Limits](/reference/limits/) · [Memory](/guides/manual-layout/) · [QPS](/guides/smaller-frames/)

no_std는 std를 제외하며 alloc을 제외하지 않습니다. qsl-core는 항상 #![no_std]입니다. 기본 alloc 기능은 alloc String/Vec, SerOut, 힙 복원을 제공합니다. 할당자는 애플리케이션이 제공하며 QSL은 전역 할당자를 정의하지 않습니다. 기본 기능을 끄면 호출자 버퍼만 사용할 수 있습니다.

생성기는 증명된 최대 프레임 크기로 출력 형식을 선택합니다. STACK_LIMIT 이하이면 경계를 포함하여 `StackOut<N>`을, 더 크거나 상한을 모르면 alloc::`Vec<u8>`을 사용합니다. 전송 헤더는 포함되지만 출력 객체 자체의 길이와 정렬 메타데이터는 제외됩니다.
