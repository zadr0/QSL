# Числа и три формата / Numbers and three formats

[Все примеры / All examples](../README.md) · [Замеры / Measurements](../PERFORMANCE.md)

Восемь u64 с диапазоном 0..15 дают 64 полезных байта. Уменьшение кадра определяется диапазоном; точные значения восстанавливаются. raw, compact и числовой QPS измеряются одним набором из 1024 записей.

```powershell
cargo run --manifest-path examples/Cargo.toml --example basic
```

Eight u64 values constrained to 0..15 contain 64 original useful bytes. Domain packing changes wire size and restores values exactly. Raw, compact and scalar QPS share 1024 records.

[main.rs](main.rs) · [схемы / schemas](../schemas/01-basic.qsl)
