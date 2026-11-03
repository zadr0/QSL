# Владение строкой и восстановление / String ownership and restoration

[Все примеры / All examples](../README.md) · [Замеры / Measurements](../PERFORMANCE.md)

String создаёт собственную строку при чтении; borrowed хранит вид входа; CopyGuard копирует UTF-8 в SerOut. В замере fresh-read и String, и CopyGuard включают выделение и освобождение памяти. Inline/LZ использует встроенную ёмкость 4096 B. Фиксированный str(4) остаётся [u8; 4].

```powershell
cargo run --manifest-path examples/Cargo.toml --example strings
```

String constructs owned storage, borrowed keeps an input view, and CopyGuard copies UTF-8 into SerOut. Fresh-read timing includes allocation and destruction for both String and CopyGuard. Inline/LZ has 4096 B inline capacity. Fixed str(4) remains [u8; 4].

[main.rs](main.rs) · [схемы / schemas](../schemas/02-strings.qsl)

Фраза выбирает raw-fallback; два байта экономит заголовок. Отдельный набор из 1024 одинаковых символов кириллицы показывает настоящее LZ-сжатие и имеет собственную String-базу. / The phrase selects raw fallback: two bytes come from the header. A separate 1024-character repeated-Cyrillic set measures genuine LZ against its own String baseline.
