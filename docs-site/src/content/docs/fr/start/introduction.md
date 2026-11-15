---
title: "Qu’est-ce que Quick Schema Language (QSL) ?"
description: "Quick Schema Language (QSL) est un langage de schéma et un générateur de codecs binaires pour Rust. Ce qu’il fait, ce qu’il vous laisse et comment les crates sont découpées."
---

Quick Schema Language (QSL) est un langage de description de messages binaires, associé à un générateur de code qui transforme ces descriptions en Rust. Vous écrivez un schéma avec les types des champs, les plages de valeurs et les limites de longueur. Pendant `cargo build`, QSL lit le schéma et produit des structures et des méthodes qui lisent et écrivent des octets. Le programme n’analyse pas le schéma à l’exécution.

La sérialisation binaire transforme une valeur en octets ; la désérialisation reconstruit une valeur à partir de ces octets. Avec QSL, le schéma définit les champs et le code Rust généré effectue les deux conversions. Les participants doivent utiliser les mêmes schémas et attributs. Commencez par le [démarrage rapide de la sérialisation binaire en Rust](/start/quickstart/), puis consultez le [format binaire](/internals/wire-format/) et les [exemples de compactage des bits](/guides/smaller-frames/).

```qsl
&Reading[safety,compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

À partir de ce schéma, QSL génère une structure `Reading` avec des champs `u8`, `u8` et `u16`, ainsi que des méthodes pour l’encoder, la décoder et la valider. Avec `compact`, les trois valeurs occupent 3 + 5 + 10 = 18 bits : une trame fait donc 3 octets au lieu de 4.

## Ce que vous contrôlez

- **La disposition dans la trame.** La largeur des entiers dans la trame, et la manière dont la trame indique quels champs optionnels sont présents.
- **La mémoire.** Si une chaîne devient un `String`, un `&str` emprunté ou un tampon intégré, et si le décodage réutilise un objet existant.
- **La validation.** `[safety]` génère des méthodes de décodage vérifié. Elles contrôlent les bornes, les plages, l’UTF-8 et un budget de taille totale avant de renvoyer une valeur.

## Ce qui reste à la charge de votre application

QSL définit la représentation des données. En dehors de `qsl::RawSocket`, une fine surcouche qui envoie sur TCP des trames préfixées par leur longueur, il ne gère pas le transport, et il ne fournit ni TLS, ni routage, ni négociation de version du schéma. Les deux côtés d’un échange ont besoin du même schéma et des mêmes attributs. Si vous changez le format, déployez le changement sur tous les pairs à la fois. QSL ne détecte pas quel format utilise une trame.

Chaque type généré expose les hashes de son schéma sous forme de constantes (voir [identité du schéma](/guides/schema-identity/)). Vous pouvez les envoyer dans votre propre poignée de main. QSL ne le fait pas à votre place.

## Crates
Crate | Version | Rôle
---|---|---
`qsl` | 0.3.0 | La crate dont vous dépendez. Réexporte le runtime, fournit `build_qsl_files` et les macros d’attribut.
`qsl-core` | 0.1.0 | Runtime partagé : erreurs, curseurs d’E/S, conteneurs de chaînes, validation, tampons.
`qsl-lexer` | 0.3.0 | Parseur des schémas `.qsl` et des fichiers de configuration `.deq`, builder de schémas.
`qsl-interpreter` | 0.3.0 | Génère le code source Rust à partir des schémas validés.
`qsl-macros-impl` | 0.1.0 | Les attributs `qsl_macro` et `schem_macro`, réexportés par `qsl`.
La plupart des projets n’ajoutent que `qsl`, une fois dans `[dependencies]` et une fois dans `[build-dependencies]`. La page [Architecture](/internals/architecture/) montre les dépendances entre les crates.

## État du projet

QSL 0.3.0 est développé et testé sous Windows 11 x86-64 avec Rust 1.98, qui est aussi la version minimale prise en charge. Les attributs sont à activer explicitement. Un attribut modifie le code généré, mais aucun ne garantit un gain de vitesse. La  indique ce qui a été mesuré et dans quelles conditions.

Suite : [Démarrage rapide](/start/quickstart/).
