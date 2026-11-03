# Вложенный шаблон и идентичность / Nested template and identity

[Все примеры / All examples](../README.md) · [Замеры / Measurements](../PERFORMANCE.md)

Rust Trace сохраняет вложенность; QSL внедряет его поля в MacroPacket. Хэши вычисляются при анализе, а не во время каждой записи. Вывод hex в этой программе форматирует константы и выполняет обычную печать; это не бенчмарк хэширования.

```powershell
cargo run --manifest-path examples/Cargo.toml --example macros
```

Rust Trace remains nested; QSL expands its fields into MacroPacket. Hashes are computed during analysis rather than each write. Hex output formats constants and prints them; it is not a hashing benchmark.

[main.rs](main.rs) · [схемы / schemas](../schemas/06-macros.qsl)
