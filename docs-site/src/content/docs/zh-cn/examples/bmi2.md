---
title: "BMI2 打包"
description: "QSL — BMI2 打包"
---

四个范围为 0..4095 的 u32 字段各需 12 位，生成 6 字节紧凑帧。Kaby Lake 配置允许受支持的 PEXT 写入路径，目标编译必须启用 BMI2。生成的读取路径仍为标量。独立 PDEP 内核演示精确恢复；调用 ISA 前检查 CPU 支持。

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
