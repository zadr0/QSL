---
title: 'Runtime y búferes'
description: 'SerOut y los búferes preparados permiten controlar asignaciones y reutilización. QSL_RT crea ranuras estáticas solo para esquemas acotados elegibles: 16 por esquema por defecto. Al agotarse devuelve un error sin una asignación oculta. El runtime actual no incluye sockets ni regiones específicos de la plataforma.'
---

no_std excluye std, no alloc. qsl-core siempre usa #![no_std]; la función alloc, activa por defecto, ofrece alloc String/Vec, SerOut y restauración en heap. La aplicación aporta el asignador; QSL no declara uno global. Desactiva las funciones predeterminadas para usar solo buffers del llamador.

Regiones estáticas no_std que implementan FramePool/FrameStorage. Un pool lleno devuelve PoolExhausted inmediatamente.
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString conserva su códec dentro del objeto o en Box en heap; las cadenas grandes se restauran directamente al destino. StackString e InlineArray contienen su capacidad limitada. aggressive[shift=true] puede elegir una longitud NonZero, distinguiendo None de Some(empty).

Los scripts de compilación y las pruebas pueden usar std sin incluirlo en el runtime de destino. Esquemas reales generados se ejecutan sin asignador; otra biblioteca no_std comprueba contenedores alloc. Ambos perfiles del núcleo y qsl-rt también compilan para wasm32-wasip2.

[Memory](/guides/memory/) · [Safety](/guides/safety/)

El generador elige la salida según el tamaño máximo demostrado de la trama. Hasta STACK_LIMIT inclusive usa `StackOut<N>`; un límite mayor o desconocido selecciona alloc::`Vec<u8>`. Las cabeceras transmitidas cuentan, los metadatos del objeto de salida no.
