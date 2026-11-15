---
title: "BMI2-Packung"
description: "QSL — BMI2-Packung"
---

Vier u32-Felder im Bereich 0..4095 benötigen je 12 Bit und ergeben einen 6-Byte-Frame. Das Kaby-Lake-Profil erlaubt den unterstützten PEXT-Writer bei aktiviertem BMI2-Zielmerkmal. Der generierte Reader bleibt skalar. Der getrennte PDEP-Kernel stellt Werte exakt wieder her; vor ISA-Aufrufen wird die CPU geprüft.

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
