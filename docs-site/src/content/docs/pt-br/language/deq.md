---
title: Configuração DEQ
description: Arquivos DEQ guardam constantes tipadas e configurações que valem para o build inteiro, como MAX_DATA_SIZE, STACK_LIMIT e o pool RT.
---

Um arquivo `.deq` tem dois tipos de diretiva: constantes tipadas, que os esquemas podem usar nos limites, e configurações da geração de código. Passe os arquivos DEQ como segundo argumento de `build_qsl_files`:

```rust title="build.rs"
qsl::build_qsl_files(&["schemas/packet.qsl"], &["schemas/config.deq"], out)?;
```

## Constantes

```deq title="config.deq"
!define MAX_NAME: u16 = 100;
```

O tipo é um de `u8` a `u64`, de `i8` a `i64`, `usize` ou `isize`, e o valor precisa caber nele. As expressões usam os mesmos operadores dos esquemas, inclusive `sizeof(T)` para um tipo inteiro. Uma constante pode se referir a outra definida antes como `!NAME`:

```deq title="config.deq"
!define HEADER: usize = sizeof(u64) + 2;
!define FRAME: usize = !HEADER * 4;
```

Os esquemas usam a constante pelo nome:

```qsl
&User[safety] { u64 id; str(1..MAX_NAME) name; }
```

## Configurações

```deq title="config.deq"
!config MAX_DATA_SIZE = 65535;
!config USE_HASH = xxHash;
```
Configuração | Padrão | Significado
---|---|---
`MAX_DATA_SIZE` | `65535` | Limite superior, em bytes, para um frame, para os dados comprimidos depois de restaurados e para o tamanho total dos valores decodificados. De 1 a `isize::MAX`.
`STACK_LIMIT` | `256` | Teto de capacidade inline, em bytes, por buffer de string ou de array. Limites maiores caem para `String` ou `Vec`. De `0` a `u32::MAX`.
`USE_HASH` | `xxHash` | Algoritmo do hash do esquema. `xxHash` é XXH3-64, `xxHash128` é XXH3-128. `XXH3_64` e `XXH3_128` também são aceitos.
`QSL_RT` | `false` | `true` gera um pool estático de slots para tipos com tamanho máximo de frame finito. Veja [buffers e reutilização](/guides/memory/#rt-slot-pool).
`QSL_RT_DEFAULT` | `16` | Slots por tipo, de 1 a 64. Conta slots, não bytes.
As configurações valem para o build inteiro. Definir o mesmo nome duas vezes, mesmo em arquivos diferentes, é erro. Diretivas e nomes de configuração desconhecidos também são erro.

Arquivos DEQ não contêm caminhos de arquivo. Quem decide quais arquivos são compilados é o `build.rs`.
