# Load-testing: full smoke comparison

**Method:** one WebSocket, four requests in flight, batch 1/64, 1 s warm-up, 2 s measurement, same-host k6. CPU numbers are **full-run processor seconds**; memory values are separate per-process peak RSS, not necessarily simultaneous. These are not maximum server capacity numbers.

| Codec | Batch | Replies/s | Records/s | App MiB/s | Avg RTT ms | Server CPU s | k6 CPU s | Server RAM MiB | k6 RAM MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| json | 1 | 14,514.0 | 14,514 | 3.48 | 0.255 | 2.047 | 3.969 | 10.22 | 59.16 |
| msgpack | 1 | 10,566.5 | 10,566 | 1.88 | 0.355 | 1.484 | 2.906 | 10.22 | 60.00 |
| protobuf | 1 | 15,157.5 | 15,158 | 2.73 | 0.245 | 1.891 | 3.797 | 10.22 | 58.89 |
| qsl | 1 | 6,689.5 | 6,690 | 0.71 | 0.565 | 1.047 | 2.062 | 10.13 | 59.75 |
| qsl-legacy | 1 | 12,387.0 | 12,387 | 2.35 | 0.303 | 1.734 | 3.312 | 10.20 | 58.48 |
| json | 64 | 6,149.5 | 393,568 | 95.18 | 0.607 | 1.672 | 3.438 | 10.32 | 58.64 |
| msgpack | 64 | 8,610.0 | 551,040 | 99.21 | 0.432 | 1.906 | 3.844 | 10.32 | 60.78 |
| protobuf | 64 | 9,858.0 | 630,912 | 114.88 | 0.376 | 2.000 | 4.297 | 10.32 | 58.08 |
| qsl | 64 | 11,513.5 | 736,864 | 75.93 | 0.314 | 1.703 | 3.734 | 10.25 | 60.36 |
| qsl-legacy | 64 | 6,469.5 | 414,048 | 77.02 | 0.582 | 1.375 | 2.891 | 10.32 | 57.56 |

## Important QSL version caveat: WebSocket and strings

**Historical result, not a measurement of the current string path.** The WebSocket smoke tests above used the **earlier QSL string-handling implementation**. The subsequent optimizations based on borrowed `&[u8]`, `Utf8Guard`, and other specialized string representations **were not part of the measured path**. Thus the QSL results above, including its `batch=1` deficit, **must not be presented as measurements of the updated string implementation**.

| Generation / test | String representation | What the measurement establishes |
|---|---|---|
| Historical WebSocket smoke (`batch=1/64`) | Earlier string-handling path | End-to-end throughput, RTT, CPU and RAM of that version through k6/WebSocket only |
| Later UTF-8 Criterion tests | `String`, borrowed `&[u8]`, `Utf8Guard`, `InlineString`, decode into destination | Costs of individual read/write operations for the specified inputs and conditions |
| WebSocket rerun after the optimizations | Not present in the supplied data | **No measured end-to-end throughput improvement yet** |

**What changed.** `binary_opt=true` exposes string bytes as borrowed `&[u8]`, avoiding a separately allocated `String` during decoding. `binary_opt=guard` uses `Utf8Guard<'a>`, a borrowed zero-copy UTF-8 view with an explicit validity contract, **not** a newly allocated heap buffer. Both can avoid ownership/allocation costs on their respective paths. An application may choose to copy the bytes afterward, but copying is not inherent to `Utf8Guard`. Fixed strings remain `[u8; N]`, while `InlineString` owns bytes within the Rust value. The alternatives have different lifetime and safety requirements.

**Interpretation.** Later Criterion measurements show meaningful owned-versus-borrowed decoding differences, but **do not prove an equivalent WebSocket RPS or RTT improvement**. Transport framing, k6 load generation, batching, and other fields still contribute to end-to-end latency. Establishing that improvement requires rerunning the campaign with the new QSL string path, identical schemas, fixtures, environment and load parameters.

## Why more MiB/s does not imply the best codec

JSON and Protobuf produce different encoded payload sizes. Therefore MiB/s alone cannot rank codec efficiency: at batch=64 QSL handled more **records**/s while Protobuf transferred more **wire** MiB/s.

## How to interpret CPU and RAM

CPU totals measured over the entire process lifetime, not just the 2 s measurement window. A faster codec may process more total records and accumulate more CPU time. Divide CPU by actual completed work only for a carefully aligned measurement window; the archived full-run CPU and measured-window records are not exactly aligned.

## Limitations

- WebSocket RTT does not isolate serialization CPU cost.
- Two seconds is too short for stable peak throughput and p99.
- One connection does not measure scalability.
- Load-generator capacity influences results.
- For micro-operations, compare identical schemas and buffers with Criterion.

## Raw data

[load-smoke.csv](data/load-smoke.csv) · [Criterion CSV](data/all-estimates.csv)