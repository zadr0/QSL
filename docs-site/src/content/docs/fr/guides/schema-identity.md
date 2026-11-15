---
title: Identité du schéma
description: "Les hashes strict et léger d’un schéma, ce qu’ils couvrent et comment les comparer lors d’une poignée de main."
---

QSL calcule deux hashes pour chaque type pendant l’analyse du schéma et les stocke sous forme de constantes dans le code généré. L’encodage et le décodage ne les calculent jamais, et les trames ne les contiennent pas.

```rust
let strict: u128 = Packet::SCHEMA_HASH_STRICT;
let light: u128 = Packet::SCHEMA_HASH_LIGHT;
let h = Packet::SCHEMA_HASHES; // aussi <Packet as qsl::SchemaIdentity>::SCHEMA_HASHES
println!("{} strict={} light={}", h.algorithm.name(), h.strict_hex(), h.light_hex());
```
Hash | Couvre
---|---
Strict | Noms du type et des champs, ordre des champs, types, plages, caractère optionnel et options de format.
Léger | La même chose, sans les noms du type et des champs.
`&A { u32 id; }` et `&B { u32 sequence; }` ont le même hash léger et des hashes stricts différents. Permuter deux champs de types différents change les deux.

`SCHEMA_HASHES` est une valeur de type `qsl::SchemaHashes` :
Champ | Signification
---|---
`version` | Version du format d’entrée du hash, actuellement 1.
`algorithm` | `Xxh3_64` ou `Xxh3_128`.
`strict`, `light` | Les hashes. Avec XXH3-64, les 64 bits de poids fort sont à zéro.
`complete` | `false` si le type utilise un type [`QslCustom`](/guides/custom-types/#qslcustom). Le hash ne couvre alors que le chemin Rust et la taille du type personnalisé, pas son codec.
Comparez `version`, `algorithm` et `complete` en même temps que le hash. `strict_hex()` et `light_hex()` formatent un hash en 16 ou 32 chiffres hexadécimaux, selon l’algorithme.

## Algorithme

XXH3-64 par défaut. Pour 128 bits, réglez-le dans un [fichier DEQ](/language/deq/) :

```deq
!config USE_HASH = xxHash128;
```

Le paramètre s’applique à toute la compilation. Les deux algorithmes sont non cryptographiques : ils détectent les incohérences, pas les falsifications.

## Ce qui change le hash

- La séquence des champs, après évaluation des constantes et expansion des modèles Rust.
- Les types, les plages de valeurs, les bornes des chaînes et des tableaux, et le caractère optionnel des champs.
- Les options de format de trame : `compact`, `alignment` avec `compact`, `optionals=bitmap`, `predict_seq`, `predict_seq` avec `opt_table`, `opt_table=legacy`, `state_bits`.
- Les codecs de chaîne : `decompress=true` et `decompress=true`.
- `qps` et son quantum. `[qps]`, `[qps=Q16]` et `[quant=Q16]` sont équivalents.
- Dans le format par défaut des champs optionnels, les tags des champs optionnels et la largeur des tags.

## Ce qui ne le change pas

`safety`, `lifetime`, `inline` et ses variantes, `default`, `opt_table` seul, `alignment` sans `compact`, `aggressive`, `transmutate`, `cache` et `loyal_cache`, `QSL_RT` et le nombre d’emplacements. Ces options modifient le code Rust généré ou la gestion de la mémoire, pas les octets.

`String`, `&str`, `StackString` et `lifetime` produisent les mêmes octets pour les mêmes bornes : leur hash est donc identique.

## Utilisation

Les hashes décrivent le format de manière conservatrice. Ils ne prouvent pas que deux schémas différents ne peuvent pas décoder les mêmes octets. Pour vous assurer que les deux côtés sont d’accord, échangez les hashes dans votre propre poignée de main avant d’envoyer des données. QSL ne fournit pas de poignée de main.
