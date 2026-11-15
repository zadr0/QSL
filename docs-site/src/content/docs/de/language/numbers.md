---
title: "Numerische Domänen"
description: "Der Wertebereich bestimmt die Anzahl der Zustände. QSL bildet gültige Werte auf fortlaufende Ränge ab; Konstanten benötigen null Bits. step und values sind explizite Einschränkungen. Ungültige oder nicht zum Schritt passende Werte werden abgelehnt. Die Abbildung ist ohne Rundung umkehrbar."
qslResources: [{"label": "Ausführliche Referenz und Quellcode", "url": "https://github.com/zadr0/QSL"}]
---


Der Wertebereich bestimmt die Anzahl der Zustände. QSL bildet gültige Werte auf fortlaufende Ränge ab; Konstanten benötigen null Bits. step und values sind explizite Einschränkungen. Ungültige oder nicht zum Schritt passende Werte werden abgelehnt. Die Abbildung ist ohne Rundung umkehrbar.

$$
N=\frac{b-a}{s}+1,\qquad r=\frac{x-a}{s},\qquad w=\lceil\log_2N\rceil.
$$

```qsl
&Discrete[safety,compact] {
    i32(-9..9) temperature|step=3|;
    i128 state|values=[-1,0,1]|;
}
```

Ausführliche Referenz und Quellcode · [English](/en/language/numbers/)
