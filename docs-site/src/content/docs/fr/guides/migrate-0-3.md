---
title: Migrer vers la 0.3.0
description: "Les changements de qsl 0.3.0 : la séparation de qsl-core et l’ensemble réduit de représentations des chaînes."
qslResources: [{"label": "notes de la 0.2.0", "url": "https://github.com/zadr0/QSL/blob/main/docs/migration-0.2.0.en.md"}]
---

QSL 0.3.0 déplace le runtime partagé dans une nouvelle crate, `qsl-core` 0.1.0, et supprime d’anciennes API de chaînes. Le format de trame du bitmap (v2) et des chaînes ordinaires ne change pas.

## Mettre à jour les versions ensemble

```toml title="Cargo.toml"
[dependencies]
qsl = "=0.4.0"

[build-dependencies]
qsl = "=0.4.0"
```

Si vous dépendez directement des crates internes, mettez aussi `qsl-lexer` et `qsl-interpreter` à jour en 0.3.0. Les macros d’attribut restent dans `qsl-macros-impl` 0.1.0.

## Découpage des crates
Crate | Contenu | Dépend de
---|---|---
`qsl-core` | Erreurs, modèle de schéma, analyse partagée, E/S, conteneurs UTF-8, validation, tampons, codecs de bas niveau | Aucune autre crate QSL
`qsl-lexer` | Lexer, parseurs QSL et DEQ, chargement des fichiers, expansion des modèles, builder de schémas | `qsl-core`
`qsl-interpreter` | Génération de code Rust | `qsl-lexer`
`qsl` | Façade, macros, points d’entrée compatibles | Toutes les précédentes et `qsl-macros-impl`
Les anciens chemins de modules fonctionnent toujours, sous forme de réexportations. `QslError` et `QslCustom` restent chacun un seul et même type, quel que soit le chemin par lequel vous les importez.

## Chaînes

Il reste quatre représentations :
Choix | Type Rust
---|---
Chaîne ordinaire | `String`
`aggressive`, borne supérieure ne dépassant pas `STACK_LIMIT` | `StackString<N>`
`decompress=true`, borne ne dépassant pas `STACK_LIMIT` | `InlineString<N>`
`lifetime` | `&str`
Supprimés : `binary_opt=guard`, `binary_opt=true`, `Utf8Guard`, `Utf8CopyGuard` et le mode chaîne d’octets. Les schémas qui les utilisent ne compilent plus ; passez chaque champ à l’une des quatre représentations ci-dessus.

*New* et *reuse* sont des choix de méthode, pas des types. Avant de décoder dans un `String` existant, donnez-lui une capacité suffisante. Voir [chaînes](/language/strings/) et [tampons et réutilisation](/guides/memory/#reading-into-existing-memory).

## Les méthodes de trait deviennent unsafe

`Serializable::serialize` et `BorrowSerializable::serialize` sont désormais `unsafe` : l’appelant garantit que la valeur est valide pour le schéma. Utilisez plutôt `serialize_checked`. Le `serialize()` généré pour les types avec un pool RT ou un empaquetage `qps` actif est une autre méthode. Elle reste sûre et renvoie un `Result`.

## De la 0.1 à la 0.2

Si vous venez de la 0.1, lisez d’abord les notes de la 0.2.0. Points principaux : le format du bitmap a changé (v2), la feature Cargo `native` ainsi que les régions Windows de sockets et d’IPC ont été supprimées, et `serialize` renvoie désormais `Vec<u8>`.
