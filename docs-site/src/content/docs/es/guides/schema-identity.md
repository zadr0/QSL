---
title: Identidad del esquema
description: Los hashes de esquema estricto y ligero, qué entra en ellos y cómo compararlos en un handshake.
---

QSL calcula dos hashes para cada tipo mientras analiza el esquema y los guarda como constantes en el código generado. La codificación y la decodificación nunca los calculan, y las tramas no los contienen.

```rust
let strict: u128 = Packet::SCHEMA_HASH_STRICT;
let light: u128 = Packet::SCHEMA_HASH_LIGHT;
let h = Packet::SCHEMA_HASHES; // también <Packet as qsl::SchemaIdentity>::SCHEMA_HASHES
println!("{} strict={} light={}", h.algorithm.name(), h.strict_hex(), h.light_hex());
```
Hash | Cubre
---|---
Estricto | Nombres del tipo y de los campos, orden de los campos, tipos, rangos, opcionalidad y opciones de formato.
Ligero | Lo mismo, sin los nombres del tipo ni de los campos.
`&A { u32 id; }` y `&B { u32 sequence; }` tienen el mismo hash ligero y hashes estrictos distintos. Intercambiar dos campos de tipos distintos cambia los dos.

`SCHEMA_HASHES` es un valor `qsl::SchemaHashes`:
Campo | Significado
---|---
`version` | Versión del formato de entrada del hash; actualmente, 1.
`algorithm` | `Xxh3_64` o `Xxh3_128`.
`strict`, `light` | Los hashes. Con XXH3-64, los 64 bits superiores son cero.
`complete` | `false` si el tipo usa un tipo [`QslCustom`](/guides/custom-types/#qslcustom). En ese caso, el hash solo cubre la ruta de Rust y el tamaño del tipo personalizado, no su códec.
Compara `version`, `algorithm` y `complete` junto con el hash. `strict_hex()` y `light_hex()` formatean un hash como 16 o 32 dígitos hexadecimales, según el algoritmo.

## Algoritmo

XXH3-64 por defecto. Para 128 bits, configúralo en un [archivo DEQ](/language/deq/):

```deq
!config USE_HASH = xxHash128;
```

El ajuste se aplica a toda la compilación. Ninguno de los dos algoritmos es criptográfico: detectan discrepancias, no manipulaciones.

## Qué cambia el hash

- La secuencia de campos, una vez evaluadas las constantes y expandidas las plantillas de Rust.
- Los tipos, los rangos de valores, los límites de cadenas y arrays, y qué campos son opcionales.
- Las opciones del formato binario: `compact`, `alignment` junto con `compact`, `optionals=bitmap`, `predict_seq`, `predict_seq` con `opt_table`, `opt_table=legacy`, `state_bits`.
- Los códecs de cadenas: `decompress=true` y `decompress=true`.
- `qps` y su quantum. `[qps]`, `[qps=Q16]` y `[quant=Q16]` son equivalentes.
- En el formato por defecto de los campos opcionales, las etiquetas de los campos opcionales y el ancho de las etiquetas.

## Qué no lo cambia

`safety`, `lifetime`, `inline` y sus variantes, `default`, `opt_table` por sí solo, `alignment` sin `compact`, `aggressive`, `transmutate`, `cache` y `loyal_cache`, `QSL_RT` y el número de slots. Cambian el código Rust generado o la gestión de la memoria, no los bytes.

`String`, `&str`, `StackString` y `lifetime` producen los mismos bytes con los mismos límites, así que dan el mismo hash.

## Cómo usarlo

Los hashes describen el formato de forma conservadora. No demuestran que dos esquemas distintos no puedan decodificar los mismos bytes. Para asegurarte de que los dos lados coinciden, intercambia los hashes en tu propio handshake antes de enviar datos. QSL no incluye ningún handshake.
