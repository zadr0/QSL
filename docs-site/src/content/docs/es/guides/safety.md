---
title: Validación de la entrada
description: Qué genera [safety], qué verifican los métodos con comprobación y qué sigue siendo responsabilidad de quien llama.
---

Los bytes que llegan de un socket, de un archivo o de otro proceso no son de confianza. Decodifícalos con los métodos con comprobación, que solo se generan para los tipos con el atributo `safety`.

```qsl
&Packet[safety] {
    u32 id;
    str(1..100) text;
}
```

```rust
let bytes = packet.serialize_checked()?;
let decoded = Packet::deserialize_checked(&bytes)?;

// O decodifica en memoria que ya tienes:
let mut slot = std::mem::MaybeUninit::uninit();
let decoded: &mut Packet = Packet::deserialize_checked_into(&bytes, &mut slot)?;
```

## Qué verifica una decodificación con comprobación

Antes de devolver un valor, el decodificador generado comprueba que:

- la trama mide al menos lo mismo que la trama válida más pequeña, contiene todos los campos y termina donde indica el esquema;
- cada número está dentro de su rango declarado;
- la longitud de cada cadena y de cada array está dentro de sus límites;
- las cadenas son UTF-8 válido;
- los metadatos de los campos opcionales son coherentes con el esquema;
- los bits y bytes de relleno son cero;
- las cadenas comprimidas, una vez restauradas, no superan su límite;
- el tamaño total de los valores decodificados no supera `MAX_DATA_SIZE`.

La validación y la decodificación se hacen en una sola pasada. No necesitas llamar antes a `validate`. Si la decodificación falla, `deserialize_checked_into` deja `slot` sin inicializar.

## Límites de tamaño

`MAX_DATA_SIZE` (65535 bytes por defecto) limita la trama de entrada y la carga útil decodificada total. Las longitudes leídas de la trama se comprueban contra sus límites y contra este presupuesto antes de reservar memoria. Súbelo o bájalo en un [archivo DEQ](/language/deq/).

El ejemplo safety del repositorio pasa a `validate` tres tramas incorrectas para este tipo:

```qsl
&Bounded[safety] { u8(1..3) kind; str(1..100) name; }
```
Entrada | Resultado
---|---
`09 01 00 00 00 61` | `InvalidValue`: `kind` vale 9, fuera de `1..3`.
`01 ff ff ff ff` | `UnexpectedEof`: 5 bytes son menos que la trama válida más pequeña, de 6 bytes.
`01 01 00 00 00 ff` | `InvalidUtf8`: `ff` no es UTF-8.
Una longitud falsificada en una trama de tamaño válido, como `01 ff ff ff ff 61`, falla con `InvalidValue` porque la longitud se compara con el límite `100` antes de copiar ningún byte.

## Codificación con comprobación

`serialize_checked` comprueba el valor contra el esquema (rangos, longitudes, UTF-8 de las cadenas de tamaño fijo, `MAX_DATA_SIZE`) y devuelve un `Vec<u8>` nuevo. `serialize_checked_to` escribe en un slice que proporcionas tú y devuelve el número de bytes escritos, o `BufferTooSmall`. Si falla la comprobación final de tamaño, puede que el slice ya esté escrito en parte. Los dos se generan para todos los tipos, con `[safety]` o sin él.

## Métodos sin comprobación

`[safety]` añade métodos con comprobación. No elimina los que no la tienen. Los métodos del trait `qsl::Serializable`, como `deserialize_into`, y los lectores con punteros crudos suponen que:

- la trama está completa y es válida para este esquema;
- la memoria de destino es lo bastante grande, está bien alineada y no se solapa con la entrada;
- las entradas prestadas viven más que el resultado.

Úsalos solo con tramas que hayas generado tú o que ya hayas validado. Son `unsafe`, incluidos `Serializable::serialize` y `BorrowSerializable::serialize` desde 0.3.0. El `serialize()` generado en los tipos con pool RT o con empaquetado `qps` activo es otro método, seguro, que devuelve un `Result`.

## Límites de la validación

- La validación demuestra que los bytes tienen una forma válida para el esquema. No demuestra qué esquema los produjo: dos esquemas con la misma disposición aceptan los mismos bytes. Usa los [hashes del esquema](/guides/schema-identity/) o tu propio discriminador en un handshake.
- No autentica al emisor.
- Un [tipo personalizado](/guides/custom-types/) es tan seguro como su implementación de `qsl_validate`. La implementación por defecto rechaza cualquier entrada.

El generador elige la salida según el tamaño máximo demostrado de la trama. Hasta STACK_LIMIT inclusive usa `StackOut<N>`; un límite mayor o desconocido selecciona alloc::`Vec<u8>`. Las cabeceras transmitidas cuentan, los metadatos del objeto de salida no.
