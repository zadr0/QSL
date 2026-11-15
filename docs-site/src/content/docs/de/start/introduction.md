---
title: Was Quick Schema Language (QSL) ist
description: Quick Schema Language (QSL) ist eine Schemasprache und ein Generator für binäre Codecs in Rust. Was es tut, was es dir überlässt und wie die Crates aufgeteilt sind.
---

Quick Schema Language (QSL) ist eine Sprache zur Beschreibung binärer Nachrichten und ein Codegenerator, der aus diesen Beschreibungen Rust-Code macht. Du schreibst ein Schema mit Feldtypen, Wertebereichen und Längengrenzen. Während `cargo build` liest QSL das Schema und erzeugt Structs und Methoden, die Bytes lesen und schreiben. Zur Laufzeit parst das Programm kein Schema.

Binäre Serialisierung wandelt einen Wert in Bytes um; Deserialisierung liest daraus wieder einen Wert. Bei QSL definiert das Schema die Felder, und der generierte Rust-Code übernimmt beide Richtungen. Die Kommunikationspartner benötigen übereinstimmende Schemas und Attribute. Beginne mit dem [Schnellstart zur binären Serialisierung in Rust](/start/quickstart/) und lies danach über das [Drahtformat](/internals/wire-format/) und die [Beispiele zum Bit-Packing](/guides/smaller-frames/).

```qsl
&Reading[safety,compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

Aus diesem Schema generiert QSL ein Struct `Reading` mit Feldern vom Typ `u8`, `u8` und `u16` sowie Methoden, die es kodieren, dekodieren und validieren. Mit `compact` belegen die drei Werte 3 + 5 + 10 = 18 Bits, ein Frame ist also 3 statt 4 Bytes groß.

## Was du steuerst

- **Wire-Layout.** Wie breit Integer im Frame sind und wie der Frame markiert, welche optionalen Felder vorhanden sind.
- **Speicher.** Ob ein String zu `String`, zu einem ausgeliehenen `&str` oder zu einem Inline-Puffer wird und ob das Dekodieren ein vorhandenes Objekt wiederverwendet.
- **Validierung.** `[safety]` generiert geprüfte Decode-Methoden. Sie prüfen Grenzen, Wertebereiche, UTF-8 und ein Budget für die Gesamtgröße, bevor sie einen Wert zurückgeben.

## Was Sache deiner Anwendung bleibt

QSL legt fest, wie Daten dargestellt werden. Abgesehen von `qsl::RawSocket`, einem dünnen Wrapper, der Frames mit Längenpräfix über TCP sendet, kümmert es sich nicht um den Transport, und es bietet weder TLS noch Routing noch das Aushandeln von Schemaversionen. Beide Seiten brauchen dasselbe Schema mit denselben Attributen. Wenn du das Format änderst, rolle die Änderung auf allen Gegenstellen gleichzeitig aus. QSL erkennt nicht, in welchem Format ein Frame vorliegt.

Jeder generierte Typ trägt seine Schema-Hashes als Konstanten (siehe [Schema-Identität](/guides/schema-identity/)). Du kannst sie in einem eigenen Handshake austauschen. QSL nimmt dir das nicht ab.

## Crates
Crate | Version | Aufgabe
---|---|---
`qsl` | 0.3.0 | Das Crate, von dem du abhängst. Re-exportiert die Runtime und stellt `build_qsl_files` und die Attributmakros bereit.
`qsl-core` | 0.1.0 | Gemeinsame Runtime: Fehler, I/O-Cursor, String-Container, Validierung, Puffer.
`qsl-lexer` | 0.3.0 | Parser für `.qsl`-Schemas und `.deq`-Konfigurationsdateien, Schema-Builder.
`qsl-interpreter` | 0.3.0 | Generiert Rust-Quellcode aus validierten Schemas.
`qsl-macros-impl` | 0.1.0 | Die Attribute `qsl_macro` und `schem_macro`, re-exportiert von `qsl`.
Die meisten Projekte binden nur `qsl` ein, einmal unter `[dependencies]` und einmal unter `[build-dependencies]`. Wie die Crates voneinander abhängen, zeigt [Architektur](/internals/architecture/).

## Stand

QSL 0.3.0 wird unter Windows 11 x86-64 mit Rust 1.98 entwickelt und getestet; das ist auch die minimal unterstützte Version. Attribute sind opt-in. Ein Attribut ändert den generierten Code, aber keines garantiert einen Geschwindigkeitsgewinn. Die Seite  listet, was unter welchen Bedingungen gemessen wurde.

Weiter: [Schnellstart](/start/quickstart/).
