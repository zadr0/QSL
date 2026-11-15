---
title: Atributos
description: Todos os atributos de tipo e modificadores de campo do QSL, o que cada um gera e se muda o formato binário.
---

Os atributos de tipo ficam entre colchetes depois do nome: `&Packet[safety,compact]`. Os modificadores de campo ficam entre barras verticais depois do nome do campo: `str(1..100) name;`. Todos são opcionais, e cada um pode aparecer uma vez.

A coluna **Formato** indica se o atributo muda os bytes e, portanto, o [hash do esquema](/guides/schema-identity/). Se mudar, as duas pontas precisam usar a mesma configuração.

## Resumo
Atributo | Onde | Formato | Efeito
---|---|---|---
[`safety`](#safety) | tipo | não | Gera métodos de decodificação verificada.
[`compact`](#compact) | tipo | sim | Armazena valores com intervalo no mínimo de bits.
[`alignment`](#alignment) | tipo | com `compact` | Começa cada campo compacto em uma fronteira de byte.
[`optionals=bitmap`](#optionalsbitmap) | tipo | sim | Máscara de presença sobre os campos opcionais.
[`predict_seq`](#predict_seq) | tipo | sim | Presença deduzida do comprimento do payload.
[`opt_table`](#opt_table) | tipo | só `legacy` | Tabela estática de leitores de campos opcionais.
[`qps`, `quant`](#qps-quant) | tipo | sim | Empacotamento reversível de campos numéricos.
[`aggressive`](#aggressive) | tipo | não | Armazenamento inline para strings e arrays com limite.
[`lifetime`](#lifetime) | tipo | não | Empresta strings e bytes da entrada.
[`transmutate`](#transmutate) | tipo | não | Layout nativo igual ao layout serializado.
[`inline`](#inline) | tipo | não | Política de `#[inline]` para os métodos gerados.
[`slots=N`](#slotsn) | tipo | não | Número de slots RT para este tipo.
[`cache=N`, `loyal_cache`](#cachen-loyal_cache) | tipo | não | Saída em um `CachedFrame` de quem chama.
[`state_bits=Qn`](#state_bitsqn) | tipo | sim | Formato legado de índice de presença.
[`decompress=true`](#decompresstrue) | campo | sim | Compressão LZ limitada para uma string.
[`default=N`](#defaultn) | campo | não | Gera `impl Default`.
## Atributos de tipo

### safety {#safety}

Gera `deserialize_checked` e `deserialize_checked_into`, que decodificam entrada não confiável. Eles verificam limites, intervalos, UTF-8, metadados dos campos opcionais, padding, descompressão e o teto total `MAX_DATA_SIZE` antes de retornar um valor. Os métodos sem verificação continuam disponíveis. `serialize_checked`, `serialize_checked_to`, `validate` e `validate_values` são gerados com ou sem `safety`. Veja [validação de entrada](/guides/safety/).

### compact {#compact}

Armazena cada inteiro com intervalo como `x - min` em `bit_length(max - min)` bits, empacotados através das fronteiras de byte. `u8(0..7)`, `u8(0..31)` e `u16(0..1023)` ocupam 3, 5 e 10 bits. Os comprimentos de strings e arrays são armazenados da mesma forma, dentro dos limites deles. Funciona com `opt_table` e `optionals=bitmap`. Não pode ser combinado com `predict_seq`, `state_bits` nem `qps`. Veja [frames menores](/guides/smaller-frames/).

### alignment {#alignment}

Com `compact`, começa cada campo em uma fronteira de byte. O frame pode ficar maior. Sem `compact`, não muda o formato. Não é alinhamento de buffers na memória. Não pode ser combinado com `qps`.

### optionals=bitmap {#optionalsbitmap}

Escreve uma máscara com um bit por campo opcional, de 0 a 16 bytes de largura, conforme a quantidade. Suporta até 128 campos opcionais. Incompatível com a máscara usada antes da 0.2.0. Não pode ser combinado com `state_bits`, `opt_table=legacy` nem `qps`. Veja [campos opcionais](/language/optionals/#bitmap).

### predict_seq {#predict_seq}

Deduz quais campos opcionais estão presentes pelo comprimento do payload. Exige que cada combinação de campos presentes resulte em um comprimento distinto. É ignorado quando `optionals=bitmap` está definido. Não pode ser combinado com `compact`. Veja [campos opcionais](/language/optionals/#length-index-predict_seq).

### opt_table {#opt_table}

`opt_table` (ou `opt_table=static`) chama os leitores dos campos opcionais por uma tabela estática em vez de um `match`. Mesmos bytes, mesmo hash. Combinado com `predict_seq`, seleciona o formato de grafo de comprimentos que `[opt_table]` produzia antes da 0.2.0.

`opt_table=legacy` é outro formato. Depois dos campos obrigatórios, ele escreve um byte de contagem e um `u32` little-endian por campo opcional: `0` para ausente; caso contrário, o offset do valor no payload opcional mais 1. Não pode ser combinado com o `opt_table` simples, `predict_seq`, `state_bits`, `optionals=bitmap` nem `qps`.

### qps, quant {#qps-quant}

`qps` significa `qps=Q16`. `qps=Q2`, `Q4`, `Q8` e `Q16` definem o quantum. `quant=Qn` é um alias. `quant` sem valor é erro, assim como usar os dois nomes. Empacota só campos numéricos: strings, arrays e os comprimentos deles mantêm os próprios codecs, e os campos numéricos precisam vir antes deles. Não pode ser combinado com `compact`, `alignment`, `optionals=bitmap` nem `opt_table=legacy`.

Sem pool RT, um tipo com empacotamento ativo ganha um `serialize()` seguro que retorna `QslResult<QuantizedOut>`. Veja [frames menores](/guides/smaller-frames/).

### aggressive {#aggressive}

Muda o armazenamento em Rust, não os bytes nem a entrada aceita:

- `str(min..max)` vira `StackString<max>` e `array<T>(min..max)` vira `InlineArray<T, max>` quando o máximo cabe em `STACK_LIMIT`. Campos maiores continuam `String` ou `Vec`.
- Um campo opcional numérico é armazenado como um inteiro `NonZero` da menor largura que comporta o intervalo dele, desde que essa largura não seja maior que a do tipo do campo.

### lifetime {#lifetime}

Dá ao tipo um lifetime `'a` e empresta da entrada: strings viram `&'a str` e `array<u8>` vira `&'a [u8]`. A entrada precisa viver mais que o valor. Arrays de outros tipos de elemento e strings comprimidas não podem ser emprestados e fazem o build falhar, assim como um tipo `lifetime` sem nada para emprestar.

### transmutate {#transmutate}

Torna o layout nativo do tipo igual ao layout serializado. Todo campo precisa ter tamanho fixo: `compact`, `qps`, campos opcionais e strings ou arrays com intervalo são erros de build. O tamanho serializado precisa ser igual ao tamanho da struct, então adicione campos `padding(N)` onde o Rust insere padding.

Sem `lifetime`, `deserialize_transmutate_copy` copia o frame para um valor owned em um único bloco quando `NATIVE_WIRE_COMPATIBLE` é verdadeiro e, caso contrário, decodifica campo a campo. Com `[safety]`, o método é seguro e valida o resultado. Sem `[safety]`, é `unsafe`.

Com `lifetime`, `deserialize_overlay` retorna um `&Self` que aponta para dentro da entrada. O build verifica que cada offset no frame é igual ao offset nativo e falha em alvos big-endian. Com `[safety]`, `deserialize_overlay` verifica `NATIVE_WIRE_COMPATIBLE`, o comprimento e o alinhamento da entrada e, depois, os valores, e `deserialize_checked` faz o mesmo. Sem `[safety]`, `deserialize_overlay` é `unsafe` e não verifica nada.

Os dois têm variantes `_unchecked` que pulam as verificações.

### inline {#inline}

`inline` (o mesmo que `inline=hint`), `inline=always` ou `inline=never` coloca o atributo correspondente nos métodos gerados. Sem ele, os métodos gerados não levam atributo de inline, e o compilador decide. `always` pode aumentar o tamanho do código; não é ganho de velocidade garantido.

### slots=N {#slotsn}

Substitui o número de slots RT deste tipo, de 1 a 64. Só tem efeito com `!config QSL_RT = true;`. Veja [pool de slots RT](/guides/memory/#rt-slot-pool).

### cache=N, loyal_cache {#cachen-loyal_cache}

Gera `CACHE_BYTES` e `serialize_cached`, que escreve em um `qsl::modernization::CachedFrame<N>` fornecido por quem chama. `loyal_cache` define `CACHE_BYTES` como o tamanho máximo do frame. O tipo precisa de um tamanho máximo de frame finito, e um `N` menor que ele faz o build falhar. Isso prepara a memória de saída; não faz cache de valores. Veja [frames de saída em cache](/guides/memory/#cached-output-frames).

### state_bits=Qn {#state_bitsqn}

`Q1` (o padrão), `Q2`, `Q4`, `Q8` ou `Q16`. Seleciona um formato legado de índice de presença. Não é um modo de compressão. Não pode ser combinado com `optionals=bitmap`, `compact` nem `opt_table=legacy`.

## Modificadores de campo

### binary_opt {#binary_opt}

binary_opt foi removido por completo. Use [lifetime] para &str emprestado, [aggressive] para StackString limitado e decompress=true para InlineString. STACK_LIMIT vale para todas as strings e arrays.

### decompress=true {#decompresstrue}

Codifica a string com o codec LZ limitado do QSL. Strings com menos de 32 bytes, e strings que a compressão não encurtaria, são armazenadas sem compressão. Cada token decodificado é verificado contra o tamanho restaurado, a capacidade do destino e `MAX_DATA_SIZE`. A string precisa de um limite superior de no máximo 65535 bytes. Uma string com intervalo vira `InlineString<max>`, ou `String` acima de `STACK_LIMIT`. Não pode ser combinado com `lifetime` nem `lifetime`. Veja [strings comprimidas](/language/strings/#compressed-strings).

### default=N {#defaultn}

Gera `impl Default` para o tipo. Este campo começa com `N`, ou `Some(N)` se for opcional. Os outros campos começam com o menor valor válido deles. `N` precisa estar dentro do intervalo do campo, e o campo precisa ser numérico. Um tipo que também tenha um campo de tipo personalizado faz o build falhar. Não muda os bytes nem o hash.
