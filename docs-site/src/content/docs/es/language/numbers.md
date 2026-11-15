---
title: "Dominios numéricos"
description: "El rango determina el número de estados. QSL transforma cada valor válido en un rango continuo; una constante usa cero bits. step y values son restricciones explícitas. Se rechazan valores fuera del dominio o del paso. La transformación es reversible y no redondea."
qslResources: [{"label": "Referencia detallada y fuentes", "url": "https://github.com/zadr0/QSL"}]
---

Edición local de desarrollo · 2026-11-14.

El rango determina el número de estados. QSL transforma cada valor válido en un rango continuo; una constante usa cero bits. step y values son restricciones explícitas. Se rechazan valores fuera del dominio o del paso. La transformación es reversible y no redondea.

$$
N=\frac{b-a}{s}+1,\qquad r=\frac{x-a}{s},\qquad w=\lceil\log_2N\rceil.
$$

```qsl
&Discrete[safety,compact] {
    i32(-9..9) temperature|step=3|;
    i128 state|values=[-1,0,1]|;
}
```

Referencia detallada y fuentes · [English](/en/language/numbers/)
