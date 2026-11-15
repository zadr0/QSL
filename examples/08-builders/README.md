# Builder в build.rs / Builders in build.rs

[Программа / Program](main.rs) · [Генерация / Generation](../build.rs) · [Все примеры / All examples](../README.md)

Функция `build_builder_example` в `examples/build.rs` показывает два способа задать одну схему: типизированные объявления и QSL-текст. Оба используют общий анализатор и `use_interpreter()`, записывают Rust в `OUT_DIR`, а программа подключает результаты в разные модули.

`QslBuilder` собирает схемы; замыкание `config` получает `DeqBuilder`, замыкание `schem` — `QslSchemaBuilder`. В этом примере `stack_limit(64)` задаёт стековый бюджет, `max_data_size(65535)` — независимый предел данных. `aggresive(false, false)` включает агрессивную раскладку без сдвига длины и без дополнительного bitmap. Это текущее имя метода API.

```rust
let builder = qsl::QslBuilder::new()
    .config(|c| { c.stack_limit(64).max_data_size(65535); })
    .schem(|s| {
        s.name("Packet").safety().aggresive(false, false);
        s.field("id").u32(0, 1_000_000, 0, false);
        s.field("name").str(1, 32, false, false);
        s.field("status").u8(0, 15, 0, true);
    })
    .build()?;
```

Для чисел параметры — `min, max, default, optional`; для `str` — `min, max, decompress, optional`. Границы строк считаются в байтах UTF-8. Обязательные поля идут первыми, `status` — последним. Текстовый вариант создаётся через `QslSchemaBuilder::from_source_with_config(source, builder.config())`.

Программа проверяет совпадение байтов и хэшей обоих вариантов, чтение с `status=None` и `Some(7)`, отказ на обрезанном кадре. Поле `name` имеет тип `StackString<32>`, выход — `StackOut<42>`; фактические кадры с именем `QSL` занимают 12 и 13 B.

The `build_builder_example` function in `examples/build.rs` declares the same schema through typed calls and QSL text, then selects the shared Rust backend with `use_interpreter()`. Generated files are included in separate modules.

`QslBuilder` collects schemas, `config` receives a `DeqBuilder`, and `schem` receives a `QslSchemaBuilder`. Integer parameters are `min, max, default, optional`; string parameters are `min, max, decompress, optional`, with UTF-8 byte bounds. The existing method `aggresive(false, false)` enables aggressive storage without length shifting or an extra bitmap. Required fields precede optional fields.

The executable checks matching bytes and schema hashes, both presence states, and truncated-input rejection. The 64 B stack budget produces `StackString<32>` and `StackOut<42>`; the data limit remains independently set to 65535 B. The two actual frames contain 12 and 13 B.

```powershell
cargo run --manifest-path examples/Cargo.toml --example builders --locked
```

Загрузка схем из файлов показана отдельно в [compiler-pipeline](../compiler-pipeline/build.rs). / See [compiler-pipeline](../compiler-pipeline/build.rs) for file-based collection.
