---
title: Métodos gerados
description: Métodos e constantes que o QSL gera em cada tipo do esquema, agrupados pelo que exigem de quem chama.
qslResources: [{"label": "docs.rs", "url": "https://docs.rs/qsl/0.3.0/qsl/"}]
---

Cada declaração produz um tipo Rust com os métodos abaixo. Quais deles existem depende dos atributos do esquema. A lista completa de itens públicos dos crates de runtime está no docs.rs.

## Verificados: seguros com dados não confiáveis

Gerados só quando o tipo tem `[safety]`:
Método | Assinatura (simplificada) | Observações
---|---|---
`deserialize_checked` | `fn(&[u8]) -> QslResult<Self>` | Valida enquanto decodifica. Com `lifetime`, a entrada é `&'a [u8]`.
`deserialize_checked_into` | `fn(&[u8], &mut MaybeUninit<Self>) -> QslResult<&mut Self>` | Decodifica na sua memória. Em caso de erro, o destino continua não inicializado.
Gerados para todo tipo:
Método | Assinatura (simplificada) | Observações
---|---|---
`validate` | `fn(&[u8]) -> QslResult<()>` | Verifica um frame inteiro sem montar um valor. Não é necessário antes de `deserialize_checked`.
`validate_values` | `fn(&self) -> QslResult<()>` | Verifica intervalos, comprimentos, UTF-8 das strings fixas e o teto `MAX_DATA_SIZE`.
`serialize_checked` | `fn(&self) -> QslResult<`Vec<u8>`>` | Verifica o valor e depois codifica.
`serialize_checked_to` | `fn(&self, &mut [u8]) -> QslResult<usize>` | Codifica em uma slice de pelo menos `serialized_size()` bytes e retorna os bytes escritos.
Tipos com um campo opcional numérico também ganham:
Método | Assinatura (simplificada) | Observações
---|---|---
`try_new` | `fn(field, ...) -> QslResult<Self>` | Recebe todos os campos na ordem de declaração, os opcionais como `Option<T>`, e verifica os valores.
`name` | `fn(&self) -> Option<T>` | Lê o campo opcional `name`.
`set_name` | `fn(&mut self, Option<T>) -> QslResult<()>` | Altera o campo. `InvalidValue` se o valor estiver fora do intervalo.
Veja [campos opcionais em Rust](/language/declarations/#optional-fields-in-rust).

## Confiáveis: quem chama garante a validade

São métodos do trait `qsl::Serializable`. Tipos com `lifetime` implementam `qsl::BorrowSerializable<'a>` no lugar dele, e nesse trait os decodificadores se chamam `deserialize_borrowed`, `deserialize_borrowed_into` e `deserialize_borrowed_unsafe`. Nos dois traits, todos os métodos, exceto `serialized_size` e `get_size`, são `unsafe`.
Método | O que quem chama garante
---|---
`serialize`, `serialize_to`, `serialize_into_unsafe`, `serialize_unsafe` | O valor é válido para o esquema; o destino tem `serialized_size()` bytes graváveis.
`deserialize`, `deserialize_unsafe` | A entrada é um único frame completo e válido.
`deserialize_into(ptr, len, out)` | Frame válido; `out` é gravável, alinhado e não se sobrepõe à entrada. O valor antigo em `out` não é descartado.
`deserialize_reuse(input, &mut value)` | Frame válido; todo buffer de string reutilizado tem capacidade suficiente. Mantém os buffers quando o layout permite.
`deserialize_prepared_into(ptr, len, &mut value)` | O mesmo, com ponteiro bruto e comprimento.
Métodos seguros do trait:
Método | Retorna
---|---
`serialized_size(&self)` | O espaço a fornecer para este valor: o comprimento exato do frame, ou um limite superior para valores com strings comprimidas.
`get_size()` | O menor comprimento de frame possível.
## Especializados
Método | Quando
---|---
`serialize()` | Com pool RT: seguro, verifica o valor, espera um slot livre e retorna um lease. Em um tipo com empacotamento `qps` ativo e sem pool RT: seguro, retorna `QuantizedOut`. Veja [pool de slots RT](/guides/memory/#rt-slot-pool).
`serialize_quantized_to` | `qps` ativo. Verifica os valores e a capacidade e depois roda o kernel de empacotamento.
`quant_write_prepared`, `quant_read_padded` | `qps` ativo. Métodos de `qsl::payload_quant::QuantizedRecord`. `unsafe`: quem chama garante intervalos, padding e memória alinhada e sem sobreposição.
`serialize_qps_to`, `deserialize_qps_into`, `deserialize_qps` | Adaptadores de `ByteQps`. Só para tipos com `[safety]` e `qps` e sem strings nem arrays.
`serialize_cached` | `cache=N` ou `loyal_cache`. Escreve em um `CachedFrame`.
`deserialize_transmutate_copy`, `deserialize_overlay` | [`transmutate`](/reference/attributes/#transmutate). Cada um tem uma variante `_unchecked` que pula as verificações.
## Constantes
Constante | Significado
---|---
`SCHEMA_HASH_STRICT: u128` | Hash que inclui os nomes. Veja [identidade do esquema](/guides/schema-identity/).
`SCHEMA_HASH_LIGHT: u128` | Hash sem os nomes.
`SCHEMA_HASHES` | Os dois, com algoritmo, versão e completude. Também `<T as qsl::SchemaIdentity>::SCHEMA_HASHES`.
`MAX_DATA_SIZE` | O teto usado pelos métodos verificados.
`NATIVE_BYTES`, `NATIVE_ALIGNMENT` | `size_of` e `align_of` da struct gerada.
`WIRE_MAX_BYTES: Option<usize>` | O tamanho máximo do frame, ou `None` se não houver limite.
`CACHE_BYTES` | Capacidade para `serialize_cached`. Só com `cache=N` ou `loyal_cache`.
`NATIVE_WIRE_COMPATIBLE` | Se o layout nativo é igual ao layout serializado. Só com `transmutate`.
`QUANT_PADDING_BYTES` | Bytes extras legíveis exigidos pelas leituras preparadas de `qps`. Atualmente `0`.
`REUSES_STORAGE` é uma constante do trait `Serializable`: `<T as qsl::Serializable>::REUSES_STORAGE` é `true` quando `deserialize_reuse` atualiza os buffers no lugar.

O gerador escolhe a saída pelo tamanho máximo comprovado do quadro. Até STACK_LIMIT inclusive usa `StackOut<N>`; um limite maior ou desconhecido seleciona alloc::`Vec<u8>`. Cabeçalhos transmitidos contam; metadados do objeto de saída não.
