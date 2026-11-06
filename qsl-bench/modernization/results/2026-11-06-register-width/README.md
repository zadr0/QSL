# Typed register windows / Типизированное регистровое чтение

Only two numerical Criterion cases were rerun after introducing typed u8/u16/u32/u64/u128 loaders. Five ASM probes are inspection targets, not additional benchmarks.

После выбора минимального типа загрузки повторены только два числовых случая Criterion. Пять ASM-проб не являются дополнительными бенчмарками.

| Case | Before, ns | After, ns | After 99% CI, ns | Change in T | Wire B | P, useful B/ns |
|---|---:|---:|---:|---:|---:|---:|
| default_deserialize_into | 2.6792 | 2.7876 | 2.7735–2.8026 | +4.04% | 24 | 8.610 |
| Q16_prepared_decode_into | 4.1467 | 4.3037 | 4.2141–4.4495 | +3.79% | 5 | 5.577 |

Times are Criterion slope estimates, matching the earlier four-case report.

Warm-up: 15 s; measurement: 10 s; 200 samples; 99% confidence; 1024 prepared frames; process cap: 8 GiB. Peak committed job memory: 12.918 MiB. Original useful M=24 B for both; P=M/T. Windows 11 x86-64, i5-7400, 24 GiB RAM at 2133 MHz. The user-waived 5% idle gate remains waived; full CPU idleness is not claimed. Allocation, validation and hot metrics samplers are excluded. Outliers: 11% / 11.5%. Earlier and current runs are not a randomized paired experiment.

Прогрев 15 с, измерение 10 с, 200 выборок, CI 99%, 1024 подготовленных кадра, лимит процесса 8 GiB. Пик committed памяти: 12,918 MiB. Для обеих строк M=24 полезных байта, P=M/T. Полный простой CPU не заявляется: порог фоновой загрузки ранее отменён. Аллокации, валидация и сбор метрик на горячем пути исключены. Выбросы: 11% / 11,5%. Прогоны не являются рандомизированным парным экспериментом.

No speedup is established. Both decoder inspection probes are instruction-identical to the earlier measured probes: default has 5 instructions and Q16 has 26, including return. LLVM had already narrowed the legacy u128 source expressions for these cases. The correction establishes explicit generated types and exact read windows; it does not establish an improvement in runtime speed.

Ускорение не подтверждено. В проверяемых ASM-пробах инструкции декодеров совпадают с прежними: 5 у default и 26 у Q16, включая возврат. Для этих случаев LLVM уже сокращал старые выражения u128. Исправление обеспечивает явные минимальные типы в генерации и точные границы чтения; прирост скорости этим прогоном не доказан.

| Probe | Input bytes | Result type | Exact memory reads | Conditional jumps / calls |
|---|---:|---|---|---|
| qsl_load_u8_probe | 1 | u8 | 1 | 0 / 0 |
| qsl_load_u16_probe | 2 | u16 | 2 | 0 / 0 |
| qsl_load_u32_probe | 3 | u32 | 2 + 1 | 0 / 0 |
| qsl_load_u64_probe | 5 | u64 | 4 + 1 | 0 / 0 |
| qsl_load_u128_probe | 9 | u128 | 8 + 1 | 0 / 0 |

Run / Запуск from dev-qsl:

```powershell
pwsh -File qsl-bench/modernization/run.ps1 -Output qsl-bench/modernization/results/2026-11-06-register-width -Filter modernization_numeric_24_useful_bytes
```

[Summary / Сводка](summary.json) · [CSV](summary.csv) · [Raw log / Журнал](run.log) · [ASM](assembly.txt) · [Source SHA-256](source-manifest.json) · [Measured source archive](measured-source.zip) · [Generated Rust](generated.rs)

The binary SHA-256 and execution settings are in metadata.json. Historical results are not overwritten. These trusted-read timings do not measure checked decoding or validate every schema's machine code.
