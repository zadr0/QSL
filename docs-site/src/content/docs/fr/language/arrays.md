---
title: 'Tableaux et fenêtres de bits'
description: 'Une longueur de tableau compte des éléments ; une longueur de chaîne compte des octets UTF-8. InlineArray réserve une capacité dans la structure, tandis que Vec possède une allocation séparée. STACK_LIMIT limite strictement le stockage intégré. Le descripteur de Vec n’est pas sa charge utile.'
---

La longueur d’un tableau compte les éléments ; celle d’une chaîne compte les octets UTF-8. `array<u16>(0..16)` autorise de zéro à seize éléments et `array<u16>(16)` en exige exactement seize.

Vec possède un buffer séparé via alloc. InlineArray contient une capacité bornée dans l’objet ; les tableaux fixes peuvent utiliser `[T;N]`. STACK_LIMIT limite le stockage intégré. Le descripteur de Vec n’est pas le payload.

$$
B_{\mathrm{array}}=H(n)+\sum_{i=1}^{n} w_i.
$$

H est la métadonnée de longueur en bits, n le nombre d’éléments et w la largeur du codec de chaque élément. L’alignement est compté séparément. Une longueur ne permet pas de lire hors du cadre. Jusqu’à 32 niveaux imbriqués sont pris en charge ; domaines et longueurs sont vérifiés séparément. Le registre dépend de la position en bits et n’est pas toujours de 128 bits.

`aggressive[shift=true]` explicite peut choisir une longueur NonZero adaptée à InlineArray. La valeur par défaut est u32. Cette optimisation physique conserve le format wire et distingue None de Some(empty).
[Read windows](/internals/optimizer/) · [Memory](/guides/memory/)

no_std exclut std, pas alloc. qsl-core utilise toujours #![no_std]. La fonctionnalité alloc, active par défaut, fournit alloc String/Vec, SerOut et la restauration dans le tas. L’application fournit l’allocateur ; QSL ne déclare aucun allocateur global. Sans fonctionnalités par défaut, seuls les buffers de l’appelant sont utilisés.
