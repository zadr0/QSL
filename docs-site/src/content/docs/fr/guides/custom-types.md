---
title: Modèles Rust et types personnalisés
description: "Réutiliser des structures Rust comme groupes de champs, enregistrer des wrappers d’entiers validés, implémenter QslCustom et déclarer des régions imbriquées."
---

QSL offre quatre moyens d’introduire vos propres types Rust dans un schéma. Ils répondent à des problèmes différents.
Mécanisme | À utiliser quand
---|---
Modèle `#[qsl::qsl_macro]` | Un groupe de champs se répète d’un schéma à l’autre.
`qsl::qsl_primitive!` | Vous avez un newtype autour d’un entier, comme `DeviceId(u32)`.
`QslCustom` | Vous avez besoin d’un contrôle total sur les octets.
`#region` | Vous voulez une structure imbriquée avec une disposition native fixe.
## Modèles Rust

Marquez une structure Rust comme modèle (*template*), puis utilisez son nom comme type de champ :

```rust title="src/meta.rs"
#[qsl::qsl_macro]
pub struct Meta { pub id: u32, pub tick: u64 }
```

```qsl
&Packet { Meta meta; u16 code; }
```

Le `Packet` généré a des champs à plat `meta_id`, `meta_tick` et `code`. `#[qsl::schem_macro]` est le même marqueur sous un autre nom.

Les champs d’un modèle peuvent être de type `u8` à `u64`, `i8` à `i64`, `String`, `&str`, `qsl::InlineString<N>`, des tableaux fixes d’entiers, des `Option<T>` de ces types, ou d’autres modèles. Les modèles peuvent s’imbriquer jusqu’à 64 niveaux. Les cycles et les collisions de noms provoquent des erreurs de compilation. Une référence à un modèle ne peut être ni optionnelle ni avoir de valeur par défaut : rendez plutôt optionnels les champs concernés du modèle, avec `Option<T>`.

Listez les fichiers Rust dans `build.rs` avec `build_qsl_files_with_rust`. Cette fonction nécessite la feature `rust-macros` de `qsl` dans la dépendance de build :

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

## Wrappers scalaires validés {#validated-scalar-wrappers}

`qsl::qsl_primitive!` déclare un newtype autour de `u8`/`i8`, `u16`/`i16`, `u32`/`i32` ou `u64`/`i64`, avec une plage de valeurs :

```rust title="primitive.rs"
qsl::qsl_primitive!(pub struct DeviceId(u32); range=(1,100););
```

La macro vérifie à la compilation que le wrapper a la taille et l’alignement de l’entier, que l’entier se trouve à l’offset 0, que le type n’implémente pas `Drop` et que la plage tient dans l’entier. Le codec par défaut écrit l’entier en little-endian. Le codec `compact` stocke `value - min` dans le plus petit nombre d’octets entiers capable de contenir la plage. Une plage d’une seule valeur occupe 0 octet.

Le script de build doit voir le type : placez-le donc dans un fichier inclus à la fois par `build.rs` et par votre crate. Enregistrez-le auprès du builder de schémas, puis générez le code :

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

Ce chemin via le builder ne lit pas les fichiers DEQ : il utilise les paramètres par défaut. Les wrappers enregistrés peuvent aussi servir d’éléments de tableau. Pour en utiliser un dans une [région imbriquée](#nested-regions), appelez aussi `qsl::qsl_native_primitive!(DeviceId, u32);` dans votre crate.

## QslCustom {#qslcustom}

Implémentez `qsl::QslCustom` quand aucun codec prédéfini ne convient :
Élément | Obligatoire | Signification
---|---|---
`SIZE` | Oui | Taille encodée, en octets.
`COMP_SIZE` | Non | Taille encodée avec `compact`. Vaut `SIZE` par défaut.
`qsl_write_unchecked`, `qsl_read_unchecked` | Oui | Codec `unsafe` à pointeurs bruts.
`qsl_comp_write_unchecked`, `qsl_comp_read_unchecked` | Non | Codec pour `compact`. Par défaut, le codec normal.
`qsl_validate(&[u8], compact)` | Pour le décodage vérifié | Vérifie les octets avant que le décodeur vérifié n’appelle votre fonction de lecture.
`qsl_validate_values(&self)` | Pour l’encodage vérifié | Vérifie la valeur avant que `serialize_checked` n’appelle votre fonction d’écriture.
Par défaut, les deux validateurs renvoient `InvalidSchema` : les méthodes vérifiées rejettent donc un type personnalisé tant que vous ne les avez pas implémentés. Votre code `unsafe` doit écrire exactement `SIZE` octets et produire des valeurs Rust valides.

Quand un schéma utilise un type personnalisé, le fichier généré commence par `use ::qsl::QslCustom;`. N’importez pas `QslCustom` une seconde fois dans le module qui l’inclut. Un type personnalisé rend aussi le [hash du schéma](/guides/schema-identity/) incomplet, car QSL ne voit pas son codec.

## Régions imbriquées {#nested-regions}

Une région transforme une suite de champs en une structure `#[repr(C)]` à part entière :

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

Cela génère `Meta { id: u64, value: u32, tag: [u8; 4] }` et `Parent { Meta: Meta, name: String }`. Le champ du parent porte le nom du type de la région. Les champs de la région ne sont pas mis à plat dans le parent.

Une région ne peut contenir que des champs de taille fixe : entiers, chaînes fixes, tableaux fixes et wrappers enregistrés. Les champs optionnels sont refusés. Les régions peuvent s’imbriquer jusqu’à 64 niveaux, mais ne peuvent pas se chevaucher. Un type avec `compact` ou `qps` ne peut pas avoir de régions.
