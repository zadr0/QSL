---
title: Deklarationen
description: Syntax eines QSL-Schemas – Deklarationen, Felder, Attribute, Modifikatoren, Wertebereiche, konstante Ausdrücke und Kommentare.
---

Eine Schemadatei enthält eine oder mehrere Deklarationen. Aus jeder Deklaration wird ein Rust-Typ.

```qsl
&Name[attribute, attribute=value] {
    type field;
    type field|modifier=value|;
    type? optional_field;
}
```
Teil | Bedeutung
---|---
`&Name` | Deklariert einen Typ namens `Name`.
`[ ... ]` | Attribute für den ganzen Typ. Optional. Siehe die [Attributreferenz](/reference/attributes/).
`type field;` | Ein Pflichtfeld.
`type? field;` | Ein optionales Feld. Mit Wertebereich steht das `?` davor: `u16?(0..1023) level;`.
`\|...\|` | Modifikatoren für ein einzelnes Feld, zwischen Name und Semikolon.
## Optionale Felder in Rust {#optional-fields-in-rust}

Ein optionales String- oder Array-Feld ist ein öffentliches Feld vom Typ `Option<T>` im Struct.

Ein optionales numerisches Feld liegt in einem privaten Feld. Du liest es mit einer gleichnamigen Methode und änderst es mit `set_<name>`; für einen Wert außerhalb des Wertebereichs gibt diese Methode `InvalidValue` zurück. Typen mit einem solchen Feld bekommen einen Konstruktor `try_new`, der alle Felder in Deklarationsreihenfolge entgegennimmt und die Werte prüft:

```qsl
&Reading[safety] { u32 id; u8? level; }
```

```rust
let mut r = Reading::try_new(42, Some(7))?;
assert_eq!(r.level(), Some(7));
r.set_level(None)?;
```

## Feldreihenfolge

Pflichtfelder müssen vor optionalen Feldern stehen. QSL erzwingt das für jeden Typ, mit oder ohne Attribute.

```qsl
&Ok    { u32 id; u16? code; }
&Error { u16? code; u32 id; }   // Build-Fehler: Pflichtfeld nach optionalem Feld
```

Pflichtwerte landen in Deklarationsreihenfolge im Frame. Die Reihenfolge der optionalen Werte hängt vom [Modus für optionale Felder](/language/optionals/) ab.

## Tags

Der Compiler vergibt jedem Feld in Deklarationsreihenfolge ein Tag, beginnend bei 1. Selbst setzen kannst du Tags nicht. Die Standardkodierung optionaler Felder speichert Tags (siehe [Optionale Felder](/language/optionals/)), deshalb ändert ein Umsortieren der Felder das Format.

Ein Typ ohne `optionals=bitmap` kann höchstens 64 Felder haben.

## Wertebereiche

`A..B` ist ein inklusiver Bereich. Was er bedeutet, hängt vom Typ ab:

```qsl
&Limits {
    u8(1..3)          kind;    // Wertebereich: 1, 2 oder 3
    str(1..100)       name;    // 1 bis 100 Bytes UTF-8
    str(4)            tag;     // genau 4 Bytes, wird zu [u8; 4]
    array<u16>(0..16) samples; // 0 bis 16 Elemente
}
```

Die geprüften Methoden erzwingen einen Wertebereich. Mit `compact` bestimmt er außerdem, wie viele Bits das Feld belegt. Ein String-Bereich zählt UTF-8-Bytes, keine Zeichen: `"Ада"` hat 6 Bytes.

## Konstante Ausdrücke

Grenzen können Ausdrücke aus Integern und benannten Konstanten sein:

```qsl
&Frame {
    str(1..MAX_NAME) name;
    array<u8>(0..MAX_NAME * 2) payload;
}
```

Ausdrücke unterstützen `+ - * / %`, unäres Minus, Klammern, `sizeof(T)` für die Integer-Typen `u8` bis `i64` sowie benannte Konstanten. Eine Konstante stammt aus `!define` in einer [DEQ-Datei](/language/deq/) oder aus einer Zeile `#NAME = expression;` im Rumpf eines Typs, die sie für die nachfolgenden Zeilen definiert. Überlauf und Division durch null sind Build-Fehler.

## Kommentare

`//` leitet einen Zeilenkommentar ein. `/* ... */` ist ein Blockkommentar.

```qsl
&Packet[safety] {
    u8 id;      // message kind
    u16? a;     /* absent in most frames */
}
```

## Was abgelehnt wird

Der Build schlägt fehl bei:

- einem Pflichtfeld nach einem optionalen Feld;
- doppelten Feld- oder Typnamen sowie Feldnamen, die Rust-Schlüsselwörter sind oder mit `__qsl_` beginnen;
- ungültigen oder vertauschten Grenzen sowie Bereichen außerhalb des Integer-Typs;
- unbekannten oder wiederholten Attributen und Modifikatoren;
- mehr als 64 Feldern ohne `optionals=bitmap` oder mehr als 128 optionalen Feldern mit `optionals=bitmap`.

Ein Typname, der nicht eingebaut ist, gilt als [eigener Typ](/guides/custom-types/) und braucht eine Rust-Implementierung. Ein eigener Typ kann keinen Wertebereich haben. Die eingebauten Typen findest du unter [Typen](/language/types/).
