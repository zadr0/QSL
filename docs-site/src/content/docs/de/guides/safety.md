---
title: Eingaben validieren
description: Was [safety] generiert, was geprüfte Methoden prüfen und wofür der Aufrufer verantwortlich bleibt.
---

Bytes aus einem Socket, einer Datei oder einem anderen Prozess sind nicht vertrauenswürdig. Dekodiere sie mit den geprüften Methoden. Diese werden nur für Typen mit dem Attribut `safety` generiert.

```qsl
&Packet[safety] {
    u32 id;
    str(1..100) text;
}
```

```rust
let bytes = packet.serialize_checked()?;
let decoded = Packet::deserialize_checked(&bytes)?;

// Oder in Speicher dekodieren, den du schon hast:
let mut slot = std::mem::MaybeUninit::uninit();
let decoded: &mut Packet = Packet::deserialize_checked_into(&bytes, &mut slot)?;
```

## Was geprüftes Dekodieren sicherstellt

Bevor der generierte Decoder einen Wert zurückgibt, prüft er, dass:

- der Frame mindestens so lang ist wie der kleinste gültige Frame, jedes Feld enthält und dort endet, wo das Schema es vorgibt;
- jede Zahl in ihrem deklarierten Wertebereich liegt;
- jede String- und Array-Länge innerhalb ihrer Grenzen liegt;
- Strings gültiges UTF-8 sind;
- die Metadaten der optionalen Felder zum Schema passen;
- Padding-Bits und -Bytes null sind;
- komprimierte Strings beim Wiederherstellen nicht über ihre Grenze hinauswachsen;
- die Gesamtgröße der dekodierten Werte innerhalb von `MAX_DATA_SIZE` bleibt.

Validieren und Dekodieren passieren in einem Durchlauf. Du musst vorher nicht `validate` aufrufen. Schlägt das Dekodieren fehl, lässt `deserialize_checked_into` `slot` uninitialisiert.

## Größengrenzen

`MAX_DATA_SIZE` (Standard: 65535 Bytes) begrenzt den Eingabe-Frame und die gesamten dekodierten Nutzdaten. Längen aus dem Frame werden gegen ihre Grenzen und dieses Budget geprüft, bevor Speicher reserviert wird. Erhöhen oder senken kannst du den Wert in einer [DEQ-Datei](/language/deq/).

Das Beispiel safety im Repository übergibt für diesen Typ drei fehlerhafte Frames an `validate`:

```qsl
&Bounded[safety] { u8(1..3) kind; str(1..100) name; }
```
Eingabe | Ergebnis
---|---
`09 01 00 00 00 61` | `InvalidValue`: `kind` ist 9, außerhalb von `1..3`.
`01 ff ff ff ff` | `UnexpectedEof`: 5 Bytes sind kürzer als der kleinste gültige Frame mit 6 Bytes.
`01 01 00 00 00 ff` | `InvalidUtf8`: `ff` ist kein UTF-8.
Eine gefälschte Länge in einem Frame gültiger Größe, etwa `01 ff ff ff ff 61`, scheitert mit `InvalidValue`, weil die Länge mit der Grenze `100` verglichen wird, bevor irgendein Byte kopiert wird.

## Geprüftes Kodieren

`serialize_checked` prüft den Wert gegen das Schema (Wertebereiche, Längen, UTF-8 von Strings fester Größe, `MAX_DATA_SIZE`) und gibt einen neuen `Vec<u8>` zurück. `serialize_checked_to` schreibt in einen Slice, den du bereitstellst, und gibt die Anzahl geschriebener Bytes zurück oder `BufferTooSmall`. Schlägt die abschließende Größenprüfung fehl, kann der Slice schon teilweise beschrieben sein. Beide werden für jeden Typ generiert, mit oder ohne `[safety]`.

## Ungeprüfte Methoden

`[safety]` fügt geprüfte Methoden hinzu, entfernt aber die ungeprüften nicht. Die Methoden des Traits `qsl::Serializable`, etwa `deserialize_into`, und die Reader mit Rohzeigern setzen voraus, dass:

- der Frame vollständig und für dieses Schema gültig ist;
- der Zielspeicher groß genug und korrekt ausgerichtet ist und sich nicht mit der Eingabe überlappt;
- ausgeliehene Eingaben länger leben als das Ergebnis.

Verwende sie nur für Frames, die du selbst erzeugt oder bereits validiert hast. Sie sind `unsafe`, seit 0.3.0 auch `Serializable::serialize` und `BorrowSerializable::serialize`. Das generierte `serialize()` von Typen mit RT-Pool oder aktiver `qps`-Packung ist eine andere, sichere Methode, die ein `Result` zurückgibt.

## Grenzen der Validierung

- Die Validierung belegt, dass die Bytes eine für das Schema gültige Form haben. Sie belegt nicht, mit welchem Schema sie erzeugt wurden: Zwei Schemas mit gleichem Layout akzeptieren dieselben Bytes. Verwende [Schema-Hashes](/guides/schema-identity/) oder einen eigenen Diskriminator in einem Handshake.
- Sie authentifiziert den Absender nicht.
- Ein [eigener Typ](/guides/custom-types/) ist nur so sicher wie seine Implementierung von `qsl_validate`. Die Standardimplementierung lehnt jede Eingabe ab.

Der Generator wählt den Ausgabetyp anhand der bewiesenen maximalen Framegröße. Bis einschließlich STACK_LIMIT wird `StackOut<N>` verwendet; eine größere oder unbekannte Grenze wählt alloc::`Vec<u8>`. Übertragene Header zählen mit, die Metadaten des Ausgabeobjekts nicht.
