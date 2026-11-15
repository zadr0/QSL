---
title: "BMI2 패킹"
description: "QSL — BMI2 패킹"
---

0..4095 범위의 u32 필드 네 개는 각각 12비트를 사용해 6바이트 compact 프레임을 만듭니다. Kaby Lake 프로필은 지원되는 PEXT 쓰기 경로를 허용하며 대상 빌드에서 BMI2를 활성화해야 합니다. 생성된 읽기는 스칼라입니다. 별도 PDEP 커널로 정확한 복원을 확인하며 ISA 호출 전에 CPU 지원을 검사합니다.

```qsl
&Bmi2Packet[compact,aggressive[group=true],safety] {
    u32(0..4095) f0;
    u32(0..4095) f1;
    u32(0..4095) f2;
    u32(0..4095) f3;
}
```

```text title="settings.deq"
!config CPU_PROFILE = kaby_lake;
```

```rust
use qsl::optimizer::kernels;
let values = [0u32, 1, 2047, 4095];
kernels::validate4(values, [12; 4])?;
let scalar = kernels::pack4::<12, 12, 12, 12>(values);
#[cfg(target_arch = "x86_64")]
if std::is_x86_feature_detected!("bmi2") {
    // CPU support and all value ranges have been checked.
    let packed = unsafe { kernels::x86::pack4_bmi2::<12, 12, 12, 12>(&values) };
    assert_eq!(packed, scalar);
    let restored = unsafe { kernels::x86::unpack4_bmi2::<12, 12, 12, 12>(packed) };
    assert_eq!(restored, values);
}
```

```powershell
cargo rustc --example bmi2 --release -- -C target-feature=+bmi2
```

[COMPACT](/language/numbers/) · [CPU_PROFILE](/language/deq/)
