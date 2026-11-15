# BMI2: PEXT и PDEP / BMI2: PEXT and PDEP

[Программа / Program](main.rs) · [Схема / Schema](../schemas/09-bmi2.qsl) · [Конфигурация / Config](../schemas/bmi2.deq) · [Ядра / Kernels](../../qsl-core/src/optimizer/kernels.rs)

Четыре поля `u32(0..4095)` занимают 16 B в памяти и 6 B в compact-кадре. Пример сравнивает сгенерированный кадр, обычные сдвиги и BMI2: `pack4_bmi2` использует PEXT, `unpack4_bmi2` — PDEP. Оба ядра обрабатывают две независимые 64-битные части массива, сохраняя порядок полей и те же wire-байты.

Перед вызовом ядра диапазоны проверяются через `validate4`, поддержка BMI2 — через `is_x86_feature_detected!`. Проверка CPU выполняется снаружи ядра; обхода полей с выбором инструкции нет. Без BMI2 программа выполняет переносимый вариант. Ядра относятся к `no_std` runtime; обнаружение CPU здесь выполняет приложение со `std`.

Обычный запуск безопасно проверяет поддержку CPU и выполняет доступные ядра:

```powershell
cargo run --manifest-path examples/Cargo.toml --example bmi2 --locked
```

Чтобы включить BMI2 именно в сгенерированной записи, нужны `CPU_PROFILE = kaby_lake`, `aggressive[group=true]`, четыре непрерывных нормализованных 12-битных поля и target-feature `bmi2`. Первые условия уже заданы в файлах примера. На CPU с BMI2 выполните:

```powershell
cargo rustc --manifest-path examples/Cargo.toml --example bmi2 --release --locked -- -C target-feature=+bmi2 --emit=link,asm
& ./examples/target/release/examples/bmi2.exe
Get-ChildItem examples/target/release/examples -Filter 'bmi2*.s' | Select-String -Pattern '\bpext\b|\bpdep\b'
```

Команда изменяет флаги только выбранного executable и сохраняет ассемблер рядом с результатами сборки; глобальный `RUSTFLAGS` не нужен. Такая целевая сборка предназначена для CPU с BMI2.

В текущем профиле сгенерированная запись выбирает PEXT, чтение остаётся скалярным. Вызов PDEP в программе демонстрирует отдельное ядро и проверяет обратимость; он не меняет выбор генератора. Программа также проверяет отказ checked-записи для значения 4096. Она проверяет корректность, а не измеряет ускорение.

Four `u32(0..4095)` fields occupy 16 native bytes and 6 compact wire bytes. The executable compares generated serialization with scalar shift/OR packing and BMI2 PEXT/PDEP kernels, checks exact restoration, and rejects an out-of-range value. Kernels load two initialized 64-bit array segments without reading padding.

The default command detects BMI2 outside the kernels and safely skips ISA calls on unsupported CPUs. Runtime kernels are `no_std`; CPU detection belongs to this `std` example. To enable the generated PEXT writer, use the second command on a BMI2 CPU. The supplied profile and schema enable its supported four-field, normalized 12-bit shape; the compiler flag enables the target instructions without changing global settings. Assembly is emitted beside the executable.

The selected generated reader remains scalar. The explicit PDEP call demonstrates the separate reversible kernel, not an automatic read-path selection. Matching bytes establish correctness; this example does not establish a performance improvement.
