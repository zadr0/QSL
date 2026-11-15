---
title: 'Runtime et tampons'
description: 'SerOut et les tampons préparés contrôlent allocations et réutilisation. QSL_RT réserve des emplacements statiques uniquement aux schémas bornés admissibles : 16 par schéma par défaut. Leur épuisement renvoie une erreur sans allocation cachée. Les sockets et régions propres à la plateforme ne font plus partie du runtime actuel.'
---

no_std exclut std, pas alloc. qsl-core utilise toujours #![no_std]. La fonctionnalité alloc, active par défaut, fournit alloc String/Vec, SerOut et la restauration dans le tas. L’application fournit l’allocateur ; QSL ne déclare aucun allocateur global. Sans fonctionnalités par défaut, seuls les buffers de l’appelant sont utilisés.

Régions statiques no_std implémentant FramePool/FrameStorage. Un pool plein renvoie immédiatement PoolExhausted.
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString conserve son codec dans l’objet ou dans une Box sur le tas ; les grandes chaînes sont restaurées directement à destination. StackString et InlineArray contiennent leur capacité bornée. aggressive[shift=true] peut choisir une longueur NonZero, en distinguant None de Some(empty).

Les scripts de compilation et les tests peuvent utiliser std sans l’inclure dans le runtime cible. De vrais schémas générés s’exécutent sans allocateur ; une autre bibliothèque no_std vérifie les conteneurs alloc. Les deux profils du cœur et qsl-rt compilent aussi pour wasm32-wasip2.

[Memory](/guides/memory/) · [Safety](/guides/safety/)

Le générateur choisit la sortie selon la taille maximale démontrée de la trame. Jusqu’à STACK_LIMIT inclus, il utilise `StackOut<N>`; une borne supérieure ou inconnue sélectionne alloc::`Vec<u8>`. Les en-têtes transmis comptent, les métadonnées de l’objet de sortie non.
