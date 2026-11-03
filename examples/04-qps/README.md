# Два пути QPS / Two QPS paths

[Все примеры / All examples](../README.md) · [Замеры / Measurements](../PERFORMANCE.md)

Числовая упаковка и оболочка QPB3 — отдельные форматы. Общий путь включает сериализацию внутреннего кадра, сжатие, восстановление, проверку и создание результата. Raw-fallback может добавить 16 B оболочки. Стенд сравнивает общий QPS с обычным checked-кодеком на том же ASCII-наборе.

```powershell
cargo run --manifest-path examples/Cargo.toml --example qps
```

Scalar packing and QPB3 are distinct formats. Whole QPS includes inner serialization, compression, restoration, validation and construction. Raw fallback may add 16 B framing. The benchmark compares whole QPS with an ordinary checked codec on the same ASCII data.

[main.rs](main.rs) · [схемы / schemas](../schemas/04-qps.qsl)
