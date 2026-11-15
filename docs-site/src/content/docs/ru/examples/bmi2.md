---
title: "Упаковка BMI2"
description: "QSL — Упаковка BMI2"
---

Четыре поля u32 с диапазоном 0..4095 требуют по 12 бит и дают compact-кадр 6 B. Профиль Kaby Lake разрешает поддержанный групповой PEXT-writer; в целевой сборке требуется BMI2. Сгенерированное чтение остаётся скалярным. Отдельное ядро PDEP показывает точное восстановление. Перед вызовом ISA проверяется поддержка CPU.

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
