---
title: Configuración DEQ
description: Los archivos DEQ contienen constantes tipadas y ajustes de toda la compilación como MAX_DATA_SIZE, STACK_LIMIT y el pool RT.
---

Un archivo `.deq` contiene dos clases de directivas: constantes tipadas que los esquemas pueden usar en los límites y ajustes para la generación de código. Pasa los archivos DEQ como segundo argumento de `build_qsl_files`:

```rust title="build.rs"
qsl::build_qsl_files(&["schemas/packet.qsl"], &["schemas/config.deq"], out)?;
```

## Constantes

```deq title="config.deq"
!define MAX_NAME: u16 = 100;
```

El tipo es uno de `u8` a `u64`, de `i8` a `i64`, `usize` o `isize`, y el valor tiene que caber en él. Las expresiones usan los mismos operadores que los esquemas, incluido `sizeof(T)` para un tipo entero. Una constante posterior puede referirse a una anterior como `!NAME`:

```deq title="config.deq"
!define HEADER: usize = sizeof(u64) + 2;
!define FRAME: usize = !HEADER * 4;
```

Los esquemas usan la constante por su nombre:

```qsl
&User[safety] { u64 id; str(1..MAX_NAME) name; }
```

## Ajustes

```deq title="config.deq"
!config MAX_DATA_SIZE = 65535;
!config USE_HASH = xxHash;
```
Ajuste | Por defecto | Significado
---|---|---
`MAX_DATA_SIZE` | `65535` | Límite superior en bytes para una trama, para los datos comprimidos una vez restaurados y para el tamaño total de los valores decodificados. De 1 a `isize::MAX`.
`STACK_LIMIT` | `256` | Presupuesto de capacidad inline, en bytes, de cada búfer de cadena o de array. Con límites mayores se usa `String` o `Vec`. De `0` a `u32::MAX`.
`USE_HASH` | `xxHash` | Algoritmo del hash de esquema. `xxHash` es XXH3-64 y `xxHash128` es XXH3-128. También se aceptan `XXH3_64` y `XXH3_128`.
`QSL_RT` | `false` | `true` genera un pool estático de slots para los tipos con un tamaño máximo de trama finito. Consulta [búferes y reutilización](/guides/memory/#rt-slot-pool).
`QSL_RT_DEFAULT` | `16` | Slots por tipo, de 1 a 64. Cuenta slots, no bytes.
Los ajustes se aplican a toda la compilación. Definir el mismo ajuste dos veces, aunque sea en archivos distintos, es un error. Las directivas desconocidas y los nombres de ajuste desconocidos también son errores.

Los archivos DEQ no contienen rutas de archivo. `build.rs` decide qué archivos se compilan.
