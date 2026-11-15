---
title: "제약과 소스"
description: "MAX_DATA_SIZE 기본값은 65535 B, STACK_LIMIT는 256 B입니다. 전자는 입력·복원·합계 데이터 예산을, 후자는 인라인 문자열과 배열을 제한합니다. 프로세스 RSS 한도가 아닙니다. dyn index16은 미지원이며 LLVM 백엔드와 QBProto 핸드셰이크는 향후 계획입니다. 미지원 조합은 생성 단계에서 거부합니다."
qslResources: [{"label": "상세 참조와 소스", "url": "https://github.com/zadr0/QSL"}]
---

로컬 개발 버전 · 2026-11-14.

MAX_DATA_SIZE 기본값은 65535 B, STACK_LIMIT는 256 B입니다. 전자는 입력·복원·합계 데이터 예산을, 후자는 인라인 문자열과 배열을 제한합니다. 프로세스 RSS 한도가 아닙니다. dyn index16은 미지원이며 LLVM 백엔드와 QBProto 핸드셰이크는 향후 계획입니다. 미지원 조합은 생성 단계에서 거부합니다.

상세 참조와 소스 · [English](/en/reference/limits/)
