---
title: Attributs
description: "Tous les attributs de type et modificateurs de champ de QSL, ce qu’ils génèrent et s’ils modifient le format de trame."
---

Les attributs de type se placent entre crochets après le nom : `&Packet[safety,compact]`. Les modificateurs de champ se placent entre barres verticales après le nom du champ : `str(1..100) name;`. Tous sont facultatifs, et chacun ne peut apparaître qu’une fois.

La colonne **Trame** indique si l’attribut modifie les octets, et donc le [hash du schéma](/guides/schema-identity/). Si c’est le cas, les deux pairs doivent utiliser le même réglage.

## Récapitulatif
Attribut | Portée | Trame | Effet
---|---|---|---
[`safety`](#safety) | type | non | Génère les méthodes de décodage vérifié.
[`compact`](#compact) | type | oui | Stocke les valeurs bornées sur le nombre minimal de bits.
[`alignment`](#alignment) | type | avec `compact` | Fait commencer chaque champ compact sur une limite d’octet.
[`optionals=bitmap`](#optionalsbitmap) | type | oui | Masque de présence sur les champs optionnels.
[`predict_seq`](#predict_seq) | type | oui | Présence déduite de la longueur de la charge utile.
[`opt_table`](#opt_table) | type | `legacy` seulement | Table statique de lecteurs de champs optionnels.
[`qps`, `quant`](#qps-quant) | type | oui | Empaquetage réversible des champs numériques.
[`aggressive`](#aggressive) | type | non | Stockage intégré pour les chaînes et les tableaux bornés.
[`lifetime`](#lifetime) | type | non | Emprunte les chaînes et les octets à l’entrée.
[`transmutate`](#transmutate) | type | non | Disposition native identique à la disposition dans la trame.
[`inline`](#inline) | type | non | Politique `#[inline]` des méthodes générées.
[`slots=N`](#slotsn) | type | non | Nombre d’emplacements RT pour ce type.
[`cache=N`, `loyal_cache`](#cachen-loyal_cache) | type | non | Sortie dans un `CachedFrame` fourni par l’appelant.
[`state_bits=Qn`](#state_bitsqn) | type | oui | Ancien format d’index de présence.
[`decompress=true`](#decompresstrue) | champ | oui | Compression LZ bornée pour une chaîne.
[`default=N`](#defaultn) | champ | non | Génère `impl Default`.
## Attributs de type

### safety {#safety}

Génère `deserialize_checked` et `deserialize_checked_into`, qui décodent des entrées non fiables. Ces méthodes vérifient les bornes, les plages, l’UTF-8, les métadonnées des champs optionnels, le remplissage, la décompression et le budget total `MAX_DATA_SIZE` avant de renvoyer une valeur. Les méthodes non vérifiées restent disponibles. `serialize_checked`, `serialize_checked_to`, `validate` et `validate_values` sont générées avec ou sans `safety`. Voir [valider les entrées](/guides/safety/).

### compact {#compact}

Stocke chaque entier borné sous la forme `x - min` sur `bit_length(max - min)` bits, empaquetés par-delà les limites d’octets. `u8(0..7)`, `u8(0..31)` et `u16(0..1023)` occupent 3, 5 et 10 bits. Les longueurs de chaînes et de tableaux sont stockées de la même façon, dans leurs bornes. Fonctionne avec `opt_table` et `optionals=bitmap`. Ne se combine pas avec `predict_seq`, `state_bits` ni `qps`. Voir [réduire la taille des trames](/guides/smaller-frames/).

### alignment {#alignment}

Avec `compact`, fait commencer chaque champ sur une limite d’octet. La trame peut grossir. Sans `compact`, ne change pas le format. Il ne s’agit pas de l’alignement en mémoire des tampons. Ne se combine pas avec `qps`.

### optionals=bitmap {#optionalsbitmap}

Écrit un masque avec un bit par champ optionnel, de 0 à 16 octets selon leur nombre. Prend en charge jusqu’à 128 champs optionnels. Incompatible avec le masque utilisé avant la 0.2.0. Ne se combine pas avec `state_bits`, `opt_table=legacy` ni `qps`. Voir [champs optionnels](/language/optionals/#bitmap).

### predict_seq {#predict_seq}

Déduit de la longueur de la charge utile quels champs optionnels sont présents. Exige que chaque combinaison de champs présents donne une longueur distincte. Ignoré quand `optionals=bitmap` est présent. Ne se combine pas avec `compact`. Voir [champs optionnels](/language/optionals/#length-index-predict_seq).

### opt_table {#opt_table}

`opt_table` (ou `opt_table=static`) aiguille vers les lecteurs de champs optionnels via une table statique au lieu d’un `match`. Mêmes octets, même hash. Combiné à `predict_seq`, il sélectionne le format à graphe de longueurs que produisait `[opt_table]` avant la 0.2.0.

`opt_table=legacy` est un autre format. Après les champs obligatoires, il écrit un octet de comptage et un `u32` little-endian par champ optionnel : `0` pour un champ absent, sinon l’offset de la valeur dans la charge utile optionnelle, plus 1. Il ne se combine pas avec la forme simple `opt_table`, `predict_seq`, `state_bits`, `optionals=bitmap` ni `qps`.

### qps, quant {#qps-quant}

`qps` signifie `qps=Q16`. `qps=Q2`, `Q4`, `Q8` et `Q16` fixent le quantum. `quant=Qn` en est un alias. `quant` sans valeur est une erreur, tout comme l’emploi des deux noms. N’empaquette que les champs numériques : les chaînes, les tableaux et leurs longueurs gardent leurs codecs, et les champs numériques doivent les précéder. Ne se combine pas avec `compact`, `alignment`, `optionals=bitmap` ni `opt_table=legacy`.

Sans pool RT, un type dont l’empaquetage est actif reçoit un `serialize()` sûr qui renvoie `QslResult<QuantizedOut>`. Voir [réduire la taille des trames](/guides/smaller-frames/).

### aggressive {#aggressive}

Modifie le stockage Rust, pas les octets ni les entrées acceptées :

- `str(min..max)` devient `StackString<max>` et `array<T>(min..max)` devient `InlineArray<T, max>` quand le maximum tient dans `STACK_LIMIT`. Les champs plus grands restent des `String` ou des `Vec`.
- Un champ numérique optionnel est stocké sous forme d’entier `NonZero` de la plus petite largeur capable de contenir sa plage, si cette largeur ne dépasse pas celle du type du champ.

### lifetime {#lifetime}

Donne au type une durée de vie `'a` et emprunte à l’entrée : les chaînes deviennent `&'a str` et `array<u8>` devient `&'a [u8]`. L’entrée doit vivre plus longtemps que la valeur. Les tableaux d’autres types d’éléments et les chaînes compressées ne peuvent pas être empruntés et font échouer la compilation, tout comme un type `lifetime` qui n’a rien à emprunter.

### transmutate {#transmutate}

Rend la disposition native du type identique à sa disposition dans la trame. Chaque champ doit avoir une taille fixe : `compact`, `qps`, les champs optionnels, et les chaînes ou tableaux à plage provoquent des erreurs de compilation. La taille dans la trame doit être égale à la taille de la structure : ajoutez donc des champs `padding(N)` là où Rust insère du remplissage.

Sans `lifetime`, `deserialize_transmutate_copy` copie la trame d’un seul bloc dans une valeur possédée quand `NATIVE_WIRE_COMPATIBLE` est vrai, et décode champ par champ sinon. Avec `[safety]`, la méthode est sûre et valide le résultat. Sans `[safety]`, elle est `unsafe`.

Avec `lifetime`, `deserialize_overlay` renvoie un `&Self` qui pointe dans l’entrée. La compilation vérifie que chaque offset dans la trame est égal à l’offset natif, et échoue sur les cibles big-endian. Avec `[safety]`, `deserialize_overlay` vérifie `NATIVE_WIRE_COMPATIBLE`, la longueur et l’alignement de l’entrée, puis les valeurs, et `deserialize_checked` fait de même. Sans `[safety]`, `deserialize_overlay` est `unsafe` et ne vérifie rien.

Les deux ont des variantes `_unchecked` qui sautent les vérifications.

### inline {#inline}

`inline` (équivalent à `inline=hint`), `inline=always` ou `inline=never` place l’attribut correspondant sur les méthodes générées. Sans lui, les méthodes générées ne portent aucun attribut inline et le compilateur décide. `always` peut augmenter la taille du code ; ce n’est pas une accélération garantie.

### slots=N {#slotsn}

Remplace le nombre d’emplacements RT pour ce type, de 1 à 64. N’a d’effet qu’avec `!config QSL_RT = true;`. Voir [pool d’emplacements RT](/guides/memory/#rt-slot-pool).

### cache=N, loyal_cache {#cachen-loyal_cache}

Génère `CACHE_BYTES` et `serialize_cached`, qui écrit dans un `qsl::modernization::CachedFrame<N>` fourni par l’appelant. `loyal_cache` fixe `CACHE_BYTES` à la taille de trame maximale. Le type doit avoir une taille de trame maximale finie, et un `N` plus petit fait échouer la compilation. Cela prépare la mémoire de sortie ; cela ne met pas les valeurs en cache. Voir [trames de sortie en cache](/guides/memory/#cached-output-frames).

### state_bits=Qn {#state_bitsqn}

`Q1` (par défaut), `Q2`, `Q4`, `Q8` ou `Q16`. Sélectionne un ancien format d’index de présence. Ce n’est pas un mode de compression. Ne se combine pas avec `optionals=bitmap`, `compact` ni `opt_table=legacy`.

## Modificateurs de champ

### binary_opt {#binary_opt}

binary_opt est entièrement supprimé. Utilisez [lifetime] pour &str emprunté, [aggressive] pour StackString borné et decompress=true pour InlineString. STACK_LIMIT concerne toutes les chaînes et tous les tableaux.

### decompress=true {#decompresstrue}

Encode la chaîne avec le codec LZ borné de QSL. Les chaînes de moins de 32 octets, et celles que la compression ne raccourcirait pas, sont stockées brutes. Chaque jeton décodé est vérifié par rapport à la taille restaurée, à la capacité de destination et à `MAX_DATA_SIZE`. La chaîne a besoin d’une borne supérieure d’au plus 65535 octets. Une chaîne à plage devient `InlineString<max>`, ou `String` au-delà de `STACK_LIMIT`. Ne se combine pas avec `lifetime` ni `lifetime`. Voir [chaînes compressées](/language/strings/#compressed-strings).

### default=N {#defaultn}

Génère `impl Default` pour le type. Ce champ vaut initialement `N`, ou `Some(N)` s’il est optionnel. Les autres champs prennent leur plus petite valeur valide. `N` doit être dans la plage du champ, et le champ doit être numérique. Un type qui a aussi un champ de type personnalisé fait échouer la compilation. Cela ne change ni les octets ni le hash.
