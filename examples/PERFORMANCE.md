# Производительность примеров / Example performance

[Примеры / Examples](README.md) · [Интерактивный отчёт / Interactive report](performance.html) · [CSV](performance/measurements.csv) · [JSON](performance/measurements.json)

Замер текущих примеров, без смешивания с прежними таблицами QSL. Windows 11 Pro x86-64 (10.0.26200), Intel Core i5-7400, 24 GiB / 2133 MHz; Rust 1.98.0 / LLVM 22.1.8; Criterion 0.8.2. Измерительный поток: CPU mask 0x4, above-normal; ОС не изолирована.

15 с прогрева + 10 с измерения на операцию, 200 выборок, 99% CI; 1024 заранее подготовленных записи. T — среднее Criterion в нс/операцию. M — исходные полезные байты, P=M/T — B/ns. Размер wire включает метаданные; он не используется как M.

Current examples only; older QSL measurements are separate. Warm-up 15 s, measurement 10 s, 200 samples, 99% CI, 1024 prepared records. T is Criterion's arithmetic mean in ns/op, M original useful bytes, and P=M/T is B/ns. Wire bytes include metadata and do not replace M.

Owned reads include construction and destruction; writes reuse prepared output unless labelled heap. Checked includes the generated validation and resource limit. Trusted assumes a prevalidated matching frame. Borrowed results do not copy payload bytes: their P is not memcpy bandwidth. Validation-only is not a full decode. RT uses one live uncontended lease.

Измерено 39 операций, снимков: 2. Основной кодек совпадает; каждая сравниваемая группа относится к одному бинарному файлу. Источники и профиль указаны для каждой строки JSON/CSV; одинаковые случаи не объединяются. / 39 operations across 2 source profiles. Codec sources match, matched groups share a binary, and duplicate cases are never pooled.

Пиковая память JobObject: **16.20 MiB**, лимит **8 GiB**. Peak job memory includes measured process descendants. Source SHA-256 before/after is identical.

## Числа: 8 u64(0..15) / Numbers: 8 u64(0..15)

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| read / compact_checked | checked | 64.0 | 4.00 | 133.809 | 132.546–135.185 | 0.4783 |
| read / qps_Q16_checked | checked | 64.0 | 4.00 | 133.410 | 131.880–135.291 | 0.4797 |
| read / raw_checked | checked | 64.0 | 64.00 | 90.354 | 89.308–91.544 | 0.7083 |
| read / raw_trusted | trusted | 64.0 | 64.00 | 7.096 | 7.022–7.178 | 9.0190 |
| write / compact_checked | checked | 64.0 | 4.00 | 10.300 | 10.168–10.448 | 6.2136 |
| write / qps_Q16_checked | checked | 64.0 | 4.00 | 9.942 | 9.844–10.052 | 6.4375 |
| write / raw_checked | checked | 64.0 | 64.00 | 10.278 | 10.196–10.367 | 6.2268 |
| write / raw_trusted | trusted | 64.0 | 64.00 | 4.712 | 4.652–4.781 | 13.5814 |

## Строки: 1024 символа UTF-8 / Strings: 1024 UTF-8 characters

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| read / borrowed_checked | checked | 1683.0 | 1687.00 | 1545.671 | 1528.485–1565.802 | 1.0888 |
| read / borrowed_trusted | trusted | 1683.0 | 1687.00 | 3.261 | 3.223–3.302 | 516.1414 |
| read / CopyGuard_checked | checked | 1683.0 | 1687.00 | 1651.096 | 1628.189–1676.704 | 1.0193 |
| read / LZ_inline_checked | checked | 1683.0 | 1685.00 | 1579.016 | 1559.897–1602.821 | 1.0659 |
| read / String_checked | checked | 1683.0 | 1687.00 | 1641.916 | 1625.116–1659.971 | 1.0250 |
| read / String_trusted | trusted | 1683.0 | 1687.00 | 118.456 | 116.212–121.002 | 14.2078 |
| write / borrowed_checked | checked | 1683.0 | 1687.00 | 59.562 | 57.404–62.461 | 28.2563 |
| write / LZ_inline_checked | checked | 1683.0 | 1685.00 | 61.120 | 59.553–63.128 | 27.5361 |
| write / String_checked | checked | 1683.0 | 1687.00 | 55.382 | 54.171–57.193 | 30.3891 |
| write / String_trusted | trusted | 1683.0 | 1687.00 | 53.157 | 52.330–54.163 | 31.6611 |

## Строки: повторяющийся UTF-8, 2048 B / Strings: repeated UTF-8, 2048 B

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| read / LZ_inline_checked | checked | 2048.0 | 55.00 | 4934.907 | 4877.011–5000.467 | 0.4150 |
| read / String_checked | checked | 2048.0 | 2052.00 | 2249.297 | 2213.111–2299.330 | 0.9105 |
| write / LZ_inline_checked | checked | 2048.0 | 55.00 | 177.482 | 171.874–185.817 | 11.5392 |
| write / String_checked | checked | 2048.0 | 2052.00 | 62.730 | 61.468–64.427 | 32.6480 |

