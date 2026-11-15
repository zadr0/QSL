# Строки: стек, heap и reuse / Strings: stack, heap and reuse

[Все примеры / All examples](../README.md) · [Схемы / Schemas](../schemas/02-strings.qsl) · [Программа / Program](main.rs)

При `STACK_LIMIT = 256` генератор выбирает следующие типы. Ограничения строк измеряются в байтах UTF-8.

With `STACK_LIMIT = 256`, the generator selects the following types. String bounds count UTF-8 bytes.

| Схема / Schema | Поле Rust / Rust field | Выход / Output |
|---|---|---|
| `OwnedText`, `OwnedReuseText` | `String` | `Vec<u8>` |
| `BorrowedText` | `&str` | `Vec<u8>` |
| `InlineText` | `InlineString<55>` | `StackOut<56>` |
| `AggressiveText` | `StackString<55>` | `StackOut<59>` |
| `CompressedText` | `Box<InlineString<4096>>` | `Vec<u8>` |
| `FixedText` | `[u8; 4]` | `StackOut<4>` |

Маленький `InlineString` — часть структуры; `Box` ему не требуется. Большой контейнер выделяется сразу в heap через `InlineString::try_boxed_from_str`, без промежуточного большого объекта на стеке. Оба варианта хранят обычный UTF-8 в памяти и используют QSL-LZ при записи. `StackString` использует raw UTF-8. Конструкторы возвращают ошибку при превышении ёмкости.

A small `InlineString` is embedded directly in the generated structure. Large storage is allocated directly on the heap through `InlineString::try_boxed_from_str`, avoiding a large stack temporary. Both variants keep decoded UTF-8 in memory and use QSL-LZ on the wire. `StackString` uses raw UTF-8. Constructors reject values exceeding capacity.

```rust
let small = InlineText {
    text: qsl::InlineString::try_from_str("QSL")?,
};
let large = CompressedText {
    text: qsl::InlineString::try_boxed_from_str("QSL")?,
};
```

Даже при одинаковом тексте типы различаются из-за ёмкости схем: 55 и 4096 B. Для нативного контейнера лимит учитывает длину и выравнивание. Для выходного кадра сравнивается максимальное число передаваемых байтов, включая wire-заголовки; равенство лимиту разрешено. Например, `StackOut<56>` содержит до 56 B кадра, но фактическая длина короткой записи меньше.

The same text can have different storage because the schema capacities differ. Native container budgeting includes length storage and alignment. Output budgeting compares the maximum transmitted frame size, including wire headers, and permits equality with the limit. A `StackOut<56>` frame can contain fewer than 56 initialized bytes.

Программа также проверяет заимствованное чтение, фиксированную строку и `deserialize_into` в уже подготовленный `String`. Перед небезопасным reuse выполняется `validate`; ёмкость 4096 B гарантирована заранее, адрес буфера после чтения сохраняется. Для одинаковых символов отдельно проверяется уменьшение размера и точное восстановление; более короткий заголовок сам по себе не считается доказательством сжатия.

The program also verifies borrowed decoding, fixed strings, and `deserialize_into` into a prepared `String`. Wire validation precedes unsafe reuse, capacity is prepared in advance, and the buffer address remains unchanged. A repeated-character control checks actual compression and exact restoration; header savings alone do not establish payload compression.

```powershell
cargo run --manifest-path examples/Cargo.toml --example strings --locked
```
