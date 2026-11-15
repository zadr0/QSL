---
title: Types
description: "Les types prédéfinis de QSL, les types Rust qu’ils génèrent et leur encodage par défaut dans la trame."
---

## Types prédéfinis
Schéma | Type Rust | Remarques
---|---|---
`u8` `u16` `u32` `u64` `i8` `i16` `i32` `i64` | Même type primitif |
`T(A..B)` pour un entier `T` | Même type primitif | La valeur doit être dans la plage. `compact` et `qps` peuvent la réduire dans la trame.
`flags(N)` | `u8`, `u16`, `u32` ou `u64`, selon `N` | `N` va de 1 à 64. Les bits au-delà de `N` doivent être à zéro.
`str`, `string`, `String` | `String` | Longueur variable, limitée seulement par `MAX_DATA_SIZE`.
`str(A..B)` | `String` ou un autre type de chaîne | Bornes inclusives, en octets UTF-8. Voir [chaînes](/language/strings/).
`str(N)` | `[u8; N]` | Exactement `N` octets d’UTF-8, sans longueur dans la trame. `String` si `N` dépasse `STACK_LIMIT`.
`array<T>`, `vec<T>` | `Vec<T>` | `T` est un type entier prédéfini ou une [primitive enregistrée](/guides/custom-types/#validated-scalar-wrappers).
`array<T>(N)` | `[T; N]` | Exactement `N` éléments, sans compteur dans la trame. `Vec<T>` si le tableau dépasse `STACK_LIMIT` octets.
`array<T>(A..B)` | `Vec<T>` | Nombre d’éléments, bornes incluses. `InlineArray<T, B>` avec `aggressive` s’il tient dans `STACK_LIMIT`.
`padding(N)` | `()` | `N` octets nuls dans la trame.
`T?` | `Option<T>` | Champ optionnel. Les champs numériques optionnels passent par des accesseurs, voir [déclarations](/language/declarations/#optional-fields-in-rust).
Tout autre nom | Votre type Rust | Doit implémenter `QslCustom`, voir [types personnalisés](/guides/custom-types/).
Avec l’attribut `lifetime`, les chaînes et les tableaux de `u8` empruntent l’entrée sous la forme `&'a str` et `&'a [u8]`.

## Encodage par défaut dans la trame

Sans attribut qui modifie le format :

- **Les entiers** sont en little-endian, sur leur largeur native. Un `u16(0..1023)` occupe 2 octets.
- **Les chaînes** sont une longueur en octets de type `u32`, suivie des octets UTF-8. Une chaîne fixe `str(N)` n’a pas de longueur.
- **Les tableaux dynamiques** sont un nombre d’éléments de type `u32`, suivi des éléments. Les tableaux fixes n’ont pas de compteur.
- **Les champs obligatoires** suivent l’ordre de déclaration.
- **Les champs optionnels** ajoutent des métadonnées de présence avant les valeurs. Voir [champs optionnels](/language/optionals/).

La trame ne transporte aucune information de type. Les deux côtés doivent connaître le schéma.

## Taille de trame et taille de l’objet

Les deux sont indépendantes :

```qsl
&Reading[compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

Dans la trame, cela fait 3 + 5 + 10 = 18 bits, écrits sur 3 octets. En mémoire, `Reading` est une structure `#[repr(C)]` contenant un `u8`, un `u8` et un `u16` : 4 octets. `Reading::NATIVE_BYTES` contient cette taille.
