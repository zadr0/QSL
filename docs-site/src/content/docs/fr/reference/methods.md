---
title: Méthodes générées
description: "Les méthodes et constantes que QSL génère sur chaque type de schéma, regroupées selon ce qu’elles exigent de l’appelant."
qslResources: [{"label": "docs.rs", "url": "https://docs.rs/qsl/0.3.0/qsl/"}]
---

Chaque déclaration produit un type Rust doté des méthodes ci-dessous. Leur présence dépend des attributs du schéma. Pour la liste complète des éléments publics des crates du runtime, voir docs.rs.

## Vérifiées : sûres sur des données non fiables

Générées uniquement quand le type a `[safety]` :
Méthode | Signature (simplifiée) | Remarques
---|---|---
`deserialize_checked` | `fn(&[u8]) -> QslResult<Self>` | Valide pendant le décodage. Avec `lifetime`, l’entrée est `&'a [u8]`.
`deserialize_checked_into` | `fn(&[u8], &mut MaybeUninit<Self>) -> QslResult<&mut Self>` | Décode dans votre mémoire. En cas d’erreur, la destination reste non initialisée.
Générées pour tous les types :
Méthode | Signature (simplifiée) | Remarques
---|---|---
`validate` | `fn(&[u8]) -> QslResult<()>` | Vérifie une trame entière sans construire de valeur. Inutile avant `deserialize_checked`.
`validate_values` | `fn(&self) -> QslResult<()>` | Vérifie les plages, les longueurs, l’UTF-8 des chaînes fixes et le budget `MAX_DATA_SIZE`.
`serialize_checked` | `fn(&self) -> QslResult<`Vec<u8>`>` | Vérifie la valeur, puis l’encode.
`serialize_checked_to` | `fn(&self, &mut [u8]) -> QslResult<usize>` | Encode dans une slice d’au moins `serialized_size()` octets et renvoie le nombre d’octets écrits.
Les types qui ont un champ numérique optionnel reçoivent aussi :
Méthode | Signature (simplifiée) | Remarques
---|---|---
`try_new` | `fn(field, ...) -> QslResult<Self>` | Prend tous les champs dans l’ordre de déclaration, les optionnels sous forme d’`Option<T>`, et vérifie les valeurs.
`name` | `fn(&self) -> Option<T>` | Lit le champ optionnel `name`.
`set_name` | `fn(&mut self, Option<T>) -> QslResult<()>` | Le modifie. `InvalidValue` si la valeur est hors plage.
Voir [champs optionnels en Rust](/language/declarations/#optional-fields-in-rust).

## De confiance : l’appelant garantit la validité

Ce sont des méthodes du trait `qsl::Serializable`. Les types avec `lifetime` implémentent à la place `qsl::BorrowSerializable<'a>`, dont les décodeurs s’appellent `deserialize_borrowed`, `deserialize_borrowed_into` et `deserialize_borrowed_unsafe`. Dans les deux traits, toutes les méthodes sauf `serialized_size` et `get_size` sont `unsafe`.
Méthode | Ce que l’appelant garantit
---|---
`serialize`, `serialize_to`, `serialize_into_unsafe`, `serialize_unsafe` | La valeur est valide pour le schéma ; la destination dispose de `serialized_size()` octets accessibles en écriture.
`deserialize`, `deserialize_unsafe` | L’entrée est une trame unique, complète et valide.
`deserialize_into(ptr, len, out)` | Trame valide ; `out` est accessible en écriture, aligné et ne chevauche pas l’entrée. L’ancienne valeur de `out` n’est pas détruite.
`deserialize_reuse(input, &mut value)` | Trame valide ; chaque tampon de chaîne réutilisé a une capacité suffisante. Conserve les tampons quand la disposition le permet.
`deserialize_prepared_into(ptr, len, &mut value)` | Idem, avec un pointeur brut et une longueur.
Méthodes sûres du trait :
Méthode | Renvoie
---|---
`serialized_size(&self)` | L’espace à fournir pour cette valeur : la longueur exacte de la trame, ou une borne supérieure pour les valeurs avec des chaînes compressées.
`get_size()` | La plus petite longueur de trame possible.
## Spécialisées
Méthode | Disponibilité
---|---
`serialize()` | Avec un pool RT : sûre, vérifie la valeur, attend un emplacement libre et renvoie un bail. Sur un type avec un empaquetage `qps` actif et sans pool RT : sûre, renvoie `QuantizedOut`. Voir [pool d’emplacements RT](/guides/memory/#rt-slot-pool).
`serialize_quantized_to` | `qps` actif. Vérifie les valeurs et la capacité, puis exécute le noyau d’empaquetage.
`quant_write_prepared`, `quant_read_padded` | `qps` actif. Méthodes de `qsl::payload_quant::QuantizedRecord`. `unsafe` : l’appelant garantit les plages, le remplissage, ainsi qu’une mémoire alignée et sans chevauchement.
`serialize_qps_to`, `deserialize_qps_into`, `deserialize_qps` | Adaptateurs `ByteQps`. Uniquement pour les types avec `[safety]` et `qps`, et sans chaînes ni tableaux.
`serialize_cached` | `cache=N` ou `loyal_cache`. Écrit dans un `CachedFrame`.
`deserialize_transmutate_copy`, `deserialize_overlay` | [`transmutate`](/reference/attributes/#transmutate). Chacune a une variante `_unchecked` qui saute les vérifications.
## Constantes
Constante | Signification
---|---
`SCHEMA_HASH_STRICT: u128` | Hash incluant les noms. Voir [identité du schéma](/guides/schema-identity/).
`SCHEMA_HASH_LIGHT: u128` | Hash sans les noms.
`SCHEMA_HASHES` | Les deux, avec l’algorithme, la version et la complétude. Aussi accessible via `<T as qsl::SchemaIdentity>::SCHEMA_HASHES`.
`MAX_DATA_SIZE` | Le budget utilisé par les méthodes vérifiées.
`NATIVE_BYTES`, `NATIVE_ALIGNMENT` | `size_of` et `align_of` de la structure générée.
`WIRE_MAX_BYTES: Option<usize>` | La taille de trame maximale, ou `None` si elle n’est pas bornée.
`CACHE_BYTES` | Capacité pour `serialize_cached`. Uniquement avec `cache=N` ou `loyal_cache`.
`NATIVE_WIRE_COMPATIBLE` | Indique si la disposition native est identique à la disposition dans la trame. Uniquement avec `transmutate`.
`QUANT_PADDING_BYTES` | Octets lisibles supplémentaires requis par les lectures `qps` préparées. Actuellement `0`.
`REUSES_STORAGE` est une constante du trait `Serializable` : `<T as qsl::Serializable>::REUSES_STORAGE` vaut `true` quand `deserialize_reuse` met à jour les tampons sur place.

Le générateur choisit la sortie selon la taille maximale démontrée de la trame. Jusqu’à STACK_LIMIT inclus, il utilise `StackOut<N>`; une borne supérieure ou inconnue sélectionne alloc::`Vec<u8>`. Les en-têtes transmis comptent, les métadonnées de l’objet de sortie non.
