# Documentation model reference / Эталон моделей документации

These are functional wire-format checks, not benchmarks. Copy this directory to `.optimization/docs-lab` in the matching local `dev-qsl` source tree, then run:

```powershell
cargo run --manifest-path .optimization/docs-lab/Cargo.toml --locked
```

The program prints 32 numeric records in ordinary, compact and compact+alignment formats; all four states of the two-optional-u8 schema; and the native validator error for three corrupted frames. The models support these concrete schemas only. The public repository does not include the qsl Cargo workspace needed to build this harness.

Это функциональная проверка формата, а не измерение времени. Скопируйте каталог в `.optimization/docs-lab` соответствующей локальной копии `dev-qsl` и выполните команду выше. Вывод позволяет сверить байты и ошибки интерактивных моделей с настоящим Rust-кодом.
