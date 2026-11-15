---
title: Plantillas de Rust y tipos personalizados
description: Reutiliza structs de Rust como grupos de campos, registra wrappers de enteros validados, implementa QslCustom y declara regiones anidadas.
---

QSL ofrece cuatro formas de llevar tus propios tipos de Rust a un esquema. Cada una resuelve un problema distinto.
Mecanismo | Úsalo cuando
---|---
Plantilla `#[qsl::qsl_macro]` | Un grupo de campos se repite en varios esquemas.
`qsl::qsl_primitive!` | Tienes un newtype sobre un entero, como `DeviceId(u32)`.
`QslCustom` | Necesitas control total sobre los bytes.
`#region` | Quieres un struct anidado con una disposición nativa fija.
## Plantillas de Rust

Marca un struct de Rust como plantilla y usa su nombre como tipo de campo:

```rust title="src/meta.rs"
#[qsl::qsl_macro]
pub struct Meta { pub id: u32, pub tick: u64 }
```

```qsl
&Packet { Meta meta; u16 code; }
```

El `Packet` generado tiene los campos planos `meta_id`, `meta_tick` y `code`. `#[qsl::schem_macro]` es el mismo marcador con otro nombre.

Los campos de una plantilla pueden ser de `u8` a `u64`, de `i8` a `i64`, `String`, `&str`, `qsl::InlineString<N>`, arrays fijos de enteros, `Option<T>` de cualquiera de ellos y otras plantillas. Las plantillas se pueden anidar hasta 64 niveles. Los ciclos y las colisiones de nombres son errores de compilación. Una referencia a una plantilla no puede ser opcional ni tener valor por defecto: en su lugar, haz que los campos concretos de la plantilla sean `Option<T>`.

Enumera los archivos de Rust en `build.rs` con `build_qsl_files_with_rust`. Necesita la feature `rust-macros` de `qsl` en la dependencia de compilación:

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

## Wrappers escalares validados {#validated-scalar-wrappers}

`qsl::qsl_primitive!` declara un newtype sobre `u8`/`i8`, `u16`/`i16`, `u32`/`i32` o `u64`/`i64` con un rango de valores:

```rust title="primitive.rs"
qsl::qsl_primitive!(pub struct DeviceId(u32); range=(1,100););
```

La macro comprueba en tiempo de compilación que el wrapper tiene el tamaño y la alineación del entero, que el entero está en el desplazamiento 0, que el tipo no implementa `Drop` y que el rango cabe en el entero. El códec por defecto escribe el entero en little-endian. El códec `compact` guarda `value - min` en el menor número de bytes completos que abarque el rango. Un rango de un solo valor ocupa 0 bytes.

El script de compilación tiene que ver el tipo, así que ponlo en un archivo que incluyan tanto `build.rs` como tu crate. Regístralo en el constructor de esquemas y después genera el código:

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

Esta vía, con el constructor, no lee archivos DEQ: usa los ajustes por defecto. Los wrappers registrados también pueden ser elementos de un array. Para usar uno dentro de una [región anidada](#nested-regions), llama además a `qsl::qsl_native_primitive!(DeviceId, u32);` en tu crate.

## QslCustom {#qslcustom}

Implementa `qsl::QslCustom` cuando ningún códec integrado te sirva:
Elemento | Obligatorio | Significado
---|---|---
`SIZE` | Sí | Tamaño codificado en bytes.
`COMP_SIZE` | No | Tamaño codificado con `compact`. Por defecto, `SIZE`.
`qsl_write_unchecked`, `qsl_read_unchecked` | Sí | Códec `unsafe` con punteros crudos.
`qsl_comp_write_unchecked`, `qsl_comp_read_unchecked` | No | Códec para `compact`. Por defecto, el códec normal.
`qsl_validate(&[u8], compact)` | Para la decodificación con comprobación | Comprueba los bytes antes de que el decodificador con comprobación llame a tu lector.
`qsl_validate_values(&self)` | Para la codificación con comprobación | Comprueba el valor antes de que `serialize_checked` llame a tu escritor.
Por defecto, los dos validadores devuelven `InvalidSchema`, así que los métodos con comprobación rechazan un tipo personalizado hasta que los implementes. Tu código `unsafe` es responsable de escribir exactamente `SIZE` bytes y de producir valores de Rust válidos.

Cuando un esquema usa un tipo personalizado, el archivo generado empieza con `use ::qsl::QslCustom;`. No vuelvas a importar `QslCustom` en el módulo que lo incluye. Un tipo personalizado también deja incompleto el [hash del esquema](/guides/schema-identity/), porque QSL no puede ver su códec.

## Regiones anidadas {#nested-regions}

Una región convierte una serie de campos consecutivos en un struct `#[repr(C)]` propio:

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

Esto genera `Meta { id: u64, value: u32, tag: [u8; 4] }` y `Parent { Meta: Meta, name: String }`. El campo del padre se llama como el tipo de la región. Los campos de la región no se aplanan en el padre.

Una región solo puede contener campos de tamaño fijo: enteros, cadenas fijas, arrays fijos y wrappers registrados. Los campos opcionales se rechazan. Las regiones se pueden anidar hasta 64 niveles, pero no pueden entrecruzarse. Un tipo con `compact` o `qps` no puede tener regiones.
