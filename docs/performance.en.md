# Fresh measurements: QSL only

> Historical profile: `opt_table` here labels the former mixed length graph. `[predict_seq,opt_table]` preserves its format; new standalone `[opt_table]` dispatches readers by tag. [Separation and migration](optional-modes.en.md).

[Русский](performance.ru.md) · [Architecture](architecture.en.md)

QPS profile: four cases of one schema, 2026-11-02: prepared reads and writes with/without QPS. No other codecs, sockets or k6 were run.

| Operation | Variant | ns/frame | 99% CI, ns |
|---|---|---:|---:|
| `deserialize_into` | QSL + opt_table | 15.5670 | 15.4188–15.7210 |
| `deserialize_into` | QSL + QPS Q16 + opt_table | 9.5583 | 9.4735–9.6470 |
| `serialize_into` | QSL + opt_table | 7.4147 | 7.3280–7.5082 |
| `serialize_into` | QSL + QPS Q16 + opt_table | 13.6984 | 13.5434–13.8676 |

QPS changes decoding time by -38.6% and encoding by +84.7% relative to current raw opt_table. Reads are faster and writes cost more in this profile. This is neither a comparison with historical runs nor a universal ranking.

Windows 11 Pro x86-64; i5-7400, 4 cores / 4 threads; RAM 24 GiB at 2133 MHz. Rust 1.98.0 / LLVM 22.1.8; Criterion 0.8.2; release opt-level=3 with debug info, RUSTFLAGS unset. Affinity was not pinned and the OS was not isolated.

Each case uses 15 s warm-up, 10 s measurement, 200 samples and 99% CI. All 16 optional masks are cycled. Wire and final-output buffers are 64 B aligned; native source objects for writes use Vec's natural alignment. Preparation, allocation and validation are excluded; frame selection and black_box are included. This is not a 1024-character string or network throughput test.

Model: i64(-10..10), u64(0..15), optional u64(0..1000), i64(-100..100), u64(7..7), full u64. Raw: 17–49 B; QPS: 3–13 B. Prepared QPS reads require 16 B of initialized private padding, never transmitted. Both Rust objects occupy 80 B. Both tests use 128 B Wire capacity; this experiment does not establish lower RSS.

Measured process and descendants: 8 GiB limit; peak job commit 6.83 MiB. Compilation occurs before applying this measured-process limit. Source hashes were captured before the run and verified afterwards. The executable is identified by SHA-256.

Selected x86-64 release QPS wrappers have no conditional jumps or calls; the raw reader has one indirect event call. This does not describe the entire checked API. Instruction counts are not cycle counts. Q2/Q4/Q8 have golden-vector and roundtrip coverage; only Q16 has fresh timing.

```powershell
# Matching dev-qsl checkout; only four cases; 8 GiB measured-process limit.
.\qsl-bench\tools\run-payload.ps1 -Benchmark current_qsl -ManifestPath .\qsl-bench\current\Cargo.toml -ResultDirectory .\qsl-bench\target\criterion-qps-v2-repeat
cargo test --manifest-path qsl-bench/current/Cargo.toml --locked
cargo rustc --manifest-path qsl-bench/current/Cargo.toml --example asm_current --release --locked -- --emit=asm
```

[JSON](data/current-qsl.json) · [CSV](data/current-qsl.csv) · [Raw samples](data/qsl-current-raw/) · [Log](data/qsl-current-run.log) · [ASM](data/qsl-current-asm/) · [Harness](../reference/current-benchmark/) · [Hashes](../reference/current-source-manifest.json)

Historical all-estimates, profiles and smoke files remain archived; the fresh chart uses only these four observations.

## Separate baseline control

[default vs opt_table](opt-table-control.en.md): N=1/2/3/5/10 optional u64 fields after one required u32; twenty paired QSL-only read/write cases. QPS is off in that group. Its shape and working set differ; absolute times are not compared across the groups.
