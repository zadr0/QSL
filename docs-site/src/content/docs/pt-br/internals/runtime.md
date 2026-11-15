---
title: 'Runtime e buffers'
description: 'SerOut e buffers preparados controlam alocações e reutilização. QSL_RT cria slots estáticos apenas para esquemas limitados elegíveis: 16 por esquema por padrão. Quando lotados, retorna erro sem alocação oculta. O runtime atual não inclui sockets nem regiões específicos da plataforma.'
---

no_std exclui std, não alloc. qsl-core sempre usa #![no_std]; o recurso alloc, ativo por padrão, oferece alloc String/Vec, SerOut e restauração no heap. O aplicativo fornece o alocador; QSL não declara um alocador global. Desative os recursos padrão para usar apenas buffers do chamador.

Regiões estáticas no_std que implementam FramePool/FrameStorage. Pool cheio retorna PoolExhausted imediatamente.
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString mantém o codec dentro do objeto ou em Box no heap; strings grandes são restauradas diretamente no destino. StackString e InlineArray contêm sua capacidade limitada. aggressive[shift=true] pode escolher comprimento NonZero, distinguindo None de Some(empty).

Scripts de compilação e testes podem usar std sem incluí-lo no runtime alvo. Esquemas reais gerados executam sem alocador; outra biblioteca no_std verifica contêineres alloc. Os dois perfis do núcleo e qsl-rt também compilam para wasm32-wasip2.

[Memory](/guides/memory/) · [Safety](/guides/safety/)

O gerador escolhe a saída pelo tamanho máximo comprovado do quadro. Até STACK_LIMIT inclusive usa `StackOut<N>`; um limite maior ou desconhecido seleciona alloc::`Vec<u8>`. Cabeçalhos transmitidos contam; metadados do objeto de saída não.
