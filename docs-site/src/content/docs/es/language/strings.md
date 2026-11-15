---
title: Cadenas y propiedad
description: Los tipos de Rust en los que se puede convertir una cadena de QSL, cuándo se elige cada uno y qué hace la decodificación con la memoria.
---

Las longitudes UTF-8 son bytes: «Яa» ocupa 3 B. String posee memoria dinámica, &str toma prestada la entrada viva y StackString/InlineString tienen capacidad acotada. QSL-LZ comprime solo cuando reduce el tamaño; en otro caso conserva raw. El gráfico muestra distribución, literales y referencias hacia atrás.

## Almacenamiento y longitud

InlineString conserva su códec dentro del objeto o en Box en heap; las cadenas grandes se restauran directamente al destino. StackString e InlineArray contienen su capacidad limitada. aggressive[shift=true] puede elegir una longitud NonZero, distinguiendo None de Some(empty).

## QSL-LZ {#compressed-strings}

Se verifican cada token expandido, distancia de referencia, UTF-8 y presupuesto acumulado. La longitud almacenada no autoriza una salida restaurada mayor. Las longitudes son bytes UTF-8, no caracteres Unicode.
[Arrays](/language/arrays/) · [Limits](/reference/limits/) · [Memory](/guides/manual-layout/) · [QPS](/guides/smaller-frames/)

no_std excluye std, no alloc. qsl-core siempre usa #![no_std]; la función alloc, activa por defecto, ofrece alloc String/Vec, SerOut y restauración en heap. La aplicación aporta el asignador; QSL no declara uno global. Desactiva las funciones predeterminadas para usar solo buffers del llamador.

El generador elige la salida según el tamaño máximo demostrado de la trama. Hasta STACK_LIMIT inclusive usa `StackOut<N>`; un límite mayor o desconocido selecciona alloc::`Vec<u8>`. Las cabeceras transmitidas cuentan, los metadatos del objeto de salida no.
