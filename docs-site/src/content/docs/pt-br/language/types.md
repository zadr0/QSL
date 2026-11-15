---
title: Tipos
description: Os tipos embutidos do QSL, os tipos Rust que eles geram e a codificação binária padrão de cada um.
---

## Tipos embutidos
Esquema | Tipo Rust | Observações
---|---|---
`u8` `u16` `u32` `u64` `i8` `i16` `i32` `i64` | O mesmo primitivo |
`T(A..B)` para um inteiro `T` | O mesmo primitivo | O valor precisa estar no intervalo. `compact` e `qps` podem reduzi-lo no frame.
`flags(N)` | `u8`, `u16`, `u32` ou `u64`, conforme `N` | `N` vai de 1 a 64. Os bits acima de `N` devem ser zero.
`str`, `string`, `String` | `String` | Comprimento variável, limitado só por `MAX_DATA_SIZE`.
`str(A..B)` | `String` ou outro tipo de string | Limites inclusivos em bytes de UTF-8. Veja [strings](/language/strings/).
`str(N)` | `[u8; N]` | Exatamente `N` bytes de UTF-8, sem comprimento no frame. `String` se `N` passar de `STACK_LIMIT`.
`array<T>`, `vec<T>` | `Vec<T>` | `T` é um tipo inteiro embutido ou um [primitivo registrado](/guides/custom-types/#validated-scalar-wrappers).
`array<T>(N)` | `[T; N]` | Exatamente `N` elementos, sem contagem no frame. `Vec<T>` se o array passar de `STACK_LIMIT` bytes.
`array<T>(A..B)` | `Vec<T>` | Contagem de elementos inclusiva. `InlineArray<T, B>` com `aggressive`, se couber em `STACK_LIMIT`.
`padding(N)` | `()` | `N` bytes zero no frame.
`T?` | `Option<T>` | Campo opcional. Campos opcionais numéricos usam métodos de acesso; veja [declarações](/language/declarations/#optional-fields-in-rust).
Qualquer outro nome | O seu tipo Rust | Precisa implementar `QslCustom`; veja [tipos personalizados](/guides/custom-types/).
Com o atributo `lifetime`, strings e arrays de `u8` são emprestados da entrada como `&'a str` e `&'a [u8]`.

## Codificação binária padrão

Sem atributos que mudem o formato:

- **Inteiros** são little-endian na largura nativa. Um `u16(0..1023)` ocupa 2 bytes.
- **Strings** são um comprimento `u32` em bytes seguido dos bytes UTF-8. Uma `str(N)` fixa não tem comprimento.
- **Arrays dinâmicos** são uma contagem de elementos `u32` seguida dos elementos. Arrays fixos não têm contagem.
- **Campos obrigatórios** seguem a ordem de declaração.
- **Campos opcionais** adicionam metadados de presença antes dos valores. Veja [campos opcionais](/language/optionals/).

O frame não carrega informação de tipo. Os dois lados precisam conhecer o esquema.

## Tamanho serializado e tamanho do objeto

Um não depende do outro:

```qsl
&Reading[compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

No frame, isso dá 3 + 5 + 10 = 18 bits, escritos como 3 bytes. Na memória, `Reading` é uma struct `#[repr(C)]` com um `u8`, um `u8` e um `u16`: 4 bytes. `Reading::NATIVE_BYTES` guarda esse tamanho.
