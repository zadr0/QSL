---
title: DEQ-Konfiguration
description: DEQ-Dateien enthalten typisierte Konstanten und Einstellungen für den ganzen Build, etwa MAX_DATA_SIZE, STACK_LIMIT und den RT-Pool.
---

Eine `.deq`-Datei enthält zwei Arten von Direktiven: typisierte Konstanten, die Schemas in Grenzen verwenden können, und Einstellungen für die Codegenerierung. DEQ-Dateien übergibst du als zweites Argument von `build_qsl_files`:

```rust title="build.rs"
qsl::build_qsl_files(&["schemas/packet.qsl"], &["schemas/config.deq"], out)?;
```

## Konstanten

```deq title="config.deq"
!define MAX_NAME: u16 = 100;
```

Der Typ ist einer von `u8` bis `u64`, `i8` bis `i64`, `usize` oder `isize`, und der Wert muss hineinpassen. Ausdrücke verwenden dieselben Operatoren wie Schemas, einschließlich `sizeof(T)` für einen Integer-Typ. Eine spätere Konstante kann sich mit `!NAME` auf eine frühere beziehen:

```deq title="config.deq"
!define HEADER: usize = sizeof(u64) + 2;
!define FRAME: usize = !HEADER * 4;
```

Schemas verwenden die Konstante über ihren Namen:

```qsl
&User[safety] { u64 id; str(1..MAX_NAME) name; }
```

## Einstellungen

```deq title="config.deq"
!config MAX_DATA_SIZE = 65535;
!config USE_HASH = xxHash;
```
Einstellung | Standard | Bedeutung
---|---|---
`MAX_DATA_SIZE` | `65535` | Obergrenze in Bytes für einen Frame, für wiederhergestellte komprimierte Daten und für die Gesamtgröße der dekodierten Werte. Von 1 bis `isize::MAX`.
`STACK_LIMIT` | `256` | Budget für die Inline-Kapazität in Bytes pro String- oder Array-Puffer. Bei größeren Grenzen weicht QSL auf `String` bzw. `Vec` aus. Von `0` bis `u32::MAX`.
`USE_HASH` | `xxHash` | Algorithmus für den Schema-Hash. `xxHash` ist XXH3-64, `xxHash128` ist XXH3-128. `XXH3_64` und `XXH3_128` werden ebenfalls akzeptiert.
`QSL_RT` | `false` | `true` generiert einen statischen Slot-Pool für Typen mit endlicher maximaler Frame-Größe. Siehe [Puffer und Wiederverwendung](/guides/memory/#rt-slot-pool).
`QSL_RT_DEFAULT` | `16` | Slots pro Typ, 1 bis 64. Zählt Slots, nicht Bytes.
Einstellungen gelten für den ganzen Build. Denselben Namen zweimal zu setzen ist ein Fehler, auch in verschiedenen Dateien. Unbekannte Direktiven und unbekannte Einstellungsnamen sind ebenfalls Fehler.

DEQ-Dateien enthalten keine Dateipfade. Welche Dateien kompiliert werden, entscheidet `build.rs`.
