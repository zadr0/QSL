---
title: 'Arrays e janelas de bits'
description: 'O comprimento de arrays conta elementos; o de strings conta bytes UTF-8. InlineArray mantém capacidade dentro da estrutura, enquanto Vec possui uma alocação separada. STACK_LIMIT limita estritamente o armazenamento interno. O descritor de Vec não é seu payload.'
---

O comprimento de um array conta elementos; o de uma string, bytes UTF-8. `array<u16>(0..16)` permite de zero a dezesseis elementos e `array<u16>(16)` exige exatamente dezesseis.

Vec possui um buffer separado por alloc. InlineArray mantém capacidade limitada dentro do objeto; arrays fixos podem usar `[T;N]`. STACK_LIMIT limita o armazenamento embutido. O descritor de Vec não é o payload.

$$
B_{\mathrm{array}}=H(n)+\sum_{i=1}^{n} w_i.
$$

H é o metadado de comprimento em bits, n é a quantidade de elementos e w é a largura do codec de cada elemento. Alinhamento é contado à parte. Um comprimento não permite ler fora do frame. Há suporte a 32 níveis aninhados; domínios e comprimentos são validados separadamente. O registrador depende da posição de bits; nem sempre é de 128 bits.

`aggressive[shift=true]` explícito pode escolher comprimento NonZero para InlineArray. O padrão é u32. Essa otimização física preserva o wire e distingue None de Some(empty).
[Read windows](/internals/optimizer/) · [Memory](/guides/memory/)

no_std exclui std, não alloc. qsl-core sempre usa #![no_std]; o recurso alloc, ativo por padrão, oferece alloc String/Vec, SerOut e restauração no heap. O aplicativo fornece o alocador; QSL não declara um alocador global. Desative os recursos padrão para usar apenas buffers do chamador.
