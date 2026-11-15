---
title: Tipos
description: Los tipos integrados de QSL, los tipos de Rust que generan y su codificación por defecto en la trama.
---

## Tipos integrados
Esquema | Tipo de Rust | Notas
---|---|---
`u8` `u16` `u32` `u64` `i8` `i16` `i32` `i64` | El mismo primitivo |
`T(A..B)` para un entero `T` | El mismo primitivo | El valor debe estar dentro del rango. `compact` y `qps` pueden reducirlo en la trama.
`flags(N)` | `u8`, `u16`, `u32` o `u64`, según `N` | `N` va de 1 a 64. Los bits por encima de `N` deben ser cero.
`str`, `string`, `String` | `String` | Longitud variable, limitada solo por `MAX_DATA_SIZE`.
`str(A..B)` | `String` u otro tipo de cadena | Límites inclusivos en bytes de UTF-8. Consulta [cadenas](/language/strings/).
`str(N)` | `[u8; N]` | Exactamente `N` bytes de UTF-8, sin longitud en la trama. `String` si `N` supera `STACK_LIMIT`.
`array<T>`, `vec<T>` | `Vec<T>` | `T` es un tipo entero integrado o un [primitivo registrado](/guides/custom-types/#validated-scalar-wrappers).
`array<T>(N)` | `[T; N]` | Exactamente `N` elementos, sin contador en la trama. `Vec<T>` si el array ocupa más de `STACK_LIMIT` bytes.
`array<T>(A..B)` | `Vec<T>` | Número de elementos inclusivo. `InlineArray<T, B>` con `aggressive` si cabe en `STACK_LIMIT`.
`padding(N)` | `()` | `N` bytes a cero en la trama.
`T?` | `Option<T>` | Campo opcional. Los campos opcionales numéricos usan métodos de acceso; consulta [declaraciones](/language/declarations/#optional-fields-in-rust).
Cualquier otro nombre | Tu tipo de Rust | Debe implementar `QslCustom`; consulta [tipos personalizados](/guides/custom-types/).
Con el atributo `lifetime`, las cadenas y los arrays de `u8` se toman prestados de la entrada como `&'a str` y `&'a [u8]`.

## Codificación por defecto en la trama

Sin atributos que cambien el formato:

- **Los enteros** van en little-endian con su ancho nativo. Un `u16(0..1023)` ocupa 2 bytes.
- **Las cadenas** son una longitud `u32` en bytes seguida de los bytes UTF-8. Un `str(N)` fijo no lleva longitud.
- **Los arrays dinámicos** son un número de elementos `u32` seguido de los elementos. Los arrays fijos no llevan contador.
- **Los campos obligatorios** siguen el orden de declaración.
- **Los campos opcionales** añaden metadatos de presencia antes de los valores. Consulta [campos opcionales](/language/optionals/).

La trama no lleva información de tipos. Los dos lados deben conocer el esquema.

## Tamaño en la trama y tamaño del objeto

Son independientes:

```qsl
&Reading[compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

En la trama son 3 + 5 + 10 = 18 bits, que se escriben como 3 bytes. En memoria, `Reading` es un struct `#[repr(C)]` con un `u8`, un `u8` y un `u16`: 4 bytes. `Reading::NATIVE_BYTES` contiene ese tamaño.
