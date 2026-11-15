---
title: Buffers e reutilização
description: Escrita nos seus próprios buffers, decodificação em valores existentes, o pool estático de slots RT e frames de saída em cache.
---

O gerador escolhe a saída pelo tamanho máximo comprovado do quadro. Até STACK_LIMIT inclusive usa `StackOut<N>`; um limite maior ou desconhecido seleciona alloc::`Vec<u8>`. Cabeçalhos transmitidos contam; metadados do objeto de saída não.

String, Vec e Box usam alloc em no_std; a aplicação fornece o alocador. InlineString fica no próprio objeto quando seu tamanho nativo cabe em STACK_LIMIT, caso contrário usa Box. A capacidade do esquema determina a escolha.

O reuse unsafe preparado exige capacidade suficiente antes da chamada e mantém buffers compatíveis de String, Vec e InlineString em Box. Não há crescimento desses buffers. Objetos inicializados e destinos não inicializados têm contratos distintos.

<a id="reading-into-existing-memory"></a>

## Exemplo {#example}

```qsl
&Record[safety] { u64 id; }
```

```rust
let value = Record { id: 42 };
let mut buffer = [0u8; 64];
let used = value.serialize_checked_to(&mut buffer)?;
let mut destination = core::mem::MaybeUninit::<Record>::uninit();
let restored = Record::deserialize_checked_into(&buffer[..used], &mut destination)?;
assert_eq!(restored.id, 42);
```

[STACK_LIMIT](/language/deq/) · [String](/language/strings/) · [RT](/internals/runtime/)

## QSL_RT {#rt-slot-pool}

LocalRegions · StaticRegions · FramePool · FrameStorage. 1–64 slots; lease → Drop. PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.
