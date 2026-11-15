---
title: "Domaines numériques"
description: "La plage détermine le nombre d’états. QSL transforme chaque valeur valide en rang continu ; une constante ne consomme aucun bit. step et values sont des contraintes explicites. Les valeurs hors domaine ou incompatibles avec le pas sont rejetées. La transformation est réversible sans arrondi."
qslResources: [{"label": "Référence détaillée et sources", "url": "https://github.com/zadr0/QSL"}]
---

Édition locale de développement · 2026-11-14.

La plage détermine le nombre d’états. QSL transforme chaque valeur valide en rang continu ; une constante ne consomme aucun bit. step et values sont des contraintes explicites. Les valeurs hors domaine ou incompatibles avec le pas sont rejetées. La transformation est réversible sans arrondi.

$$
N=\frac{b-a}{s}+1,\qquad r=\frac{x-a}{s},\qquad w=\lceil\log_2N\rceil.
$$

```qsl
&Discrete[safety,compact] {
    i32(-9..9) temperature|step=3|;
    i128 state|values=[-1,0,1]|;
}
```

Référence détaillée et sources · [English](/en/language/numbers/)
