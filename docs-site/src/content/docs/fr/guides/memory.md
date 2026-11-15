---
title: Tampons et réutilisation
description: "Écrire dans vos propres tampons, décoder dans des valeurs existantes, le pool statique d’emplacements RT et les trames de sortie en cache."
---

Le générateur choisit la sortie selon la taille maximale démontrée de la trame. Jusqu’à STACK_LIMIT inclus, il utilise `StackOut<N>`; une borne supérieure ou inconnue sélectionne alloc::`Vec<u8>`. Les en-têtes transmis comptent, les métadonnées de l’objet de sortie non.

String, Vec et Box utilisent alloc dans no_std; l’application fournit l’allocateur. InlineString est intégré si sa taille native respecte STACK_LIMIT, sinon placé dans une Box. La capacité du schéma détermine ce choix.

Le reuse unsafe préparé exige une capacité suffisante avant l’appel et conserve les buffers String, Vec et InlineString en Box compatibles. Il ne les agrandit pas. Objet initialisé et destination non initialisée ont des contrats distincts.

<a id="reading-into-existing-memory"></a>

## Exemple {#example}

```qsl
&Record[safety] { u64 id; }
```

```rust
let value = Record { id: 42 };
let mut buffer = [0u8; 64];
let used = value.serialize_checked_to(&mut buffer)?;
let mut destination = core::mem::MaybeUninit::<Record>::uninit();
let restored = Record::deserialize_checked_into(&buffer[..used], &mut destination)?;
assert_eq!(restored.id, 42);
```

[STACK_LIMIT](/language/deq/) · [String](/language/strings/) · [RT](/internals/runtime/)

## QSL_RT {#rt-slot-pool}

LocalRegions · StaticRegions · FramePool · FrameStorage. 1–64 slots; lease → Drop. PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.
