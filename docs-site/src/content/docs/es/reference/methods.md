---
title: Métodos generados
description: Métodos y constantes que QSL genera en cada tipo del esquema, agrupados según lo que necesitan de quien llama.
qslResources: [{"label": "docs.rs", "url": "https://docs.rs/qsl/0.3.0/qsl/"}]
---

Cada declaración produce un tipo de Rust con los métodos que siguen. Cuáles existen depende de los atributos del esquema. La lista completa de elementos públicos de los crates del runtime está en docs.rs.

## Con comprobación: seguros con datos no confiables

Solo se generan si el tipo tiene `[safety]`:
Método | Firma (simplificada) | Notas
---|---|---
`deserialize_checked` | `fn(&[u8]) -> QslResult<Self>` | Valida mientras decodifica. Con `lifetime`, la entrada es `&'a [u8]`.
`deserialize_checked_into` | `fn(&[u8], &mut MaybeUninit<Self>) -> QslResult<&mut Self>` | Decodifica en tu memoria. Si hay un error, el destino queda sin inicializar.
Se generan para todos los tipos:
Método | Firma (simplificada) | Notas
---|---|---
`validate` | `fn(&[u8]) -> QslResult<()>` | Comprueba una trama completa sin construir un valor. No hace falta antes de `deserialize_checked`.
`validate_values` | `fn(&self) -> QslResult<()>` | Comprueba rangos, longitudes, el UTF-8 de las cadenas fijas y el presupuesto `MAX_DATA_SIZE`.
`serialize_checked` | `fn(&self) -> QslResult<`Vec<u8>`>` | Comprueba el valor y después lo codifica.
`serialize_checked_to` | `fn(&self, &mut [u8]) -> QslResult<usize>` | Codifica en un slice de al menos `serialized_size()` bytes y devuelve los bytes escritos.
Los tipos con un campo opcional numérico tienen además:
Método | Firma (simplificada) | Notas
---|---|---
`try_new` | `fn(field, ...) -> QslResult<Self>` | Recibe todos los campos en orden de declaración, los opcionales como `Option<T>`, y comprueba los valores.
`name` | `fn(&self) -> Option<T>` | Lee el campo opcional `name`.
`set_name` | `fn(&mut self, Option<T>) -> QslResult<()>` | Lo cambia. `InvalidValue` si el valor está fuera de rango.
Consulta [campos opcionales en Rust](/language/declarations/#optional-fields-in-rust).

## De confianza: quien llama garantiza la validez

Son métodos del trait `qsl::Serializable`. Los tipos con `lifetime` implementan en su lugar `qsl::BorrowSerializable<'a>`, donde los decodificadores se llaman `deserialize_borrowed`, `deserialize_borrowed_into` y `deserialize_borrowed_unsafe`. En los dos traits, todos los métodos salvo `serialized_size` y `get_size` son `unsafe`.
Método | Qué garantiza quien llama
---|---
`serialize`, `serialize_to`, `serialize_into_unsafe`, `serialize_unsafe` | El valor es válido para el esquema; el destino tiene `serialized_size()` bytes escribibles.
`deserialize`, `deserialize_unsafe` | La entrada es una única trama completa y válida.
`deserialize_into(ptr, len, out)` | Trama válida; `out` es escribible, está alineado y no se solapa con la entrada. No se hace drop del valor anterior de `out`.
`deserialize_reuse(input, &mut value)` | Trama válida; cada búfer de cadena reutilizado tiene capacidad suficiente. Conserva los búferes cuando la disposición lo permite.
`deserialize_prepared_into(ptr, len, &mut value)` | Lo mismo, con un puntero crudo y una longitud.
Métodos seguros del trait:
Método | Devuelve
---|---
`serialized_size(&self)` | El espacio que hay que proporcionar para este valor: la longitud exacta de la trama, o un límite superior para los valores con cadenas comprimidas.
`get_size()` | La longitud de trama más pequeña posible.
## Métodos especializados
Método | Cuándo
---|---
`serialize()` | Con pool RT: seguro, comprueba el valor, espera a que haya un slot libre y devuelve un lease. En un tipo con empaquetado `qps` activo y sin pool RT: seguro, devuelve `QuantizedOut`. Consulta [pool de slots RT](/guides/memory/#rt-slot-pool).
`serialize_quantized_to` | `qps` activo. Comprueba los valores y la capacidad y después ejecuta el kernel de empaquetado.
`quant_write_prepared`, `quant_read_padded` | `qps` activo. Métodos de `qsl::payload_quant::QuantizedRecord`. `unsafe`: quien llama garantiza los rangos, el relleno y una memoria disjunta y alineada.
`serialize_qps_to`, `deserialize_qps_into`, `deserialize_qps` | Adaptadores de `ByteQps`. Solo para tipos con `[safety]` y `qps` y sin cadenas ni arrays.
`serialize_cached` | `cache=N` o `loyal_cache`. Escribe en un `CachedFrame`.
`deserialize_transmutate_copy`, `deserialize_overlay` | [`transmutate`](/reference/attributes/#transmutate). Cada uno tiene una variante `_unchecked` que se salta las comprobaciones.
## Constantes
Constante | Significado
---|---
`SCHEMA_HASH_STRICT: u128` | Hash que incluye los nombres. Consulta [identidad del esquema](/guides/schema-identity/).
`SCHEMA_HASH_LIGHT: u128` | Hash sin los nombres.
`SCHEMA_HASHES` | Los dos, con el algoritmo, la versión y si están completos. También `<T as qsl::SchemaIdentity>::SCHEMA_HASHES`.
`MAX_DATA_SIZE` | El presupuesto que usan los métodos con comprobación.
`NATIVE_BYTES`, `NATIVE_ALIGNMENT` | `size_of` y `align_of` del struct generado.
`WIRE_MAX_BYTES: Option<usize>` | El tamaño máximo de trama, o `None` si no está acotado.
`CACHE_BYTES` | Capacidad para `serialize_cached`. Solo con `cache=N` o `loyal_cache`.
`NATIVE_WIRE_COMPATIBLE` | Si la disposición nativa es igual a la disposición en la trama. Solo con `transmutate`.
`QUANT_PADDING_BYTES` | Bytes legibles adicionales que necesitan las lecturas preparadas de `qps`. Actualmente, `0`.
`REUSES_STORAGE` es una constante del trait `Serializable`: `<T as qsl::Serializable>::REUSES_STORAGE` vale `true` cuando `deserialize_reuse` actualiza los búferes en su sitio.

El generador elige la salida según el tamaño máximo demostrado de la trama. Hasta STACK_LIMIT inclusive usa `StackOut<N>`; un límite mayor o desconocido selecciona alloc::`Vec<u8>`. Las cabeceras transmitidas cuentan, los metadatos del objeto de salida no.
