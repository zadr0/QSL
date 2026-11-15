# `opt_table` — спецификация атрибута

**Статус:** реализовано, кроме оговорки для `safety`: только флаг в этой версии.

## Синтаксис

```qsl
&TableFields[opt_table] { u64 id; u32? a; str?(0..100) text|decompress=true|; u16? b; }
```

## Точная механика и wire-эффект

Для трёх optional записываются count=3 и 3×u32 LE. Если a и b присутствуют, а text отсутствует, offsets равны 1,0,5 при payload a(4 B)+b(2 B).

## Ограничения и недопустимые сочетания

Required поля сначала; не compact/bitmap/predict_seq. Для одного optional действует EOF, каталог отсутствует. Offset проверяет metadata, не содержимое field codec.

## Контракт исполнения

При изменении схемы обе стороны должны использовать совместимую ревизию формата. Путь `unsafe` не подтверждает корректность произвольного сетевого ввода; источник, объём данных и диапазоны должны соответствовать схеме.

## Что измерять

Сравнивай полную запись и чтение на одних данных, отдельно метаданные и payload; фиксируй размер сообщения, аллокации, CPU, измерительный профиль и p99. Исходный пример замера:
`optional/seven_table_read` — [all-estimates.csv](../../data/all-estimates.csv).

## Исходная реализация

`qsl/src/optional.rs OptionalOffsets; interpreter.rs emit_sequential_write/read`.

[← Все атрибуты](index.md) · [Architecture](../../architecture.ru.md)