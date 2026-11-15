---
title: Declarações
description: Sintaxe de um esquema QSL - declarações, campos, atributos, modificadores, intervalos, expressões constantes e comentários.
---

Um arquivo de esquema contém uma ou mais declarações. Cada declaração vira um tipo Rust.

```qsl
&Name[attribute, attribute=value] {
    type field;
    type field|modifier=value|;
    type? optional_field;
}
```
Parte | Significado
---|---
`&Name` | Declara um tipo chamado `Name`.
`[ ... ]` | Atributos do tipo inteiro. Opcional. Veja a [referência de atributos](/reference/attributes/).
`type field;` | Um campo obrigatório.
`type? field;` | Um campo opcional. Com intervalo, o `?` vem antes dele: `u16?(0..1023) level;`.
`\|...\|` | Modificadores de um campo, escritos entre o nome e o ponto e vírgula.
## Campos opcionais em Rust {#optional-fields-in-rust}

Um campo opcional de string ou de array é um campo público `Option<T>` da struct.

Um campo opcional numérico fica guardado em um campo privado. Leia o valor com o método de mesmo nome e altere com `set_<name>`, que retorna `InvalidValue` para um valor fora do intervalo do campo. Tipos com um campo assim ganham um construtor `try_new`, que recebe todos os campos na ordem de declaração e verifica os valores:

```qsl
&Reading[safety] { u32 id; u8? level; }
```

```rust
let mut r = Reading::try_new(42, Some(7))?;
assert_eq!(r.level(), Some(7));
r.set_level(None)?;
```

## Ordem dos campos

Os campos obrigatórios precisam vir antes dos opcionais. O QSL exige isso em todos os tipos, com ou sem atributos.

```qsl
&Ok    { u32 id; u16? code; }
&Error { u16? code; u32 id; }   // erro de build: obrigatório depois de opcional
```

Os valores obrigatórios são escritos no frame na ordem de declaração. A ordem dos valores opcionais depende do [modo dos campos opcionais](/language/optionals/).

## Tags

O compilador atribui a cada campo uma tag na ordem de declaração, começando em 1. Você não pode definir as tags. A codificação padrão de campos opcionais armazena tags (veja [campos opcionais](/language/optionals/)), então reordenar os campos muda o formato.

Um tipo sem `optionals=bitmap` pode ter no máximo 64 campos.

## Intervalos

`A..B` é um intervalo inclusivo. O significado depende do tipo:

```qsl
&Limits {
    u8(1..3)          kind;    // intervalo de valores: 1, 2 ou 3
    str(1..100)       name;    // de 1 a 100 bytes de UTF-8
    str(4)            tag;     // exatamente 4 bytes, vira [u8; 4]
    array<u16>(0..16) samples; // de 0 a 16 elementos
}
```

Os métodos verificados impõem o intervalo de valores. Com `compact`, o intervalo também define quantos bits o campo ocupa. O intervalo de uma string conta bytes de UTF-8, não caracteres: `"Ада"` tem 6 bytes.

## Expressões constantes

Os limites podem ser expressões com inteiros e constantes nomeadas:

```qsl
&Frame {
    str(1..MAX_NAME) name;
    array<u8>(0..MAX_NAME * 2) payload;
}
```

As expressões aceitam `+ - * / %`, menos unário, parênteses, `sizeof(T)` para os tipos inteiros de `u8` a `i64` e constantes nomeadas. Uma constante vem de um `!define` em um [arquivo DEQ](/language/deq/) ou de uma linha `#NAME = expression;` dentro do corpo de um tipo, que a define para as linhas seguintes. Overflow e divisão por zero são erros de build.

## Comentários

`//` inicia um comentário de linha. `/* ... */` é um comentário de bloco.

```qsl
&Packet[safety] {
    u8 id;      // tipo da mensagem
    u16? a;     /* ausente na maioria dos frames */
}
```

## O que é rejeitado

O build falha com:

- um campo obrigatório depois de um opcional;
- nomes de campo ou de tipo duplicados, e nomes de campo que são palavras-chave do Rust ou começam com `__qsl_`;
- limites inválidos ou invertidos, e intervalos fora do tipo inteiro;
- atributos e modificadores desconhecidos ou repetidos;
- mais de 64 campos sem `optionals=bitmap`, ou mais de 128 campos opcionais com ele.

Um nome de tipo que não é embutido é tratado como [tipo personalizado](/guides/custom-types/) e precisa de uma implementação em Rust. Um tipo personalizado não pode ter intervalo. A lista dos tipos embutidos está em [tipos](/language/types/).
