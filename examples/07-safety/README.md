# Ошибки до указательного чтения / Errors before pointer decoding

[Все примеры / All examples](../README.md) · [Замеры / Measurements](../PERFORMANCE.md)

Каждая схема имеет validate, serialize_checked_to и deserialize_checked_into. Пример проверяет диапазон, ложную длину и UTF-8. MAX_DATA_SIZE=65535 ограничивает входной кадр, восстановленный QPS-кадр и суммарные значения; ограничения схемы могут отклонить поле раньше общего лимита.

```powershell
cargo run --manifest-path examples/Cargo.toml --example safety
```

Every schema has validate, serialize_checked_to and deserialize_checked_into. This program exercises numeric domains, forged length and UTF-8. MAX_DATA_SIZE=65535 bounds input, QPS restoration and aggregate values; schema constraints can reject a field before the global budget.

[main.rs](main.rs) · [схемы / schemas](../schemas/07-safety.qsl)
