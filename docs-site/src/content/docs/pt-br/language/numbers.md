---
title: "Domínios numéricos"
description: "O intervalo define o número de estados. QSL transforma cada valor válido em um rank contínuo; constantes usam zero bits. step e values são restrições explícitas. Valores fora do domínio ou do passo são rejeitados. A transformação é reversível e não arredonda."
qslResources: [{"label": "Referência detalhada e fontes", "url": "https://github.com/zadr0/QSL"}]
---

Edição local de desenvolvimento · 2026-11-14.

O intervalo define o número de estados. QSL transforma cada valor válido em um rank contínuo; constantes usam zero bits. step e values são restrições explícitas. Valores fora do domínio ou do passo são rejeitados. A transformação é reversível e não arredonda.

$$
N=\frac{b-a}{s}+1,\qquad r=\frac{x-a}{s},\qquad w=\lceil\log_2N\rceil.
$$

```qsl
&Discrete[safety,compact] {
    i32(-9..9) temperature|step=3|;
    i128 state|values=[-1,0,1]|;
}
```

Referência detalhada e fontes · [English](/en/language/numbers/)
