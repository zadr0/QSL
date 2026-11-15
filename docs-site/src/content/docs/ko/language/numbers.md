---
title: "수치 도메인"
description: "값의 범위로 상태 수를 구하고 유효한 값을 연속된 순위로 바꿉니다. 상수는 데이터 비트가 필요하지 않습니다. step과 values는 명시적인 제약이며 범위를 벗어나거나 간격에 맞지 않는 값은 거부합니다. 반올림 없이 정확히 복원합니다."
qslResources: [{"label": "상세 참조와 소스", "url": "https://github.com/zadr0/QSL"}]
---

로컬 개발 버전 · 2026-11-14.

값의 범위로 상태 수를 구하고 유효한 값을 연속된 순위로 바꿉니다. 상수는 데이터 비트가 필요하지 않습니다. step과 values는 명시적인 제약이며 범위를 벗어나거나 간격에 맞지 않는 값은 거부합니다. 반올림 없이 정확히 복원합니다.

$$
N=\frac{b-a}{s}+1,\qquad r=\frac{x-a}{s},\qquad w=\lceil\log_2N\rceil.
$$

```qsl
&Discrete[safety,compact] {
    i32(-9..9) temperature|step=3|;
    i128 state|values=[-1,0,1]|;
}
```

상세 참조와 소스 · [English](/en/language/numbers/)
