---
title: Actualizar a 0.3.0
description: Cambios en qsl 0.3.0 (la separación de qsl-core y el conjunto reducido de representaciones de cadenas).
qslResources: [{"label": "notas de 0.2.0", "url": "https://github.com/zadr0/QSL/blob/main/docs/migration-0.2.0.en.md"}]
---

QSL 0.3.0 traslada el runtime compartido a un crate nuevo, `qsl-core` 0.1.0, y elimina API antiguas de cadenas. El formato binario del mapa de bits (v2) y de las cadenas normales no cambia.

## Actualiza las versiones a la vez

```toml title="Cargo.toml"
[dependencies]
qsl = "=0.4.0"

[build-dependencies]
qsl = "=0.4.0"
```

Si dependes directamente de los crates internos, actualiza también `qsl-lexer` y `qsl-interpreter` a 0.3.0. Las macros de atributo siguen en `qsl-macros-impl` 0.1.0.

## Separación en crates
Crate | Contiene | Depende de
---|---|---
`qsl-core` | Errores, modelo de esquema, análisis compartido, E/S, contenedores UTF-8, validación, búferes, códecs de bajo nivel | Ningún otro crate de QSL
`qsl-lexer` | Lexer, parsers de QSL y DEQ, carga de archivos, expansión de plantillas, constructor de esquemas | `qsl-core`
`qsl-interpreter` | Generación de código Rust | `qsl-lexer`
`qsl` | Fachada, macros, puntos de entrada compatibles | Todos los anteriores y `qsl-macros-impl`
Las rutas de módulo antiguas siguen funcionando como reexportaciones. `QslError` y `QslCustom` son un único tipo cada uno, sea cual sea la ruta desde la que los importes.

## Cadenas

Quedan cuatro representaciones:
Opción | Tipo de Rust
---|---
Cadena normal | `String`
`aggressive`, con el límite superior dentro de `STACK_LIMIT` | `StackString<N>`
`decompress=true`, con el límite dentro de `STACK_LIMIT` | `InlineString<N>`
`lifetime` | `&str`
Eliminados: `binary_opt=guard`, `binary_opt=true`, `Utf8Guard`, `Utf8CopyGuard` y el modo de cadenas de bytes. Los esquemas que los usan no compilan; cambia cada campo a una de las cuatro representaciones anteriores.

*Nuevo* y *reutilización* se eligen con el método, no con el tipo. Antes de decodificar en un `String` existente, dale capacidad suficiente. Consulta [cadenas](/language/strings/) y [búferes y reutilización](/guides/memory/#reading-into-existing-memory).

## Los métodos de los traits son unsafe

`Serializable::serialize` y `BorrowSerializable::serialize` ahora son `unsafe`: quien llama garantiza que el valor es válido para el esquema. Usa `serialize_checked` en su lugar. El `serialize()` generado en los tipos con pool RT o con empaquetado `qps` activo es otro método. Sigue siendo seguro y devuelve un `Result`.

## De 0.1 a 0.2

Si vienes de 0.1, lee primero las notas de 0.2.0. Lo principal: cambió el formato del mapa de bits (v2), se eliminaron la feature `native` de Cargo y los sockets y las regiones IPC de Windows, y `serialize` ahora devuelve `Vec<u8>`.