## Optional: N=1 / Optional: N=1

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| read / default_checked | checked | 4.5 | 5.50 | 55.088 | 54.387–55.879 | 0.0817 |
| read / opt_table_checked | checked | 4.5 | 4.50 | 36.215 | 35.768–36.815 | 0.1243 |

## Optional: N=2 / Optional: N=2

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| read / default_checked | checked | 5.0 | 6.00 | 77.947 | 77.168–78.808 | 0.0641 |
| read / opt_table_checked | checked | 5.0 | 6.00 | 53.369 | 52.769–54.085 | 0.0937 |

## Optional: N=3 / Optional: N=3

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| read / default_checked | checked | 5.5 | 6.50 | 95.502 | 94.440–96.674 | 0.0576 |
| read / opt_table_checked | checked | 5.5 | 6.50 | 59.998 | 59.324–60.765 | 0.0917 |

## Optional: N=5 / Optional: N=5

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| read / default_checked | checked | 6.5 | 8.34 | 132.404 | 130.805–134.329 | 0.0491 |
| read / opt_table_checked | checked | 6.5 | 7.50 | 77.071 | 75.582–79.740 | 0.0843 |

## Optional: N=10 / Optional: N=10

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| read / default_checked | checked | 9.0 | 12.25 | 239.633 | 237.149–242.311 | 0.0376 |
| read / opt_table_checked | checked | 9.0 | 11.00 | 138.455 | 135.834–143.657 | 0.0650 |

## Общий QPS: 1024 байта ASCII / Whole QPS: 1024 ASCII bytes

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| read / default_checked | checked | 1024.0 | 1028.00 | 167.951 | 165.443–170.601 | 6.0970 |
| read / qps_Q16_checked | checked | 1024.0 | 981.00 | 516.283 | 508.845–524.376 | 1.9834 |
| write / default_checked | checked | 1024.0 | 1028.00 | 35.676 | 35.310–36.067 | 28.7026 |
| write / qps_Q16_checked | checked | 1024.0 | 981.00 | 520.645 | 514.209–528.319 | 1.9668 |

## Выход: новая Vec и статический RT / Output: fresh Vec and static RT

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| write / heap_checked | checked | 64.0 | 64.00 | 82.211 | 81.355–83.182 | 0.7785 |
| write / rt_checked | checked | 64.0 | 64.00 | 22.865 | 22.609–23.155 | 2.7990 |

## Валидация без создания объекта / Validation without constructing a value

| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---|---:|---:|---:|---:|---:|
| validate / numbers_checked | checked | 64.0 | 64.00 | 86.733 | 85.758–87.946 | 0.7379 |

## Как читать результаты / Interpretation

Строковый набор содержит ровно 1024 символа, но размер M указан в байтах UTF-8. Доверенное чтение String и checked-чтение выполняют разную работу. Новая CopyGuard тоже владеет выделенным буфером; это не замер reuse. Для фразы LZ выбрал raw-fallback: разница в 2 B обусловлена заголовком. Отдельный набор повторяющейся кириллицы демонстрирует настоящее LZ и сравнивается с собственной String-базой. Общий QPS сравнивается с обычным кодеком на одинаковых ASCII-данных, а не с UTF-8-группой.

Optional-поля — одинаковые u8. Для каждого N состояния равновероятны; M и wire являются средними по набору. Улучшение opt_table зависит от N и режима чтения; все измеренные варианты приведены, включая замедления. RT сравнивается с новой Vec; подготовленный выход дополнительно представлен numbers_8/write/raw_checked.

The UTF-8 set has exactly 1024 characters, while M counts encoded UTF-8 bytes. Fresh CopyGuard owns an allocated buffer; it is not reuse. The phrase set uses LZ raw fallback (2 B shorter header); the separate repeated-Cyrillic set measures genuine compression with its own String baseline. Whole QPS and its default baseline share ASCII data. Optional states are uniform and sizes are fixture means. All modes, including regressions, are retained. RT acquisition/release is included; reusable output has its own numbers_8 baseline.

Кардиограмма показывает 200 средних внутри пакетов Criterion. Это не отдельные задержки сообщений, не сеть и не p99. Для операции с разным контрактом значение P не означает одинаковый объём копирования.

## Воспроизводимость / Reproduction

[Исходники измеренного снимка](performance/measured-sources.zip) · [SHA-256 исходников](performance/source-before.json) · [Оригинальный Criterion JSON](performance/criterion-raw.zip) · [Лог](performance/run.log) · [Лимит и SHA-256 бинарного файла](performance/memory.json).

The source archive retains exact measured bytes; paths in manifests are relative to its root. The executable SHA-256 identifies the local release build, not a promise of byte-identical binaries from different source paths/toolchains. No new performance guarantee follows from this one machine and dataset.

```powershell
pwsh -ExecutionPolicy Bypass -File examples/tools/measure.ps1 -ResultDirectory examples/results/my-run
python examples/tools/build-report.py examples/results/my-run
```

### Снимки / Profiles

- main: 35 cases · [source](performance/measured-sources.zip) · [raw](performance/criterion-raw.zip) · [SHA-256](performance/source-before.json)
- control-1: 4 cases · [source](performance/control-1/measured-sources.zip) · [raw](performance/control-1/criterion-raw.zip) · [SHA-256](performance/control-1/source-before.json)
