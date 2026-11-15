---
title: Errores
description: Las variantes de QslError y cuándo se devuelve cada una.
---

Los métodos generados devuelven `qsl::QslResult<T>`, un alias de `Result<T, QslError>`. `QslError` tiene 11 variantes e implementa `std::error::Error`. Su salida de `Display` es `QSL error: <Variant>`, por ejemplo `QSL error: InvalidUtf8`.
Variante | Se devuelve cuando
---|---
`UnexpectedEof` | La entrada termina antes que la trama.
`BufferTooSmall` | Un búfer que pasaste es demasiado pequeño para la salida.
`LengthOverflow` | Un cálculo de longitud desbordaría.
`InvalidWidth` | Un ancho o un quantum no está soportado.
`InvalidValue` | Un valor, una longitud o un campo de metadatos está fuera de su dominio permitido.
`InvalidUtf8` | Los bytes de una cadena no son UTF-8 válido.
`InvalidSchema` | El esquema no es válido o la operación no se le aplica. También es el resultado por defecto de los validadores de `QslCustom`.
`InvalidPadding` | Los bits o bytes de relleno no son cero.
`PoolExhausted` | Todos los slots RT están ocupados. Solo lo devuelve `try_acquire`, que no espera.
`SizeLimitExceeded` | La trama o los datos decodificados superan `MAX_DATA_SIZE`.
`AllocationFailed` | Falló una reserva de memoria falible.
Las operaciones con archivos y sockets también pueden devolver `std::io::Error`.

Una validación correcta significa que los bytes encajan en la forma del esquema. No dice quién los envió ni con qué esquema se escribieron.
