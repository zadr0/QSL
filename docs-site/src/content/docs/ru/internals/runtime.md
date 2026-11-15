---
title: 'Runtime, RT и транспорт'
description: 'Документация QSL: Runtime, RT и транспорт.'
---

no_std исключает std, но не alloc. qsl-core всегда использует #![no_std]. Возможность alloc включена по умолчанию и предоставляет alloc String/Vec, SerOut и восстановление в heap. Аллокатор задаёт приложение; QSL не объявляет глобальный аллокатор. Для работы только с переданной памятью отключаются default features.

Статические регионы no_std, реализующие FramePool/FrameStorage ядра. Исчерпанный пул сразу возвращает PoolExhausted.
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString сохраняет кодек при размещении на стеке или в Box на heap. Большие heap-строки восстанавливаются прямо в назначение. StackString и InlineArray содержат ограниченную ёмкость внутри объекта. Явное aggressive[shift=true] может выбрать NonZero-длину для строк и массивов, сохраняя различие None и Some(empty).

build.rs и тестовый harness могут использовать std, не добавляя его в runtime целевой программы. Реально сгенерированные схемы запускаются без аллокатора; отдельная no_std-библиотека проверяет alloc-контейнеры. Оба режима ядра и qsl-rt собираются также для wasm32-wasip2.

[Memory](/guides/memory/) · [Safety](/guides/safety/)

Генератор выбирает тип выхода по доказанному максимуму кадра. Если он не превышает STACK_LIMIT, включая равенство, обычная сериализация возвращает `StackOut<N>`; больший или неизвестный максимум выбирает alloc::`Vec<u8>`. Передаваемые заголовки входят в порог; служебная длина и выравнивание самого выходного объекта — нет.
