---
title: Déclarations
description: "Syntaxe d’un schéma QSL : déclarations, champs, attributs, modificateurs, plages, expressions constantes et commentaires."
---

Un fichier de schéma contient une ou plusieurs déclarations. Chaque déclaration devient un type Rust.

```qsl
&Name[attribute, attribute=value] {
    type field;
    type field|modifier=value|;
    type? optional_field;
}
```
Élément | Signification
---|---
`&Name` | Déclare un type nommé `Name`.
`[ ... ]` | Attributs du type entier. Facultatifs. Voir la [référence des attributs](/reference/attributes/).
`type field;` | Un champ obligatoire.
`type? field;` | Un champ optionnel. Avec une plage, le `?` se place devant elle : `u16?(0..1023) level;`.
`\|...\|` | Modificateurs d’un champ, écrits entre le nom et le point-virgule.
## Champs optionnels en Rust {#optional-fields-in-rust}

Un champ optionnel de type chaîne ou tableau est un champ public `Option<T>` de la structure.

Un champ numérique optionnel est stocké dans un champ privé. Lisez-le avec la méthode du même nom et modifiez-le avec `set_<name>`, qui renvoie `InvalidValue` pour une valeur hors de la plage du champ. Les types qui ont un tel champ reçoivent un constructeur `try_new` qui prend tous les champs dans l’ordre de déclaration et vérifie les valeurs :

```qsl
&Reading[safety] { u32 id; u8? level; }
```

```rust
let mut r = Reading::try_new(42, Some(7))?;
assert_eq!(r.level(), Some(7));
r.set_level(None)?;
```

## Ordre des champs

Les champs obligatoires doivent précéder les champs optionnels. QSL l’impose pour tous les types, avec ou sans attributs.

```qsl
&Ok    { u32 id; u16? code; }
&Error { u16? code; u32 id; }   // erreur de compilation : obligatoire après optionnel
```

Les valeurs obligatoires sont écrites dans la trame dans l’ordre de déclaration. L’ordre des valeurs optionnelles dépend du [mode des champs optionnels](/language/optionals/).

## Tags

Le compilateur attribue à chaque champ un tag dans l’ordre de déclaration, à partir de 1. Vous ne pouvez pas fixer les tags vous-même. L’encodage par défaut des champs optionnels stocke les tags (voir [champs optionnels](/language/optionals/)) : réordonner les champs change donc le format.

Un type sans `optionals=bitmap` peut avoir au plus 64 champs.

## Plages

`A..B` est une plage inclusive. Son sens dépend du type :

```qsl
&Limits {
    u8(1..3)          kind;    // plage de valeurs : 1, 2 ou 3
    str(1..100)       name;    // 1 à 100 octets UTF-8
    str(4)            tag;     // exactement 4 octets, devient [u8; 4]
    array<u16>(0..16) samples; // 0 à 16 éléments
}
```

Les méthodes vérifiées font respecter une plage de valeurs. Avec `compact`, elle détermine aussi le nombre de bits qu’occupe le champ. Une plage de chaîne compte des octets UTF-8, pas des caractères : `"Ада"` fait 6 octets.

## Expressions constantes

Les bornes peuvent être des expressions sur des entiers et des constantes nommées :

```qsl
&Frame {
    str(1..MAX_NAME) name;
    array<u8>(0..MAX_NAME * 2) payload;
}
```

Les expressions acceptent `+ - * / %`, le moins unaire, les parenthèses, `sizeof(T)` pour les types entiers de `u8` à `i64`, et les constantes nommées. Une constante provient d’un `!define` dans un [fichier DEQ](/language/deq/) ou d’une ligne `#NAME = expression;` dans le corps d’un type, qui la définit pour les lignes suivantes. Un dépassement de capacité ou une division par zéro provoque une erreur de compilation.

## Commentaires

`//` ouvre un commentaire de ligne. `/* ... */` délimite un commentaire de bloc.

```qsl
&Packet[safety] {
    u8 id;      // message kind
    u16? a;     /* absent in most frames */
}
```

## Ce qui est rejeté

La compilation échoue dans les cas suivants :

- un champ obligatoire après un champ optionnel ;
- des noms de champs ou de types en double, et des noms de champs qui sont des mots-clés Rust ou qui commencent par `__qsl_` ;
- des bornes invalides ou inversées, et des plages qui sortent du type entier ;
- des attributs ou des modificateurs inconnus ou répétés ;
- plus de 64 champs sans `optionals=bitmap`, ou plus de 128 champs optionnels avec.

Un nom de type qui n’est pas prédéfini est traité comme un [type personnalisé](/guides/custom-types/) et nécessite une implémentation en Rust. Un type personnalisé ne peut pas avoir de plage. La liste des types prédéfinis se trouve dans [types](/language/types/).
