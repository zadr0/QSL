---
title: Qué es Quick Schema Language (QSL)
description: Quick Schema Language (QSL) es un lenguaje de esquemas y un generador de códecs binarios para Rust. Qué hace, qué deja en tus manos y cómo se reparten los crates.
---

Quick Schema Language (QSL) es un lenguaje para describir mensajes binarios y un generador de código que convierte esas descripciones en Rust. Escribes un esquema con tipos de campo, rangos de valores y límites de longitud. Durante `cargo build`, QSL lee el esquema y genera structs y métodos que leen y escriben bytes. El programa no analiza el esquema en tiempo de ejecución.

La serialización binaria convierte un valor en bytes; la deserialización recupera el valor a partir de esos bytes. En QSL, el esquema define los campos y el código Rust generado realiza ambas conversiones. Los participantes necesitan esquemas y atributos coincidentes. Empieza por el [inicio rápido de serialización binaria en Rust](/start/quickstart/), después consulta el [formato binario](/internals/wire-format/) y los [ejemplos de empaquetado de bits](/guides/smaller-frames/).

```qsl
&Reading[safety,compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

A partir de este esquema, QSL genera un struct `Reading` con campos `u8`, `u8` y `u16`, y métodos que lo codifican, lo decodifican y lo validan. Con `compact`, los tres valores ocupan 3 + 5 + 10 = 18 bits, así que una trama tiene 3 bytes en lugar de 4.

## Lo que controlas

- **Disposición en la trama.** El ancho de los enteros en la trama y cómo indica la trama qué campos opcionales están presentes.
- **Memoria.** Si una cadena se convierte en `String`, en un `&str` prestado o en un búfer inline, y si la decodificación reutiliza un objeto existente.
- **Validación.** `[safety]` genera métodos de decodificación con comprobación. Verifican límites, rangos, UTF-8 y un presupuesto de tamaño total antes de devolver un valor.

## Lo que queda en tu aplicación

QSL define cómo se representan los datos. Aparte de `qsl::RawSocket`, un wrapper ligero que envía tramas con prefijo de longitud sobre TCP, no se ocupa del transporte y no incluye TLS, enrutamiento ni negociación de versiones del esquema. Los dos lados de un intercambio necesitan el mismo esquema y los mismos atributos. Si cambias el formato, despliega el cambio en todos los extremos a la vez. QSL no detecta qué formato usa una trama.

Cada tipo generado incluye los hashes de su esquema como constantes (consulta [identidad del esquema](/guides/schema-identity/)). Puedes enviarlos en tu propio handshake. QSL no lo hace por ti.

## Crates
Crate | Versión | Función
---|---|---
`qsl` | 0.3.0 | El crate del que dependes. Reexporta el runtime y proporciona `build_qsl_files` y las macros de atributo.
`qsl-core` | 0.1.0 | Runtime compartido: errores, cursores de E/S, contenedores de cadenas, validación, búferes.
`qsl-lexer` | 0.3.0 | Parser de esquemas `.qsl` y de archivos de configuración `.deq`, constructor de esquemas.
`qsl-interpreter` | 0.3.0 | Genera código fuente Rust a partir de esquemas validados.
`qsl-macros-impl` | 0.1.0 | Los atributos `qsl_macro` y `schem_macro`, reexportados por `qsl`.
La mayoría de los proyectos solo añaden `qsl`, una vez en `[dependencies]` y otra en `[build-dependencies]`. [Arquitectura](/internals/architecture/) muestra cómo dependen los crates entre sí.

## Estado

QSL 0.3.0 se desarrolla y se prueba en Windows 11 x86-64 con Rust 1.98, que también es la versión mínima compatible. Los atributos hay que activarlos de forma explícita. Un atributo cambia el código generado, pero ninguno garantiza una mejora de velocidad. La  recoge qué se midió y en qué condiciones.

Siguiente: [Inicio rápido](/start/quickstart/).
