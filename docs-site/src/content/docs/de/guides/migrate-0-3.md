---
title: Umstieg auf 0.3.0
description: Änderungen in qsl 0.3.0 – die Abspaltung von qsl-core und die kleinere Auswahl an String-Darstellungen.
qslResources: [{"label": "Hinweise zu 0.2.0", "url": "https://github.com/zadr0/QSL/blob/main/docs/migration-0.2.0.en.md"}]
---

QSL 0.3.0 verschiebt die gemeinsame Runtime in ein neues Crate, `qsl-core` 0.1.0, und entfernt alte String-APIs. Das Wire-Format der Bitmap (v2) und gewöhnlicher Strings bleibt gleich.

## Versionen gemeinsam aktualisieren

```toml title="Cargo.toml"
[dependencies]
qsl = "=0.4.0"

[build-dependencies]
qsl = "=0.4.0"
```

Wenn du direkt von den inneren Crates abhängst, aktualisiere auch `qsl-lexer` und `qsl-interpreter` auf 0.3.0. Die Attributmakros bleiben in `qsl-macros-impl` 0.1.0.

## Aufteilung der Crates
Crate | Enthält | Abhängigkeiten
---|---|---
`qsl-core` | Fehler, Schema-Modell, gemeinsame Analyse, I/O, UTF-8-Container, Validierung, Puffer, Low-Level-Codecs | Kein anderes QSL-Crate
`qsl-lexer` | Lexer, QSL- und DEQ-Parser, Laden von Dateien, Template-Expansion, Schema-Builder | `qsl-core`
`qsl-interpreter` | Rust-Codegenerierung | `qsl-lexer`
`qsl` | Fassade, Makros, kompatible Einstiegspunkte | Alle oben genannten und `qsl-macros-impl`
Alte Modulpfade funktionieren weiterhin als Re-Exports. `QslError` und `QslCustom` sind jeweils ein einziger Typ, egal über welchen Pfad du sie importierst.

## Strings

Vier Darstellungen bleiben:
Auswahl | Rust-Typ
---|---
Gewöhnlicher String | `String`
`aggressive`, Obergrenze innerhalb von `STACK_LIMIT` | `StackString<N>`
`decompress=true`, Grenze innerhalb von `STACK_LIMIT` | `InlineString<N>`
`lifetime` | `&str`
Entfernt: `binary_opt=guard`, `binary_opt=true`, `Utf8Guard`, `Utf8CopyGuard` und der Byte-String-Modus. Schemas, die sie verwenden, lassen sich nicht mehr bauen; stelle jedes betroffene Feld auf eine der vier Darstellungen oben um.

*New* und *reuse* sind eine Wahl der Methode, keine Typen. Bevor du in einen vorhandenen `String` dekodierst, gib ihm genug Kapazität. Siehe [Strings](/language/strings/) und [Puffer und Wiederverwendung](/guides/memory/#reading-into-existing-memory).

## Trait-Methoden sind unsafe

`Serializable::serialize` und `BorrowSerializable::serialize` sind jetzt `unsafe`: Der Aufrufer garantiert, dass der Wert für das Schema gültig ist. Verwende stattdessen `serialize_checked`. Das generierte `serialize()` von Typen mit RT-Pool oder aktiver `qps`-Packung ist eine andere Methode. Sie bleibt sicher und gibt ein `Result` zurück.

## Von 0.1 auf 0.2

Wenn du von 0.1 kommst, lies zuerst die Hinweise zu 0.2.0. Die wichtigsten Punkte: Das Bitmap-Format hat sich geändert (v2), das Cargo-Feature `native` sowie die Windows-Sockets und IPC-Regionen wurden entfernt, und `serialize` gibt jetzt `Vec<u8>` zurück.
