---
title: Erreurs
description: "Les variantes de QslError et les cas où chacune est renvoyée."
---

Les méthodes générées renvoient `qsl::QslResult<T>`, un alias de `Result<T, QslError>`. `QslError` a 11 variantes et implémente `std::error::Error`. Sa sortie `Display` est `QSL error: <Variant>`, par exemple `QSL error: InvalidUtf8`.
Variante | Renvoyée quand
---|---
`UnexpectedEof` | L’entrée se termine avant la fin de la trame.
`BufferTooSmall` | Un tampon que vous avez passé est trop petit pour la sortie.
`LengthOverflow` | Un calcul de longueur provoquerait un dépassement de capacité.
`InvalidWidth` | Une largeur ou un quantum n’est pas pris en charge.
`InvalidValue` | Une valeur, une longueur ou un champ de métadonnées sort de son domaine autorisé.
`InvalidUtf8` | Les octets d’une chaîne ne sont pas de l’UTF-8 valide.
`InvalidSchema` | Le schéma est invalide, ou l’opération ne s’y applique pas. C’est aussi le résultat par défaut des validateurs de `QslCustom`.
`InvalidPadding` | Des bits ou des octets de remplissage ne sont pas à zéro.
`PoolExhausted` | Tous les emplacements RT sont pris. Uniquement depuis `try_acquire`, qui n’attend pas.
`SizeLimitExceeded` | La trame ou les données décodées dépassent `MAX_DATA_SIZE`.
`AllocationFailed` | Une allocation faillible a échoué.
Les opérations sur les fichiers et les sockets peuvent aussi renvoyer `std::io::Error`.

Une validation réussie signifie que les octets correspondent à la forme du schéma. Elle n’indique ni qui les a envoyés, ni avec quel schéma ils ont été écrits.
