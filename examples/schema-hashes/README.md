# Хэши схем / Schema hashes

Этот минимальный QSL/DEQ-фрагмент сохранён для CLI-анализатора. Полный запускаемый пример находится в [06-macros](../06-macros/README.md).

```powershell
cargo build --manifest-path qsl/Cargo.toml --release --bin qsl-analyze --features rust-macros
Get-Content examples/schema-hashes/packet.qsl -Raw | ./qsl/target/release/qsl-analyze.exe --generate examples/schema-hashes/schema.deq
cargo run --manifest-path examples/Cargo.toml --example macros
```

The analyzer computes strict/light identity during analysis. The complete Rust example demonstrates nested field expansion and the generated constants. Printing their hexadecimal representation is not a codec benchmark.
