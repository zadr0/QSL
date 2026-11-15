---
title: "数值域"
description: "范围决定可表示状态的数量，而不是 Rust 类型的名字。QSL 将合法值映射为连续的秩；常量无需数据位。step 和 values 必须显式声明，越界或不在步长上的值会被拒绝。压缩必须精确可逆，不进行舍入。"
qslResources: [{"label": "详细参考与源码", "url": "https://github.com/zadr0/QSL"}]
---

本地开发版本 · 2026-11-14.

范围决定可表示状态的数量，而不是 Rust 类型的名字。QSL 将合法值映射为连续的秩；常量无需数据位。step 和 values 必须显式声明，越界或不在步长上的值会被拒绝。压缩必须精确可逆，不进行舍入。

$$
N=\frac{b-a}{s}+1,\qquad r=\frac{x-a}{s},\qquad w=\lceil\log_2N\rceil.
$$

```qsl
&Discrete[safety,compact] {
    i32(-9..9) temperature|step=3|;
    i128 state|values=[-1,0,1]|;
}
```

详细参考与源码 · [English](/en/language/numbers/)
