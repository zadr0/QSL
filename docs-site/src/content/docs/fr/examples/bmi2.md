---
title: "Compactage BMI2"
description: "QSL — Compactage BMI2"
---

Quatre champs u32 dans 0..4095 utilisent 12 bits chacun et produisent une trame de 6 octets. Le profil Kaby Lake autorise le writer PEXT prévu si BMI2 est activé pour la cible. Le reader généré reste scalaire. Le kernel PDEP séparé vérifie la restitution exacte; les appels ISA suivent une vérification du CPU.

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
