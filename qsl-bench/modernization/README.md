# Focused deserialization measurements / Точечные замеры

Four cases only: String versus inline owned UTF-8, and default numeric versus Q16 prepared decoding. No k6, socket matrix or hot-path metrics sampler.

```powershell
pwsh -File qsl-bench/modernization/run.ps1
python qsl-bench/modernization/report.py
```

Run from dev-qsl (or the public repository root) in PowerShell 7. The runner builds first, starts a waiting executable, attaches an 8 GiB Job Object, then opens the start gate. Fixture preparation, frame validation, ring warm-up and Criterion measurements follow. The default 5% idle gate was waived by the user; ambient noise remains part of this run.

| Case | T (ns) | 99% CI (ns) | M (useful B) | Wire B | P (B/ns) |
|---|---|---|---|---|---|
| String | 130.375 | 122.153–140.367 | 2048 | 2052 | 15.709 |
| StackString | 63.202 | 62.554–63.893 | 2048 | 2052 | 32.404 |
| Numeric default | 2.679 | 2.667–2.693 | 24 | 24 | 8.958 |
| Scalar Q16 | 4.147 | 4.082–4.228 | 24 | 5 | 5.788 |

15 s warm-up; 10 s measurement; 200 samples; 99% confidence. Ring: 1024 frames. Windows 11 x86-64, i5-7400, RAM 24 GiB at 2133 MHz as reported by the developer. Rust 1.98.0, LLVM 22.1.8, thin LTO and one codegen unit. Peak committed job memory: 13.80 MiB; not RSS.

The string is 1024 Unicode scalars, all two-byte UTF-8: 2048 useful bytes. This separate stand explicitly sets STACK_LIMIT to 2048; the library default remains 256. Trusted reads exclude validation. String construction and destruction are included. Buffer construction, selection ring preparation and allocator/CPU counters are outside timing; fixture indexing and black_box remain inside. Empty-loop overhead is not subtracted.

The 25 ns string target was not met. The within-run String/StackString ratio is about 2.06; it is not a repeatability guarantee. String has 15% flagged outliers. Q16 saves wire bytes but costs more decoding time for this schema. No claim is made about full CPU idleness, I-cache counters or network throughput.

The separate release probes contain 5 instructions for default and 26 for Q16; neither has conditional jumps or calls. This is one required-only numerical specialization, not a claim about every optional/QPS path.

[Interactive report](report.html) · [Machine data](results/2026-11-06/summary.json) · [CSV](results/2026-11-06/summary.csv) · [Assembly](results/2026-11-06/assembly.txt) · [Measured source manifest](results/2026-11-06/source-manifest.json)

The source directory under results is the measured snapshot. Subsequent parser/linter diagnostics may change without changing the measured codec; the recorded binary hash is authoritative for this run.


## Typed register correction / Исправление ширины

[Two numerical reruns / Два повторных числовых замера](results/2026-11-06-register-width/README.md): explicit u8–u128 generated loads, five exact-read ASM probes, source/binary hashes. Default 2.788 ns; Q16 4.304 ns. No speedup is established; the measured decoder probes match the earlier instructions. The four-case table above retains its original snapshot.
