# Статические выходные слоты / Static output slots

[Все примеры / All examples](../README.md) · [Замеры / Measurements](../PERFORMANCE.md)

RT включён только для RtNumbers. Запись получает один из 16 слотов; живая lease сохраняет его занятым. После Drop слот доступен снова. Vec в программе лишь удерживает lease для демонстрации исчерпания и не входит в RT-алгоритм. Замер включает получение и освобождение одного слота без конкуренции.

```powershell
cargo run --manifest-path examples/Cargo.toml --example rt
```

RT is enabled only for RtNumbers. A live lease occupies one of 16 slots until Drop. The demo Vec merely holds leases to exercise exhaustion and is not part of RT encoding. Timing includes acquiring and releasing one uncontended slot.

[main.rs](main.rs) · [схемы / schemas](../schemas/05-rt.qsl)
