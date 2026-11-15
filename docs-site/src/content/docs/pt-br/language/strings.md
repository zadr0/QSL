---
title: Strings e ownership
description: Os tipos Rust em que uma string QSL pode se transformar, quando cada um é escolhido e o que a decodificação faz com a memória.
---

UTF-8 é contado em bytes: «Яa» ocupa 3 B. String possui memória dinâmica, &str empresta a entrada viva e StackString/InlineString têm capacidade limitada. QSL-LZ comprime apenas quando reduz o tamanho; caso contrário mantém raw. O gráfico mostra distribuição, literais e referências anteriores.

## Armazenamento e tamanho

InlineString mantém o codec dentro do objeto ou em Box no heap; strings grandes são restauradas diretamente no destino. StackString e InlineArray contêm sua capacidade limitada. aggressive[shift=true] pode escolher comprimento NonZero, distinguindo None de Some(empty).

## QSL-LZ {#compressed-strings}

São verificados cada token expandido, distância da referência, UTF-8 e orçamento acumulado. O tamanho armazenado não autoriza uma saída restaurada maior. Comprimentos são bytes UTF-8, não caracteres Unicode.
[Arrays](/language/arrays/) · [Limits](/reference/limits/) · [Memory](/guides/manual-layout/) · [QPS](/guides/smaller-frames/)

no_std exclui std, não alloc. qsl-core sempre usa #![no_std]; o recurso alloc, ativo por padrão, oferece alloc String/Vec, SerOut e restauração no heap. O aplicativo fornece o alocador; QSL não declara um alocador global. Desative os recursos padrão para usar apenas buffers do chamador.

O gerador escolhe a saída pelo tamanho máximo comprovado do quadro. Até STACK_LIMIT inclusive usa `StackOut<N>`; um limite maior ou desconhecido seleciona alloc::`Vec<u8>`. Cabeçalhos transmitidos contam; metadados do objeto de saída não.
