---
title: Declaraciones
description: Sintaxis de un esquema QSL (declaraciones, campos, atributos, modificadores, rangos, expresiones constantes y comentarios).
---

Un archivo de esquema contiene una o varias declaraciones. Cada declaración se convierte en un tipo de Rust.

```qsl
&Name[attribute, attribute=value] {
    type field;
    type field|modifier=value|;
    type? optional_field;
}
```
Parte | Significado
---|---
`&Name` | Declara un tipo llamado `Name`.
`[ ... ]` | Atributos de todo el tipo. Son opcionales. Consulta la [referencia de atributos](/reference/attributes/).
`type field;` | Un campo obligatorio.
`type? field;` | Un campo opcional. Si tiene rango, el `?` va antes: `u16?(0..1023) level;`.
`\|...\|` | Modificadores de un solo campo, escritos entre el nombre y el punto y coma.
## Campos opcionales en Rust {#optional-fields-in-rust}

Un campo opcional de cadena o de array es un campo público `Option<T>` del struct.

Un campo opcional numérico se guarda en un campo privado. Se lee con un método del mismo nombre y se cambia con `set_<name>`, que devuelve `InvalidValue` si el valor está fuera del rango del campo. Los tipos con un campo así tienen un constructor `try_new` que recibe todos los campos en orden de declaración y comprueba los valores:

```qsl
&Reading[safety] { u32 id; u8? level; }
```

```rust
let mut r = Reading::try_new(42, Some(7))?;
assert_eq!(r.level(), Some(7));
r.set_level(None)?;
```

## Orden de los campos

Los campos obligatorios deben ir antes que los opcionales. QSL lo exige en todos los tipos, con atributos o sin ellos.

```qsl
&Ok    { u32 id; u16? code; }
&Error { u16? code; u32 id; }   // error de compilación: obligatorio tras opcional
```

Los valores obligatorios se escriben en la trama en orden de declaración. El orden de los valores opcionales depende del [modo de campos opcionales](/language/optionals/).

## Etiquetas

El compilador asigna a cada campo una etiqueta en orden de declaración, empezando por 1. No puedes fijar las etiquetas tú mismo. La codificación por defecto de los campos opcionales guarda etiquetas (consulta [campos opcionales](/language/optionals/)), así que reordenar los campos cambia el formato.

Un tipo sin `optionals=bitmap` puede tener como máximo 64 campos.

## Rangos

`A..B` es un rango inclusivo. Su significado depende del tipo:

```qsl
&Limits {
    u8(1..3)          kind;    // rango de valores: 1, 2 o 3
    str(1..100)       name;    // de 1 a 100 bytes de UTF-8
    str(4)            tag;     // exactamente 4 bytes, se convierte en [u8; 4]
    array<u16>(0..16) samples; // de 0 a 16 elementos
}
```

Los métodos con comprobación hacen cumplir el rango de valores. Con `compact`, el rango también fija cuántos bits ocupa el campo. El rango de una cadena cuenta bytes de UTF-8, no caracteres: `"Ада"` ocupa 6 bytes.

## Expresiones constantes

Los límites pueden ser expresiones con enteros y constantes con nombre:

```qsl
&Frame {
    str(1..MAX_NAME) name;
    array<u8>(0..MAX_NAME * 2) payload;
}
```

Las expresiones admiten `+ - * / %`, el menos unario, paréntesis, `sizeof(T)` para los tipos enteros de `u8` a `i64` y constantes con nombre. Una constante viene de `!define` en un [archivo DEQ](/language/deq/) o de una línea `#NAME = expression;` dentro del cuerpo de un tipo, que la define para las líneas siguientes. El desbordamiento y la división por cero son errores de compilación.

## Comentarios

`//` inicia un comentario de línea. `/* ... */` es un comentario de bloque.

```qsl
&Packet[safety] {
    u8 id;      // message kind
    u16? a;     /* absent in most frames */
}
```

## Qué se rechaza

La compilación falla si hay:

- un campo obligatorio después de uno opcional;
- nombres de campo o de tipo duplicados, y nombres de campo que son palabras clave de Rust o empiezan por `__qsl_`;
- límites no válidos o invertidos, y rangos que se salen del tipo entero;
- atributos y modificadores desconocidos o repetidos;
- más de 64 campos sin `optionals=bitmap`, o más de 128 campos opcionales con él.

Un nombre de tipo que no es de los integrados se trata como [tipo personalizado](/guides/custom-types/) y necesita una implementación en Rust. Un tipo personalizado no puede tener rango. La lista de tipos integrados está en [tipos](/language/types/).
