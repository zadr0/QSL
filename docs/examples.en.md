# Examples and fresh measurements

> Historical profile: `opt_table` here labels the former mixed length graph. `[predict_seq,opt_table]` preserves its format; new standalone `[opt_table]` dispatches readers by tag. [Separation and migration](optional-modes.en.md).

[Русский](examples.ru.md) · [Examples](../examples/README.md) · [Interactive](../examples/performance.html) · [CSV](../examples/performance/measurements.csv)

Seven executable programs cover basic I/O, strings, optional fields, QPS, RT, nested templates/hashes and checked errors. All use actual generated types; commands and schemas are under examples.

39 operations: 35 main cases and 4 additional genuine-LZ controls. Source profiles remain separate, QSL sources match, and each comparison group uses one binary. Windows 11 Pro x86-64, i5-7400, 24 GiB / 2133 MHz; Criterion 0.8.2, 15 s warm-up, 10 s measurement, 200 samples, 99% CI.

| Operation | Wire, B | T, ns | 99% CI, ns | P, B/ns |
|---|---:|---:|---:|---:|
| numbers_8/read/raw_checked | 64.00 | 90.354 | 89.308–91.544 | 0.7083 |
| numbers_8/write/raw_checked | 64.00 | 10.278 | 10.196–10.367 | 6.2268 |
| numbers_8/read/qps_Q16_checked | 4.00 | 133.410 | 131.880–135.291 | 0.4797 |
| numbers_8/write/qps_Q16_checked | 4.00 | 9.942 | 9.844–10.052 | 6.4375 |
| strings_utf8_1024/read/String_checked | 1687.00 | 1641.916 | 1625.116–1659.971 | 1.0250 |
| strings_utf8_1024/read/String_trusted | 1687.00 | 118.456 | 116.212–121.002 | 14.2078 |
| strings_utf8_1024/read/borrowed_checked | 1687.00 | 1545.671 | 1528.485–1565.802 | 1.0888 |
| strings_utf8_1024_uniform/read/String_checked | 2052.00 | 2249.297 | 2213.111–2299.330 | 0.9105 |
| strings_utf8_1024_uniform/read/LZ_inline_checked | 55.00 | 4934.907 | 4877.011–5000.467 | 0.4150 |
| buffers/write/heap_checked | 64.00 | 82.211 | 81.355–83.182 | 0.7785 |
| buffers/write/rt_checked | 64.00 | 22.865 | 22.609–23.155 | 2.7990 |

P=M/T counts original useful bytes. Trusted borrowing is not payload-copy bandwidth. Full default/opt_table pairs for N=1/2/3/5/10, writes, raw samples and exact snapshots are in the [report](../examples/PERFORMANCE.md).

The phrase LZ set selects raw fallback and saves only 2 B in its header. The repeated-Cyrillic control genuinely shrinks from 2052 to 55 B, but checked restoration costs more. Templates and hashes run during analysis; constant reads are not assigned fictional timings.

[PNG](../examples/performance/overview.png) · [SVG](../examples/performance/overview.svg) · [JSON](../examples/performance/measurements.json)
