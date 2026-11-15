---
title: 에러
description: QslError의 배리언트와 각 배리언트가 반환되는 경우.
---

생성된 메서드는 `qsl::QslResult<T>`를 반환합니다. 이는 `Result<T, QslError>`의 별칭입니다. `QslError`에는 배리언트가 11개 있으며 `std::error::Error`를 구현합니다. `Display` 출력은 `QSL error: <Variant>` 형태입니다. 예: `QSL error: InvalidUtf8`.
배리언트 | 반환되는 경우
---|---
`UnexpectedEof` | 프레임이 끝나기 전에 입력이 끝났습니다.
`BufferTooSmall` | 전달한 버퍼가 출력을 담기에 작습니다.
`LengthOverflow` | 길이 계산이 오버플로됩니다.
`InvalidWidth` | 지원하지 않는 폭이나 퀀텀입니다.
`InvalidValue` | 값, 길이, 메타데이터 필드가 허용 범위를 벗어났습니다.
`InvalidUtf8` | 문자열 바이트가 유효한 UTF-8이 아닙니다.
`InvalidSchema` | 스키마가 유효하지 않거나, 해당 작업을 이 스키마에 적용할 수 없습니다. `QslCustom` 검증 함수의 기본 결과이기도 합니다.
`InvalidPadding` | 패딩 비트나 패딩 바이트가 0이 아닙니다.
`PoolExhausted` | 모든 RT 슬롯이 사용 중입니다. 기다리지 않는 `try_acquire`에서만 반환됩니다.
`SizeLimitExceeded` | 프레임이나 디코딩된 데이터가 `MAX_DATA_SIZE`를 넘습니다.
`AllocationFailed` | 실패할 수 있는 할당이 실패했습니다.
파일과 소켓 작업은 `std::io::Error`도 반환할 수 있습니다.

검증에 성공했다는 것은 바이트가 스키마의 형태에 맞는다는 뜻입니다. 누가 보냈는지, 어떤 스키마로 썼는지는 알려 주지 않습니다.
