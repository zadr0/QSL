---
title: "Dyn v1 и wire-формат"
description: "Документация QSL: Dyn v1 и wire-формат."
---


```qsl
&P[safety,dyn[type=index(dynamic),width=32,revision=1]] {
    u32 id = 1;
    str?(0..64) text = 4;
}
```

В dyn `=1` задаёт tag, а не initializer. Идентификаторы есть у всех полей, включая required.

Заголовок — 12 B: QDY1, mode:u8, width:u8, reserved:u16=0, revision:u32 LE. Служебные целые little-endian, адресация payload — с нуля, MSB-first.
Режим | Запись
---|---
line16/32/64 | tag:uW, wire_span:uW, payload; span в байтах
dynamic index32 | offset:u16, tag:u8, size:u8
dynamic index64 | offset:u32, tag:u16, size:u16
strict index32 | offset:u16, tag:u16
strict index64 | offset:u32, tag:u32
В index offset/size измеряются в битах: offset=32,size=4 — старшие четыре бита пятого байта payload. У динамической строки/массива offset указывает на мета-заголовок, size — только его ширина. В raw это 32-битная длина; текст считает UTF-8 байты, массив — элементы. Полная граница определяется следующим offset/началом таблицы.

Таблица следует за payload. Trailer:u32 LE хранит расстояние назад от начала trailer до начала таблицы **в байтах**. Отсутствующий optional не создаёт запись. Неизвестное поле пропускается по границам; одинаковый tag сам по себе не доказывает совместимость типов.

Checked index принимает scratch u32 на запись, line — u64 на запись:

```rust
let mut scratch = [0u32; 2];
let value = P::deserialize_checked(frame, &mut scratch)?;
```

Scratch проверяет повторные теги, передаётся явно и не заимствуется результатом. Временный Vec для этой проверки не создаётся. Нехватка — BufferTooSmall. Проверяются header/trailer, ширины, теги, порядок offsets, длины, required, домены, UTF-8, padding и общий бюджет.

Index32/64 поддерживает raw и выбранные COMPACT scalar-поля. COMPACT здесь MSB-first, строки/массивы сохраняют byte barriers и raw metadata. Line+COMPACT, index16 и текущие dyn-сочетания с QPS/PS/OT/BM/LZ/RT/overlay/private bitmap отклоняются. .
