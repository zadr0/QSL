# Предсказание optional-хвоста / Optional-tail prediction

[Все примеры / All examples](../README.md) · [Замеры / Measurements](../PERFORMANCE.md)

Программа выводит все состояния N=3. Стенд отдельно сравнивает default и opt_table для N=1/2/3/5/10. Обязательный id занимает 4 B; поля optional находятся в конце. Проверяемое чтение включает полную валидацию; одна длина не различает все маски одинаковых u8.

```powershell
cargo run --manifest-path examples/Cargo.toml --example optionals
```

The program prints all N=3 states. The benchmark compares default and opt_table for N=1/2/3/5/10. Required id occupies 4 B; optional fields are last. Checked reading includes full validation; length alone cannot distinguish all equal-width u8 masks.

[main.rs](main.rs) · [схемы / schemas](../schemas/03-optionals.qsl)
