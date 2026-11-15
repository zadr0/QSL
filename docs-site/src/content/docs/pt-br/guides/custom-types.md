---
title: Templates Rust e tipos personalizados
description: Reaproveite structs Rust como grupos de campos, registre wrappers de inteiros validados, implemente QslCustom e declare regiões aninhadas.
---

O QSL tem quatro formas de levar os seus próprios tipos Rust para um esquema. Cada uma resolve um problema diferente.
Mecanismo | Use quando
---|---
Template `#[qsl::qsl_macro]` | Um grupo de campos se repete em vários esquemas.
`qsl::qsl_primitive!` | Você tem um newtype sobre um inteiro, como `DeviceId(u32)`.
`QslCustom` | Você precisa de controle total sobre os bytes.
`#region` | Você quer uma struct aninhada com layout nativo fixo.
## Templates Rust

Marque uma struct Rust como template e use o nome dela como tipo de campo:

```rust title="src/meta.rs"
#[qsl::qsl_macro]
pub struct Meta { pub id: u32, pub tick: u64 }
```

```qsl
&Packet { Meta meta; u16 code; }
```

O `Packet` gerado tem os campos planos `meta_id`, `meta_tick` e `code`. `#[qsl::schem_macro]` é o mesmo marcador com outro nome.

Os campos de um template podem ser de `u8` a `u64`, de `i8` a `i64`, `String`, `&str`, `qsl::InlineString<N>`, arrays fixos de inteiros, `Option<T>` desses tipos e outros templates. Templates podem se aninhar até 64 níveis. Ciclos e colisões de nome são erros de build. Uma referência a um template não pode ser opcional nem ter valor padrão: em vez disso, declare campos individuais do template como `Option<T>`.

Liste os arquivos Rust no `build.rs` com `build_qsl_files_with_rust`. Essa função precisa da feature `rust-macros` do `qsl` na dependência de build:

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

`qsl::qsl_primitive!` declara um newtype sobre `u8`/`i8`, `u16`/`i16`, `u32`/`i32` ou `u64`/`i64` com um intervalo de valores:

```rust title="primitive.rs"
qsl::qsl_primitive!(pub struct DeviceId(u32); range=(1,100););
```

A macro verifica em tempo de compilação que o wrapper tem o tamanho e o alinhamento do inteiro, que o inteiro está no offset 0, que o tipo não tem `Drop` e que o intervalo cabe no inteiro. O codec padrão escreve o inteiro em little-endian. O codec `compact` armazena `value - min` no menor número de bytes inteiros que comporta o intervalo. Um intervalo de um único valor ocupa 0 bytes.

O script de build precisa enxergar o tipo, então mantenha-o em um arquivo que tanto o `build.rs` quanto o seu crate incluam. Registre o tipo no builder de esquemas e depois gere o código:

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

Esse caminho pelo builder não lê arquivos DEQ: ele usa as configurações padrão. Wrappers registrados também podem ser elementos de array. Para usar um deles dentro de uma [região aninhada](#nested-regions), chame também `qsl::qsl_native_primitive!(DeviceId, u32);` no seu crate.

## QslCustom {#qslcustom}

Implemente `qsl::QslCustom` quando nenhum codec embutido servir:
Item | Obrigatório | Significado
---|---|---
`SIZE` | Sim | Tamanho codificado, em bytes.
`COMP_SIZE` | Não | Tamanho codificado com `compact`. O padrão é `SIZE`.
`qsl_write_unchecked`, `qsl_read_unchecked` | Sim | Codec `unsafe` com ponteiros brutos.
`qsl_comp_write_unchecked`, `qsl_comp_read_unchecked` | Não | Codec para `compact`. O padrão é o codec normal.
`qsl_validate(&[u8], compact)` | Para decodificação verificada | Verifica os bytes antes que o decodificador verificado chame o seu leitor.
`qsl_validate_values(&self)` | Para codificação verificada | Verifica o valor antes que `serialize_checked` chame o seu escritor.
Por padrão, os dois validadores retornam `InvalidSchema`, então os métodos verificados rejeitam o tipo personalizado até você implementá-los. O seu código `unsafe` é responsável por escrever exatamente `SIZE` bytes e por produzir valores Rust válidos.

Quando um esquema usa um tipo personalizado, o arquivo gerado começa com `use ::qsl::QslCustom;`. Não importe `QslCustom` de novo no módulo que inclui esse arquivo. Um tipo personalizado também deixa o [hash do esquema](/guides/schema-identity/) incompleto, porque o QSL não enxerga o codec dele.

## Regiões aninhadas {#nested-regions}

Uma região transforma uma sequência de campos em uma struct `#[repr(C)]` própria:

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

Isso gera `Meta { id: u64, value: u32, tag: [u8; 4] }` e `Parent { Meta: Meta, name: String }`. O campo no tipo pai recebe o nome do tipo da região. Os campos da região não são achatados no tipo pai.

Uma região só pode conter campos de tamanho fixo: inteiros, strings fixas, arrays fixos e wrappers registrados. Campos opcionais são rejeitados. Regiões podem se aninhar até 64 níveis, mas não podem se cruzar. Um tipo com `compact` ou `qps` não pode ter regiões.
