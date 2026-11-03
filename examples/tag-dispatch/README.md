# Отдельный opt_table: контроль / Standalone opt_table control

10 checked-read cases, N=1/2/3/5/10; default and tag dispatch use identical frames. 15 s warmup, 10 s measurement, 200 samples, 99% CI. Windows 11, i5-7400, 24 GiB / 2133 MHz. This profile predates the newly requested OS-idle/instrumentation-noise controls and is not the strict cross-protocol experiment.

Таблица по тегам сравнивается с default на одинаковых кадрах. Проход ещё не содержит нового контроля простоя ОС и влияния метрик. P использует исходные полезные байты. Снимок исходников сохранён до последующего исправления генерации пустого combined-хвоста; измеренные случаи его не используют.

| N | Default, ns | opt_table, ns | Default / table | P table, B/ns |
|---|---:|---:|---:|---:|
| 1 | 56.583 | 56.350 | 1.004 | 0.0799 |
| 2 | 80.726 | 87.611 | 0.921 | 0.0571 |
| 3 | 95.817 | 123.984 | 0.773 | 0.0444 |
| 5 | 133.840 | 153.428 | 0.872 | 0.0424 |
| 10 | 240.804 | 249.750 | 0.964 | 0.0360 |

[Raw JSON](measurements.json) · [Criterion samples](criterion-raw.zip) · [Measured sources](measured-sources.zip) · [Memory and binary SHA](memory.json)
