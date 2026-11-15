---
title: Chaînes et possession
description: 'Les types Rust que peut devenir une chaîne QSL, quand chacun est choisi, et ce que le décodage fait de la mémoire.'
---

UTF-8 se compte en octets : «Яa» occupe 3 B. String possède la mémoire dynamique, &str emprunte l’entrée vivante, StackString et InlineString ont une capacité bornée. QSL-LZ ne compresse que si le résultat est réellement plus petit ; sinon il conserve raw. Le graphe montre distribution, littéraux et références arrière.

## Stockage et longueur

InlineString conserve son codec dans l’objet ou dans une Box sur le tas ; les grandes chaînes sont restaurées directement à destination. StackString et InlineArray contiennent leur capacité bornée. aggressive[shift=true] peut choisir une longueur NonZero, en distinguant None de Some(empty).

## QSL-LZ {#compressed-strings}

Chaque token développé, distance de référence, UTF-8 et budget total est vérifié. La longueur stockée n’autorise pas une sortie restaurée plus grande. Les longueurs sont des octets UTF-8, pas des caractères Unicode.
[Arrays](/language/arrays/) · [Limits](/reference/limits/) · [Memory](/guides/manual-layout/) · [QPS](/guides/smaller-frames/)

no_std exclut std, pas alloc. qsl-core utilise toujours #![no_std]. La fonctionnalité alloc, active par défaut, fournit alloc String/Vec, SerOut et la restauration dans le tas. L’application fournit l’allocateur ; QSL ne déclare aucun allocateur global. Sans fonctionnalités par défaut, seuls les buffers de l’appelant sont utilisés.

Le générateur choisit la sortie selon la taille maximale démontrée de la trame. Jusqu’à STACK_LIMIT inclus, il utilise `StackOut<N>`; une borne supérieure ou inconnue sélectionne alloc::`Vec<u8>`. Les en-têtes transmis comptent, les métadonnées de l’objet de sortie non.
