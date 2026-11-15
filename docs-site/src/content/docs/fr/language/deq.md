---
title: Configuration DEQ
description: "Les fichiers DEQ contiennent des constantes typées et des paramètres valables pour toute la compilation, comme MAX_DATA_SIZE, STACK_LIMIT et le pool RT."
---

Un fichier `.deq` contient deux sortes de directives : des constantes typées que les schémas peuvent utiliser dans leurs bornes, et des paramètres de génération de code. Passez les fichiers DEQ en second argument de `build_qsl_files` :

```rust title="build.rs"
qsl::build_qsl_files(&["schemas/packet.qsl"], &["schemas/config.deq"], out)?;
```

## Constantes

```deq title="config.deq"
!define MAX_NAME: u16 = 100;
```

Le type est l’un de `u8` à `u64`, `i8` à `i64`, `usize` ou `isize`, et la valeur doit y tenir. Les expressions utilisent les mêmes opérateurs que les schémas, y compris `sizeof(T)` pour un type entier. Une constante peut faire référence à une constante définie plus haut, sous la forme `!NAME` :

```deq title="config.deq"
!define HEADER: usize = sizeof(u64) + 2;
!define FRAME: usize = !HEADER * 4;
```

Les schémas utilisent la constante par son nom :

```qsl
&User[safety] { u64 id; str(1..MAX_NAME) name; }
```

## Paramètres

```deq title="config.deq"
!config MAX_DATA_SIZE = 65535;
!config USE_HASH = xxHash;
```
Paramètre | Défaut | Signification
---|---|---
`MAX_DATA_SIZE` | `65535` | Borne supérieure, en octets, d’une trame, des données compressées une fois restaurées et de la taille totale des valeurs décodées. De 1 à `isize::MAX`.
`STACK_LIMIT` | `256` | Budget de capacité intégrée, en octets, par tampon de chaîne ou de tableau. Au-delà, le champ se rabat sur `String` ou `Vec`. De `0` à `u32::MAX`.
`USE_HASH` | `xxHash` | Algorithme de hash du schéma. `xxHash` correspond à XXH3-64, `xxHash128` à XXH3-128. `XXH3_64` et `XXH3_128` sont aussi acceptés.
`QSL_RT` | `false` | `true` génère un pool statique d’emplacements pour les types dont la taille de trame maximale est finie. Voir [tampons et réutilisation](/guides/memory/#rt-slot-pool).
`QSL_RT_DEFAULT` | `16` | Emplacements par type, de 1 à 64. Compte des emplacements, pas des octets.
Les paramètres s’appliquent à toute la compilation. Définir deux fois le même nom, même dans des fichiers différents, est une erreur. Les directives inconnues et les noms de paramètres inconnus sont aussi des erreurs.

Les fichiers DEQ ne contiennent aucun chemin de fichier. C’est `build.rs` qui décide quels fichiers sont compilés.
