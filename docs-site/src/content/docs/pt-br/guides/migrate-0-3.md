---
title: Migração para a 0.3.0
description: Mudanças no qsl 0.3.0 - a separação do qsl-core e o conjunto reduzido de representações de string.
qslResources: [{"label": "notas da 0.2.0", "url": "https://github.com/zadr0/QSL/blob/main/docs/migration-0.2.0.en.md"}]
---

O QSL 0.3.0 move o runtime compartilhado para um crate novo, o `qsl-core` 0.1.0, e remove APIs antigas de string. O formato binário do bitmap (v2) e das strings comuns não muda.

## Atualize as versões juntas

```toml title="Cargo.toml"
[dependencies]
qsl = "=0.4.0"

[build-dependencies]
qsl = "=0.4.0"
```

Se você depende diretamente dos crates internos, atualize também `qsl-lexer` e `qsl-interpreter` para a 0.3.0. As macros de atributo continuam no `qsl-macros-impl` 0.1.0.

## Divisão dos crates
Crate | Contém | Depende de
---|---|---
`qsl-core` | Erros, modelo de esquema, análise compartilhada, I/O, contêineres UTF-8, validação, buffers, codecs de baixo nível | Nenhum outro crate do QSL
`qsl-lexer` | Lexer, parsers de QSL e DEQ, carregamento de arquivos, expansão de templates, builder de esquemas | `qsl-core`
`qsl-interpreter` | Geração de código Rust | `qsl-lexer`
`qsl` | Fachada, macros, pontos de entrada compatíveis | Todos os anteriores e `qsl-macros-impl`
Os caminhos de módulo antigos continuam funcionando como reexportações. `QslError` e `QslCustom` são um único tipo cada, seja qual for o caminho de importação.

## Strings

Restam quatro representações:
Escolha | Tipo Rust
---|---
String comum | `String`
`aggressive`, limite superior dentro de `STACK_LIMIT` | `StackString<N>`
`decompress=true`, limite dentro de `STACK_LIMIT` | `InlineString<N>`
`lifetime` | `&str`
Removidos: `binary_opt=guard`, `binary_opt=true`, `Utf8Guard`, `Utf8CopyGuard` e o modo de byte string. Esquemas que os usam não compilam; troque cada campo por uma das quatro representações acima.

*New* e *reuse* são escolhas de método, não tipos. Antes de decodificar em uma `String` existente, dê a ela capacidade suficiente. Veja [strings](/language/strings/) e [buffers e reutilização](/guides/memory/#reading-into-existing-memory).

## Métodos de trait agora são unsafe

`Serializable::serialize` e `BorrowSerializable::serialize` agora são `unsafe`: quem chama garante que o valor é válido para o esquema. Use `serialize_checked` no lugar deles. O `serialize()` gerado em tipos com pool RT ou com empacotamento `qps` ativo é outro método. Ele continua seguro e retorna um `Result`.

## Da 0.1 para a 0.2

Se você está vindo da 0.1, leia antes as notas da 0.2.0. Os pontos principais: o formato do bitmap mudou (v2), a feature `native` do Cargo e as regiões de sockets e IPC do Windows foram removidas, e `serialize` agora retorna `Vec<u8>`.
