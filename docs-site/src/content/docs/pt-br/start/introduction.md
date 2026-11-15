---
title: O que é o Quick Schema Language (QSL)
description: O Quick Schema Language (QSL) é uma linguagem de esquemas e um gerador de codecs binários para Rust. O que ele faz, o que fica com você e como os crates se dividem.
---

O Quick Schema Language (QSL) é uma linguagem para descrever mensagens binárias e um gerador de código que transforma essas descrições em Rust. Você escreve um esquema com tipos de campo, intervalos de valores e limites de comprimento. Durante o `cargo build`, o QSL lê o esquema e gera structs e métodos que leem e escrevem bytes. O programa não faz o parsing do esquema em tempo de execução.

A serialização binária transforma um valor em bytes; a desserialização lê esses bytes de volta como um valor. No QSL, o esquema define os campos e o código Rust gerado faz as duas conversões. Os participantes precisam usar esquemas e atributos iguais. Comece pelo [início rápido de serialização binária em Rust](/start/quickstart/), depois veja o [formato binário](/internals/wire-format/) e os [exemplos de empacotamento de bits](/guides/smaller-frames/).

```qsl
&Reading[safety,compact] {
    u8(0..7)     kind;
    u8(0..31)    channel;
    u16(0..1023) value;
}
```

A partir desse esquema, o QSL gera uma struct `Reading` com campos `u8`, `u8` e `u16`, além de métodos que a codificam, decodificam e validam. Com `compact`, os três valores ocupam 3 + 5 + 10 = 18 bits, e o frame fica com 3 bytes em vez de 4.

## O que você controla

- **Layout serializado.** A largura dos inteiros no frame e a forma como o frame indica quais campos opcionais estão presentes.
- **Memória.** Se uma string vira `String`, um `&str` emprestado ou um buffer inline, e se a decodificação reaproveita um objeto existente.
- **Validação.** `[safety]` gera métodos de decodificação verificada. Eles verificam limites, intervalos, UTF-8 e um teto de tamanho total antes de retornar um valor.

## O que fica na sua aplicação

O QSL define como os dados são representados. Fora o `qsl::RawSocket`, um wrapper simples que envia frames prefixados pelo comprimento por TCP, ele não cuida de transporte e não tem TLS, roteamento nem negociação de versão do esquema. Os dois lados de uma troca precisam do mesmo esquema e dos mesmos atributos. Se você mudar o formato, leve a mudança a todas as pontas de uma vez. O QSL não detecta qual formato um frame usa.

Todo tipo gerado traz os hashes do esquema como constantes (veja [identidade do esquema](/guides/schema-identity/)). Você pode enviá-los no seu próprio handshake. O QSL não faz isso por você.

## Crates
Crate | Versão | Papel
---|---|---
`qsl` | 0.3.0 | O crate de que você depende. Reexporta o runtime e fornece `build_qsl_files` e as macros de atributo.
`qsl-core` | 0.1.0 | Runtime compartilhado: erros, cursores de I/O, contêineres de string, validação, buffers.
`qsl-lexer` | 0.3.0 | Parser de esquemas `.qsl` e de arquivos de configuração `.deq`, builder de esquemas.
`qsl-interpreter` | 0.3.0 | Gera código-fonte Rust a partir de esquemas validados.
`qsl-macros-impl` | 0.1.0 | Os atributos `qsl_macro` e `schem_macro`, reexportados pelo `qsl`.
A maioria dos projetos só adiciona o `qsl`, uma vez em `[dependencies]` e outra em `[build-dependencies]`. A página [Arquitetura](/internals/architecture/) mostra como os crates dependem uns dos outros.

## Status

O QSL 0.3.0 é desenvolvido e testado no Windows 11 x86-64 com Rust 1.98, que também é a versão mínima suportada. Nenhum atributo vem ativado por padrão. Um atributo muda o código gerado, mas nenhum deles garante ganho de velocidade. A  lista o que foi medido e em que condições.

Próximo passo: [Início rápido](/start/quickstart/).
