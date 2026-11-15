---
title: Rust-Templates und eigene Typen
description: Rust-Structs als Feldgruppen wiederverwenden, validierte Integer-Wrapper registrieren, QslCustom implementieren und verschachtelte Regionen deklarieren.
---

QSL bietet vier Wege, eigene Rust-Typen in ein Schema zu bringen. Sie lösen unterschiedliche Probleme.
Mechanismus | Wann du ihn verwendest
---|---
`#[qsl::qsl_macro]`-Template | Eine Gruppe von Feldern wiederholt sich in mehreren Schemas.
`qsl::qsl_primitive!` | Du hast einen Newtype über einem Integer, etwa `DeviceId(u32)`.
`QslCustom` | Du brauchst volle Kontrolle über die Bytes.
`#region` | Du willst ein verschachteltes Struct mit festem nativem Layout.
## Rust-Templates

Markiere ein Rust-Struct als Template und verwende seinen Namen als Feldtyp:

```rust title="src/meta.rs"
#[qsl::qsl_macro]
pub struct Meta { pub id: u32, pub tick: u64 }
```

```qsl
&Packet { Meta meta; u16 code; }
```

Das generierte `Packet` hat die flachen Felder `meta_id`, `meta_tick` und `code`. `#[qsl::schem_macro]` ist derselbe Marker unter anderem Namen.

Template-Felder können `u8` bis `u64`, `i8` bis `i64`, `String`, `&str`, `qsl::InlineString<N>`, feste Integer-Arrays, `Option<T>` dieser Typen und andere Templates sein. Templates lassen sich bis zu 64 Ebenen tief verschachteln. Zyklen und Namenskollisionen sind Build-Fehler. Ein Verweis auf ein Template kann weder optional sein noch einen Standardwert haben: Mach stattdessen einzelne Template-Felder zu `Option<T>`.

Gib die Rust-Dateien in `build.rs` mit `build_qsl_files_with_rust` an. Dafür braucht die Build-Abhängigkeit `qsl` das Feature `rust-macros`:

```toml title="Cargo.toml"
[build-dependencies]
qsl = { version = "=0.4.0", features = ["rust-macros"] }
```

```rust title="build.rs"
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::var("OUT_DIR")?;
    qsl::build_qsl_files_with_rust(&["packet.qsl"], &[], &["src/meta.rs"], &out)?;
    Ok(())
}
```

## Validierte Skalar-Wrapper {#validated-scalar-wrappers}

`qsl::qsl_primitive!` deklariert einen Newtype über `u8`/`i8`, `u16`/`i16`, `u32`/`i32` oder `u64`/`i64` mit Wertebereich:

```rust title="primitive.rs"
qsl::qsl_primitive!(pub struct DeviceId(u32); range=(1,100););
```

Das Makro prüft zur Compile-Zeit, dass der Wrapper Größe und Ausrichtung des Integers hat, dass der Integer bei Offset 0 liegt, dass der Typ kein `Drop` hat und dass der Bereich in den Integer passt. Der Standardcodec schreibt den Integer als Little-Endian. Der `compact`-Codec speichert `value - min` in so wenigen ganzen Bytes, wie der Bereich braucht. Ein Bereich mit nur einem Wert belegt 0 Bytes.

Das Build-Skript muss den Typ sehen. Leg ihn deshalb in eine Datei, die sowohl `build.rs` als auch dein Crate einbinden. Registriere ihn beim Schema-Builder und generiere dann den Code:

```rust title="build.rs"
mod primitive {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/primitive.rs"));
}
use qsl::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rust = QslSchemaBuilder::new()
        .with_primitive::<primitive::DeviceId>("DeviceId")?
        .with_source("&Packet[safety]{DeviceId id; DeviceId? extra;}")?
        .use_interpreter()?;
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("qsl_gen.rs");
    std::fs::write(out, rust)?;
    Ok(())
}
```

Dieser Weg über den Builder liest keine DEQ-Dateien, sondern verwendet die Standardeinstellungen. Registrierte Wrapper können auch Array-Elemente sein. Um einen davon in einer [verschachtelten Region](#nested-regions) zu verwenden, rufe in deinem Crate zusätzlich `qsl::qsl_native_primitive!(DeviceId, u32);` auf.

## QslCustom {#qslcustom}

Implementiere `qsl::QslCustom`, wenn kein eingebauter Codec passt:
Element | Pflicht | Bedeutung
---|---|---
`SIZE` | Ja | Kodierte Größe in Bytes.
`COMP_SIZE` | Nein | Kodierte Größe mit `compact`. Standard ist `SIZE`.
`qsl_write_unchecked`, `qsl_read_unchecked` | Ja | `unsafe`-Codec über Rohzeiger.
`qsl_comp_write_unchecked`, `qsl_comp_read_unchecked` | Nein | Codec für `compact`. Standard ist der normale Codec.
`qsl_validate(&[u8], compact)` | Für geprüftes Dekodieren | Prüft die Bytes, bevor der geprüfte Decoder deinen Reader aufruft.
`qsl_validate_values(&self)` | Für geprüftes Kodieren | Prüft den Wert, bevor `serialize_checked` deinen Writer aufruft.
Beide Validatoren geben standardmäßig `InvalidSchema` zurück. Geprüfte Methoden lehnen einen eigenen Typ also ab, bis du sie implementierst. Dein `unsafe`-Code ist dafür verantwortlich, genau `SIZE` Bytes zu schreiben und gültige Rust-Werte zu erzeugen.

Verwendet ein Schema einen eigenen Typ, beginnt die generierte Datei mit `use ::qsl::QslCustom;`. Importiere `QslCustom` in dem Modul, das sie einbindet, nicht noch einmal. Ein eigener Typ macht außerdem den [Schema-Hash](/guides/schema-identity/) unvollständig, weil QSL seinen Codec nicht sieht.

## Verschachtelte Regionen {#nested-regions}

Eine Region macht aus einer Folge von Feldern ein eigenes `#[repr(C)]`-Struct:

```qsl
&Parent[safety] {
    #region(aggressive_transmutate="Meta");
    u64 id;
    u32 value;
    str(4) tag;
    #end_region;
    str(3..10) name;
}
```

Daraus entstehen `Meta { id: u64, value: u32, tag: [u8; 4] }` und `Parent { Meta: Meta, name: String }`. Das Feld im übergeordneten Struct heißt wie der Typ der Region. Die Felder der Region landen nicht flach im übergeordneten Struct.

Eine Region kann nur Felder fester Größe enthalten: Integer, feste Strings, feste Arrays und registrierte Wrapper. Optionale Felder werden abgelehnt. Regionen lassen sich bis zu 64 Ebenen tief verschachteln, dürfen sich aber nicht überkreuzen. Ein Typ mit `compact` oder `qps` kann keine Regionen haben.
