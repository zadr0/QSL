---
title: 'Arrays y ventanas de bits'
description: 'La longitud de un array cuenta elementos; la de una cadena cuenta bytes UTF-8. InlineArray reserva capacidad dentro de la estructura y Vec posee una asignación separada. STACK_LIMIT limita estrictamente el almacenamiento interno. Los descriptores de Vec no son el contenido del mensaje.'
---

La longitud de un array cuenta elementos; la de una cadena, bytes UTF-8. `array<u16>(0..16)` permite de cero a dieciséis elementos y `array<u16>(16)` exige exactamente dieciséis.

Vec posee un buffer separado mediante alloc. InlineArray contiene capacidad limitada dentro del objeto; los arrays fijos pueden usar `[T;N]`. STACK_LIMIT limita el almacenamiento integrado. El descriptor de Vec no es el payload.

$$
B_{\mathrm{array}}=H(n)+\sum_{i=1}^{n} w_i.
$$

H es la metadata de longitud en bits, n el número de elementos y w el ancho del códec de cada elemento. La alineación se cuenta aparte. Una longitud no permite leer fuera del frame. Se admiten hasta 32 niveles anidados; los dominios y longitudes se validan por separado. El registro depende de la posición de bits; no siempre es de 128 bits.

`aggressive[shift=true]` explícito puede elegir una longitud NonZero para InlineArray. El valor predeterminado es u32. Esta optimización física conserva el wire y distingue None de Some(empty).
[Read windows](/internals/optimizer/) · [Memory](/guides/memory/)

no_std excluye std, no alloc. qsl-core siempre usa #![no_std]; la función alloc, activa por defecto, ofrece alloc String/Vec, SerOut y restauración en heap. La aplicación aporta el asignador; QSL no declara uno global. Desactiva las funciones predeterminadas para usar solo buffers del llamador.
