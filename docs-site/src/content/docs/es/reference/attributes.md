---
title: Atributos
description: Todos los atributos de tipo y modificadores de campo de QSL, qué generan y si cambian el formato binario.
---

Los atributos de tipo van entre corchetes después del nombre: `&Packet[safety,compact]`. Los modificadores de campo van entre barras verticales después del nombre del campo: `str(1..100) name;`. Todos son opcionales y cada uno puede aparecer una sola vez.

**Trama** indica si el atributo cambia los bytes y, por tanto, el [hash del esquema](/guides/schema-identity/). Si los cambia, los dos extremos deben usar el mismo ajuste.

## Resumen
Atributo | Dónde | Trama | Efecto
---|---|---|---
[`safety`](#safety) | tipo | no | Genera métodos de decodificación con comprobación.
[`compact`](#compact) | tipo | sí | Guarda los valores con rango en el mínimo número de bits.
[`alignment`](#alignment) | tipo | con `compact` | Hace que cada campo compacto empiece en un límite de byte.
[`optionals=bitmap`](#optionalsbitmap) | tipo | sí | Máscara de presencia sobre los campos opcionales.
[`predict_seq`](#predict_seq) | tipo | sí | Presencia deducida de la longitud de la carga útil.
[`opt_table`](#opt_table) | tipo | solo `legacy` | Tabla estática de lectores de campos opcionales.
[`qps`, `quant`](#qps-quant) | tipo | sí | Empaquetado reversible de campos numéricos.
[`aggressive`](#aggressive) | tipo | no | Almacenamiento inline para cadenas y arrays acotados.
[`lifetime`](#lifetime) | tipo | no | Toma prestados de la entrada las cadenas y los bytes.
[`transmutate`](#transmutate) | tipo | no | Disposición nativa igual a la disposición en la trama.
[`inline`](#inline) | tipo | no | Política de `#[inline]` para los métodos generados.
[`slots=N`](#slotsn) | tipo | no | Número de slots RT para este tipo.
[`cache=N`, `loyal_cache`](#cachen-loyal_cache) | tipo | no | Salida en un `CachedFrame` de quien llama.
[`state_bits=Qn`](#state_bitsqn) | tipo | sí | Formato heredado de índice de presencia.
[`decompress=true`](#decompresstrue) | campo | sí | Compresión LZ acotada para una cadena.
[`default=N`](#defaultn) | campo | no | Genera `impl Default`.
## Atributos de tipo

### safety {#safety}

Genera `deserialize_checked` y `deserialize_checked_into`, que decodifican entrada no confiable. Antes de devolver un valor comprueban límites, rangos, UTF-8, metadatos de los campos opcionales, relleno, descompresión y el presupuesto total `MAX_DATA_SIZE`. Los métodos sin comprobación siguen disponibles. `serialize_checked`, `serialize_checked_to`, `validate` y `validate_values` se generan con `safety` o sin él. Consulta [validación de la entrada](/guides/safety/).

### compact {#compact}

Guarda cada entero con rango como `x - min` en `bit_length(max - min)` bits, empaquetados sin respetar los límites de byte. `u8(0..7)`, `u8(0..31)` y `u16(0..1023)` ocupan 3, 5 y 10 bits. Las longitudes de cadenas y arrays se guardan igual, dentro de sus límites. Funciona con `opt_table` y `optionals=bitmap`. No se puede combinar con `predict_seq`, `state_bits` ni `qps`. Consulta [tramas más pequeñas](/guides/smaller-frames/).

### alignment {#alignment}

Con `compact`, hace que cada campo empiece en un límite de byte. La trama puede crecer. Sin `compact`, no cambia el formato. No es la alineación en memoria de los búferes. No se puede combinar con `qps`.

### optionals=bitmap {#optionalsbitmap}

Escribe una máscara con un bit por campo opcional, de 0 a 16 bytes de ancho según cuántos haya. Admite hasta 128 campos opcionales. Es incompatible con la máscara que se usaba antes de 0.2.0. No se puede combinar con `state_bits`, `opt_table=legacy` ni `qps`. Consulta [campos opcionales](/language/optionals/#bitmap).

### predict_seq {#predict_seq}

Deduce qué campos opcionales están presentes a partir de la longitud de la carga útil. Requiere que cada combinación de campos presentes dé una longitud distinta. Se ignora si está activo `optionals=bitmap`. No se puede combinar con `compact`. Consulta [campos opcionales](/language/optionals/#length-index-predict_seq).

### opt_table {#opt_table}

`opt_table` (u `opt_table=static`) despacha los lectores de campos opcionales mediante una tabla estática en lugar de un `match`. Mismos bytes, mismo hash. Combinado con `predict_seq`, selecciona el formato de grafo de longitudes que producía `[opt_table]` antes de 0.2.0.

`opt_table=legacy` es otro formato. Después de los campos obligatorios escribe un byte con el número de campos y un `u32` little-endian por campo opcional: `0` si está ausente y, si no, el desplazamiento del valor en la carga útil opcional más 1. No se puede combinar con `opt_table` a secas, `predict_seq`, `state_bits`, `optionals=bitmap` ni `qps`.

### qps, quant {#qps-quant}

`qps` equivale a `qps=Q16`. `qps=Q2`, `Q4`, `Q8` y `Q16` fijan el quantum. `quant=Qn` es un alias. `quant` sin valor es un error, y también lo es usar los dos nombres. Solo empaqueta campos numéricos: las cadenas, los arrays y sus longitudes conservan sus códecs, y los campos numéricos deben ir antes que ellos. No se puede combinar con `compact`, `alignment`, `optionals=bitmap` ni `opt_table=legacy`.

Sin pool RT, un tipo con el empaquetado activo obtiene un `serialize()` seguro que devuelve `QslResult<QuantizedOut>`. Consulta [tramas más pequeñas](/guides/smaller-frames/).

### aggressive {#aggressive}

Cambia el almacenamiento en Rust, no los bytes ni qué entrada se acepta:

- `str(min..max)` pasa a ser `StackString<max>` y `array<T>(min..max)` pasa a ser `InlineArray<T, max>` cuando el máximo cabe en `STACK_LIMIT`. Los campos más grandes siguen siendo `String` o `Vec`.
- Un campo opcional numérico se guarda como un entero `NonZero` del menor ancho que abarque su rango, si ese ancho no es mayor que el del tipo del campo.

### lifetime {#lifetime}

Da al tipo un tiempo de vida `'a` y toma prestado de la entrada: las cadenas pasan a ser `&'a str` y `array<u8>` pasa a ser `&'a [u8]`. La entrada debe vivir más que el valor. Los arrays con otros tipos de elemento y las cadenas comprimidas no se pueden tomar prestados y hacen fallar la compilación, igual que un tipo `lifetime` sin nada que tomar prestado.

### transmutate {#transmutate}

Hace que la disposición nativa del tipo sea igual a su disposición en la trama. Todos los campos deben tener tamaño fijo: `compact`, `qps`, los campos opcionales y las cadenas o arrays con rango son errores de compilación. El tamaño en la trama debe ser igual al tamaño del struct, así que añade campos `padding(N)` donde Rust inserte relleno.

Sin `lifetime`, `deserialize_transmutate_copy` copia la trama en un valor propio de un solo bloque cuando `NATIVE_WIRE_COMPATIBLE` es verdadero y, si no, decodifica campo a campo. Con `[safety]` es seguro y valida el resultado. Sin `[safety]` es `unsafe`.

Con `lifetime`, `deserialize_overlay` devuelve un `&Self` que apunta dentro de la entrada. La compilación comprueba que cada desplazamiento en la trama sea igual al desplazamiento nativo y falla en targets big-endian. Con `[safety]`, `deserialize_overlay` comprueba `NATIVE_WIRE_COMPATIBLE`, la longitud y la alineación de la entrada y, después, los valores, y `deserialize_checked` hace lo mismo. Sin `[safety]`, `deserialize_overlay` es `unsafe` y no comprueba nada.

Los dos tienen variantes `_unchecked` que se saltan las comprobaciones.

### inline {#inline}

`inline` (igual que `inline=hint`), `inline=always` o `inline=never` ponen ese atributo en los métodos generados. Sin él, los métodos generados no llevan ningún atributo inline y decide el compilador. `always` puede aumentar el tamaño del código; no garantiza una mejora de velocidad.

### slots=N {#slotsn}

Sustituye el número de slots RT para este tipo, de 1 a 64. Solo tiene efecto con `!config QSL_RT = true;`. Consulta [pool de slots RT](/guides/memory/#rt-slot-pool).

### cache=N, loyal_cache {#cachen-loyal_cache}

Genera `CACHE_BYTES` y `serialize_cached`, que escribe en un `qsl::modernization::CachedFrame<N>` proporcionado por quien llama. `loyal_cache` fija `CACHE_BYTES` en el tamaño máximo de trama. El tipo necesita un tamaño máximo de trama finito, y un `N` menor que ese tamaño hace fallar la compilación. Esto prepara la memoria de salida; no guarda valores en caché. Consulta [tramas de salida en caché](/guides/memory/#cached-output-frames).

### state_bits=Qn {#state_bitsqn}

`Q1` (por defecto), `Q2`, `Q4`, `Q8` o `Q16`. Selecciona un formato heredado de índice de presencia. No es un modo de compresión. No se puede combinar con `optionals=bitmap`, `compact` ni `opt_table=legacy`.

## Modificadores de campo

### binary_opt {#binary_opt}

binary_opt se ha eliminado por completo. Use [lifetime] para &str prestado, [aggressive] para StackString acotado y decompress=true para InlineString. STACK_LIMIT se aplica a todas las cadenas y matrices.

### decompress=true {#decompresstrue}

Codifica la cadena con el códec LZ acotado de QSL. Las cadenas de menos de 32 bytes y las que la compresión no acortaría se guardan sin comprimir. Cada token decodificado se comprueba contra el tamaño restaurado, la capacidad del destino y `MAX_DATA_SIZE`. La cadena necesita un límite superior de 65535 bytes como máximo. Una cadena con rango pasa a ser `InlineString<max>`, o `String` por encima de `STACK_LIMIT`. No se puede combinar con `lifetime` ni con `lifetime`. Consulta [cadenas comprimidas](/language/strings/#compressed-strings).

### default=N {#defaultn}

Genera `impl Default` para el tipo. Este campo empieza en `N`, o en `Some(N)` si es opcional. Los demás campos empiezan en su valor válido más pequeño. `N` debe estar dentro del rango del campo, y el campo debe ser numérico. Un tipo que además tenga un campo de tipo personalizado no compila. No cambia los bytes ni el hash.
