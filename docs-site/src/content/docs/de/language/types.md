---
title: Typen
description: Die eingebauten QSL-Typen, die Rust-Typen, die daraus entstehen, und ihre Standardkodierung im Frame.
---

## Eingebaute Typen
Schema | Rust-Typ | Hinweise
---|---|---
`u8` `u16` `u32` `u64` `i8` `i16` `i32` `i64` | Derselbe primitive Typ |
`T(A..B)` für einen Integer-Typ `T` | Derselbe primitive Typ | Der Wert muss im Bereich liegen. `compact` und `qps` können ihn im Frame verkleinern.
`flags(N)` | `u8`, `u16`, `u32` oder `u64`, abhängig von `N` | `N` ist 1 bis 64. Bits oberhalb von `N` müssen null sein.
`str`, `string`, `String` | `String` | Variable Länge, nur durch `MAX_DATA_SIZE` begrenzt.
`str(A..B)` | `String` oder ein anderer String-Typ | Inklusive Grenzen in UTF-8-Bytes. Siehe [Strings](/language/strings/).
`str(N)` | `[u8; N]` | Genau `N` Bytes UTF-8, keine Länge im Frame. `String`, wenn `N` über `STACK_LIMIT` liegt.
`array<T>`, `vec<T>` | `Vec<T>` | `T` ist ein eingebauter Integer-Typ oder ein [registrierter Primitivtyp](/guides/custom-types/#validated-scalar-wrappers).
`array<T>(N)` | `[T; N]` | Genau `N` Elemente, keine Anzahl im Frame. `Vec<T>`, wenn das Array größer als `STACK_LIMIT` Bytes ist.
`array<T>(A..B)` | `Vec<T>` | Inklusive Elementanzahl. `InlineArray<T, B>` mit `aggressive`, wenn es in `STACK_LIMIT` passt.
`padding(N)` | `()` | `N` Null-Bytes im Frame.
`T?` | `Option<T>` | Optionales Feld. Numerische optionale Felder verwenden Zugriffsmethoden, siehe [Deklarationen](/language/declarations/#optional-fields-in-rust).
Jeder andere Name | Dein Rust-Typ | Muss `QslCustom` implementieren, siehe [Eigene Typen](/guides/custom-types/).
Mit dem Attribut `lifetime` leihen Strings und `u8`-Arrays aus der Eingabe aus, als `&'a str` und `&'a [u8]`.

## Standardkodierung im Frame

Ohne Attribute, die das Format ändern:

- **Integer** stehen als Little-Endian in ihrer nativen Breite im Frame. Ein `u16(0..1023)` belegt 2 Bytes.
- **Strings** bestehen aus einer Bytelänge als `u32`, gefolgt von den UTF-8-Bytes. Ein festes `str(N)` hat keine Länge.
- **Dynamische Arrays** bestehen aus einer Elementanzahl als `u32`, gefolgt von den Elementen. Feste Arrays haben keine Anzahl.
- **Pflichtfelder** folgen der Deklarationsreihenfolge.
- **Optionale Felder** fügen vor den Werten Metadaten zum Vorhandensein hinzu. Siehe [Optionale Felder](/language/optionals/).

Der Frame enthält keine Typinformationen. Beide Seiten müssen das Schema kennen.

## Frame-Größe und Objektgröße

Beide sind voneinander unabhängig:

```qsl
&Reading[compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

Im Frame sind das 3 + 5 + 10 = 18 Bits, geschrieben als 3 Bytes. Im Speicher ist `Reading` ein `#[repr(C)]`-Struct mit einem `u8`, einem `u8` und einem `u16`: 4 Bytes. `Reading::NATIVE_BYTES` enthält diese Größe.
