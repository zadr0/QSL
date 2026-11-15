---
title: Validação de entrada
description: O que o [safety] gera, o que os métodos verificados conferem e o que continua sendo responsabilidade de quem chama.
---

Bytes vindos de um socket, de um arquivo ou de outro processo não são confiáveis. Decodifique-os com os métodos verificados, que só são gerados para tipos com o atributo `safety`.

```qsl
&Packet[safety] {
    u32 id;
    str(1..100) text;
}
```

```rust
let bytes = packet.serialize_checked()?;
let decoded = Packet::deserialize_checked(&bytes)?;

// Ou decodifique em memória que você já tem:
let mut slot = std::mem::MaybeUninit::uninit();
let decoded: &mut Packet = Packet::deserialize_checked_into(&bytes, &mut slot)?;
```

## O que uma decodificação verificada confere

Antes de retornar um valor, o decodificador gerado verifica que:

- o frame tem pelo menos o comprimento do menor frame válido, contém todos os campos e termina onde o esquema diz;
- todo número está dentro do intervalo declarado;
- todo comprimento de string e de array está dentro dos limites;
- as strings são UTF-8 válido;
- os metadados dos campos opcionais são coerentes com o esquema;
- os bits e bytes de padding são zero;
- as strings comprimidas não restauram mais bytes que o limite delas;
- o tamanho total dos valores decodificados fica dentro de `MAX_DATA_SIZE`.

Validação e decodificação acontecem em uma única passada. Você não precisa chamar `validate` antes. Se a decodificação falhar, `deserialize_checked_into` deixa `slot` não inicializado.

## Limites de tamanho

`MAX_DATA_SIZE` (padrão de 65535 bytes) limita o frame de entrada e o payload decodificado total. Os comprimentos lidos do frame são verificados contra os limites deles e contra esse teto antes de qualquer reserva de memória. Aumente ou diminua o valor em um [arquivo DEQ](/language/deq/).

O exemplo safety do repositório passa três frames inválidos para o `validate` deste tipo:

```qsl
&Bounded[safety] { u8(1..3) kind; str(1..100) name; }
```
Entrada | Resultado
---|---
`09 01 00 00 00 61` | `InvalidValue`: `kind` é 9, fora de `1..3`.
`01 ff ff ff ff` | `UnexpectedEof`: 5 bytes é menos que o menor frame válido, de 6 bytes.
`01 01 00 00 00 ff` | `InvalidUtf8`: `ff` não é UTF-8.
Um comprimento forjado em um frame de tamanho válido, como `01 ff ff ff ff 61`, falha com `InvalidValue`, porque o comprimento é comparado com o limite `100` antes de qualquer byte ser copiado.

## Codificação verificada

`serialize_checked` verifica o valor contra o esquema (intervalos, comprimentos, UTF-8 das strings de tamanho fixo, `MAX_DATA_SIZE`) e retorna um `Vec<u8>` novo. `serialize_checked_to` escreve em uma slice fornecida por você e retorna o número de bytes escritos, ou `BufferTooSmall`. Se a verificação final de tamanho falhar, a slice pode já estar parcialmente escrita. Os dois são gerados para todo tipo, com ou sem `[safety]`.

## Métodos sem verificação

`[safety]` adiciona métodos verificados. Ele não remove os sem verificação. Os métodos do trait `qsl::Serializable`, como `deserialize_into`, e os leitores com ponteiro bruto partem do princípio de que:

- o frame está completo e é válido para este esquema;
- a memória de destino é grande o bastante, está alinhada corretamente e não se sobrepõe à entrada;
- as entradas emprestadas vivem mais que o resultado.

Use esses métodos só com frames que você mesmo produziu ou já validou. Eles são `unsafe`, inclusive `Serializable::serialize` e `BorrowSerializable::serialize` desde a 0.3.0. O `serialize()` gerado em tipos com pool RT ou com empacotamento `qps` ativo é outro método, seguro, que retorna um `Result`.

## Limites da validação

- A validação prova que os bytes têm um formato válido para o esquema. Ela não prova qual esquema os produziu: dois esquemas com o mesmo layout aceitam os mesmos bytes. Use os [hashes do esquema](/guides/schema-identity/) ou um discriminador próprio em um handshake.
- Ela não autentica o remetente.
- Um [tipo personalizado](/guides/custom-types/) só é tão seguro quanto a implementação de `qsl_validate` dele. A implementação padrão rejeita qualquer entrada.

O gerador escolhe a saída pelo tamanho máximo comprovado do quadro. Até STACK_LIMIT inclusive usa `StackOut<N>`; um limite maior ou desconhecido seleciona alloc::`Vec<u8>`. Cabeçalhos transmitidos contam; metadados do objeto de saída não.
