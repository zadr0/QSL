---
title: Valider les entrées
description: "Ce que génère [safety], ce que contrôlent les méthodes vérifiées et ce qui reste à la charge de l’appelant."
---

Les octets qui viennent d’un socket, d’un fichier ou d’un autre processus ne sont pas fiables. Décodez-les avec les méthodes vérifiées, qui ne sont générées que pour les types dotés de l’attribut `safety`.

```qsl
&Packet[safety] {
    u32 id;
    str(1..100) text;
}
```

```rust
let bytes = packet.serialize_checked()?;
let decoded = Packet::deserialize_checked(&bytes)?;

// Ou décoder dans une mémoire que vous possédez déjà :
let mut slot = std::mem::MaybeUninit::uninit();
let decoded: &mut Packet = Packet::deserialize_checked_into(&bytes, &mut slot)?;
```

## Ce que contrôle un décodage vérifié

Avant de renvoyer une valeur, le décodeur généré vérifie que :

- la trame est au moins aussi longue que la plus petite trame valide, contient tous les champs et se termine là où le schéma l’indique ;
- chaque nombre est dans sa plage déclarée ;
- chaque longueur de chaîne et de tableau est dans ses bornes ;
- les chaînes sont de l’UTF-8 valide ;
- les métadonnées des champs optionnels sont cohérentes avec le schéma ;
- les bits et octets de remplissage sont à zéro ;
- les chaînes compressées, une fois restaurées, ne dépassent pas leur borne ;
- la taille totale des valeurs décodées reste dans la limite de `MAX_DATA_SIZE`.

La validation et le décodage se font en une seule passe. Inutile d’appeler `validate` avant. Si le décodage échoue, `deserialize_checked_into` laisse `slot` non initialisé.

## Limites de taille

`MAX_DATA_SIZE` (65535 octets par défaut) limite la trame d’entrée et la charge utile décodée totale. Les longueurs lues dans la trame sont comparées à leurs bornes et à ce budget avant toute réservation de mémoire. Augmentez-le ou diminuez-le dans un [fichier DEQ](/language/deq/).

L’exemple `safety` du dépôt passe trois trames invalides à `validate` pour ce type :

```qsl
&Bounded[safety] { u8(1..3) kind; str(1..100) name; }
```
Entrée | Résultat
---|---
`09 01 00 00 00 61` | `InvalidValue` : `kind` vaut 9, hors de `1..3`.
`01 ff ff ff ff` | `UnexpectedEof` : 5 octets, c’est moins que la plus petite trame valide, qui fait 6 octets.
`01 01 00 00 00 ff` | `InvalidUtf8` : `ff` n’est pas de l’UTF-8.
Une longueur falsifiée dans une trame de taille valide, comme `01 ff ff ff ff 61`, échoue avec `InvalidValue`, car la longueur est comparée à la borne `100` avant la copie du moindre octet.

## Encodage vérifié

`serialize_checked` vérifie la valeur par rapport au schéma (plages, longueurs, UTF-8 des chaînes de taille fixe, `MAX_DATA_SIZE`) et renvoie un nouveau `Vec<u8>`. `serialize_checked_to` écrit dans une slice que vous fournissez et renvoie le nombre d’octets écrits, ou `BufferTooSmall`. Si la vérification finale de la taille échoue, la slice peut déjà être partiellement écrite. Les deux méthodes sont générées pour tous les types, avec ou sans `[safety]`.

## Méthodes non vérifiées

`[safety]` ajoute des méthodes vérifiées. Il ne retire pas les méthodes non vérifiées. Les méthodes du trait `qsl::Serializable`, comme `deserialize_into`, et les lecteurs à pointeur brut supposent que :

- la trame est complète et valide pour ce schéma ;
- la mémoire de destination est assez grande, correctement alignée et ne chevauche pas l’entrée ;
- les entrées empruntées vivent plus longtemps que le résultat.

Ne les utilisez que pour des trames que vous avez produites vous-même ou déjà validées. Elles sont `unsafe`, y compris `Serializable::serialize` et `BorrowSerializable::serialize` depuis la 0.3.0. Le `serialize()` généré pour les types avec un pool RT ou un empaquetage `qps` actif est une autre méthode, sûre, qui renvoie un `Result`.

## Limites de la validation

- La validation prouve que des octets ont une forme valide pour le schéma. Elle ne prouve pas quel schéma les a produits : deux schémas de même disposition acceptent les mêmes octets. Utilisez les [hashes de schéma](/guides/schema-identity/) ou votre propre discriminant dans une poignée de main.
- Elle n’authentifie pas l’émetteur.
- Un [type personnalisé](/guides/custom-types/) n’est sûr que dans la mesure où son implémentation de `qsl_validate` l’est. L’implémentation par défaut rejette toute entrée.

Le générateur choisit la sortie selon la taille maximale démontrée de la trame. Jusqu’à STACK_LIMIT inclus, il utilise `StackOut<N>`; une borne supérieure ou inconnue sélectionne alloc::`Vec<u8>`. Les en-têtes transmis comptent, les métadonnées de l’objet de sortie non.
