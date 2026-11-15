---
title: Búferes y reutilización
description: Escritura en tus propios búferes, decodificación en valores existentes, el pool estático de slots RT y las tramas de salida en caché.
---

El generador elige la salida según el tamaño máximo demostrado de la trama. Hasta STACK_LIMIT inclusive usa `StackOut<N>`; un límite mayor o desconocido selecciona alloc::`Vec<u8>`. Las cabeceras transmitidas cuentan, los metadatos del objeto de salida no.

String, Vec y Box usan alloc dentro de no_std; la aplicación aporta el asignador. InlineString queda dentro del objeto si su tamaño nativo cabe en STACK_LIMIT; en caso contrario usa Box. Decide la capacidad del esquema.

El reuse unsafe preparado exige capacidad suficiente antes de llamar y conserva los buffers compatibles de String, Vec e InlineString en Box. No los amplía. El objeto inicializado y el destino sin inicializar tienen contratos distintos.

<a id="reading-into-existing-memory"></a>

## Ejemplo {#example}

```qsl
&Record[safety] { u64 id; }
```

```rust
let value = Record { id: 42 };
let mut buffer = [0u8; 64];
let used = value.serialize_checked_to(&mut buffer)?;
let mut destination = core::mem::MaybeUninit::<Record>::uninit();
let restored = Record::deserialize_checked_into(&buffer[..used], &mut destination)?;
assert_eq!(restored.id, 42);
```

[STACK_LIMIT](/language/deq/) · [String](/language/strings/) · [RT](/internals/runtime/)

## QSL_RT {#rt-slot-pool}

LocalRegions · StaticRegions · FramePool · FrameStorage. 1–64 slots; lease → Drop. PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.
