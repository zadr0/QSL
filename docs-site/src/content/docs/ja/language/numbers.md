---
title: "数値ドメイン"
description: "値域から状態数を求め、正当な値を連続したランクに変換します。定数のデータ幅はゼロです。step と values は明示的な制約であり、範囲外や刻みに一致しない値は拒否されます。圧縮は丸めを行わず、完全に可逆です。"
qslResources: [{"label": "詳細リファレンスとソース", "url": "https://github.com/zadr0/QSL"}]
---

ローカル開発版 · 2026-11-14.

値域から状態数を求め、正当な値を連続したランクに変換します。定数のデータ幅はゼロです。step と values は明示的な制約であり、範囲外や刻みに一致しない値は拒否されます。圧縮は丸めを行わず、完全に可逆です。

$$
N=\frac{b-a}{s}+1,\qquad r=\frac{x-a}{s},\qquad w=\lceil\log_2N\rceil.
$$

```qsl
&Discrete[safety,compact] {
    i32(-9..9) temperature|step=3|;
    i128 state|values=[-1,0,1]|;
}
```

詳細リファレンスとソース · [English](/en/language/numbers/)
