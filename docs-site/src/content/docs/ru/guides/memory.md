---
title: "Память и повторное использование"
description: "Документация QSL: Память и повторное использование."
---

Генератор выбирает тип выхода по доказанному максимуму кадра. Если он не превышает STACK_LIMIT, включая равенство, обычная сериализация возвращает `StackOut<N>`; больший или неизвестный максимум выбирает alloc::`Vec<u8>`. Передаваемые заголовки входят в порог; служебная длина и выравнивание самого выходного объекта — нет.

String, Vec и Box используют alloc в runtime no_std; аллокатор предоставляет приложение. Ограниченный InlineString размещается внутри структуры, если его нативный размер укладывается в STACK_LIMIT, иначе — в Box. Выбор определяется ёмкостью схемы, а не текущей длиной значения.

Подготовленный unsafe reuse требует достаточной ёмкости до вызова и сохраняет пригодные буферы String, Vec и boxed InlineString. Их рост в этом пути не выполняется. Инициализированный mutable-объект и неинициализированное назначение имеют разные контракты.

<a id="reading-into-existing-memory"></a>

## Пример {#example}

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

Пулы qsl-rt предоставляют 1–64 слота. LocalRegions не использует атомики, StaticRegions допускает несколько потоков. Занятый слот удерживается lease до Drop; заполненный пул сразу возвращает PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.
