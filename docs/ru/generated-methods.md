# Разбор сгенерированных методов QSL

Приведённый код ниже — **объясняющий псевдокод**, а не точная копия `qsl_gen.rs` конкретной сборки: фактический генератор находится в `qsl/src/interpreter.rs` и выбирает реализацию на основании полной схемы. Названия низкоуровневых методов и публичных трейтов взяты из предоставленного исходника.

## `Fixed`: статический плоский путь

```qsl
&Fixed { u32 id; u64 created_at; u16 checksum; }
```

```rust
// Иллюстрация прямой записи; actual QSL generator may inline differently.
unsafe fn fixed_write(value: &Fixed, out: *mut u8) {
    out.cast::<u32>().write_unaligned(value.id.to_le());
    out.add(4).cast::<u64>().write_unaligned(value.created_at.to_le());
    out.add(12).cast::<u16>().write_unaligned(value.checksum.to_le());
}
```

**Почему быстро:** нет runtime-таблицы типов; смещения 0/4/12 вычислены ранее. **Почему не обязательно быстрее всех:** нужны загрузки значений из исходной структуры, stores в выход, возможна зависимость памяти, аппаратная пропускная способность и функции обёртки.

`get_size()` и `serialized_size()` становятся 14, `serialize_unsafe` делегирует генераторному `__qsl_write_unchecked`, `deserialize_unsafe` — `__qsl_read_unchecked`. Для фиксированной схемы это один из путей `emit_flat_write` / `emit_flat_read`.

## `Dense`: компактная ранговая запись

```qsl
&Dense[compact] { u8(0..7) flags; u8(0..31) rank; u16(0..1023) code; }
```

Поля занимают 3, 5 и 10 бит. При значениях `flags=6, rank=29, code=777` условная битовая формула QSL (LSB-first) имеет вид:

```text
packed = (flags & 0b111)
       | ((rank & 0b11111) << 3)
       | ((code & 0b1111111111) << 8)
wire_length = ceil(18 / 8) = 3
```

Generator emits constant masks and shifts in `emit_compact_flat_write/read`. В отличие от обычной структуры, компактный код платит за маскирование, но экономит 1 байт на этом конкретном наборе полей.

## `SeqNumber[predict_seq]`: длина как признак хвоста

```rust
// Псевдокод только для валидного input.
let id = read_u64_le(input);
let extra = if input.len() > 8 {
    Some(read_u32_le(&input[8..12]))
} else {
    None
};
```

Для `len=9..11` это **небезопасно**, поскольку поле целиком не помещается. Следовательно вход либо должен быть заранее проверен, либо для сетевого режима нужен отдельный checked reader. `deserialize_unsafe(ptr)` намеренно паникует, поскольку без длины нельзя определить наличие.

## `TableFields[opt_table]`: каталог и точечный декодер

```text
required u64 id | count=3 | u32 offset[0] | u32 offset[1] | u32 offset[2] | payload
                   03       01 00 00 00     00 00 00 00     05 00 00 00      a(4B) + b(2B)
```

Псевдокод `offset(i)`: `raw = read_u32_le(directory + 4*i); raw==0 ? None : Some(payload + raw-1)`. Это не удаляет декодирование конкретного значения: `&str`/`InlineString` требуют собственных процедур. `OptionalOffsets::parse` проверяет последовательность и границы каталога, но не UTF-8 каждого поля.

## `serialize`, `serialize_to`, `serialize_into_unsafe`

- `get_size()` — build-time минимум wire-размера, **не** всегда реальная длина сообщения.
- `serialized_size()` — точная обычная длина или верхняя граница при сжатых строках; не равна всегда написанным байтам.
- `serialize()` — единичная аллокация Vec/SerOut и trusted raw write.
- `serialize_into_unsafe(out_ptr)` — запись и возврат фактически инициализированной длины.
- `serialize_to(&mut buffer)` — повторное использование буфера без runtime-проверки вместимости; вызывающий отвечает за ёмкость.
- `deserialize(&[u8])` — **unsafe**: наличие среза не означает проверки содержимого.
- `deserialize_from_raw_parts(ptr,len)` — нужен для predict_seq; trusted.
- `deserialize_into(ptr,len,out)` — чтение в выровненную неинициализированную итоговую память; полезно для больших `InlineString<MAX>`.
- `BorrowSerializable<'a>` — соответствующие заимствованные формы; вход живёт как минимум `'a`.

Все публичные сигнатуры и исходные строки см. [`API.ru.md`](../API.ru.md).
