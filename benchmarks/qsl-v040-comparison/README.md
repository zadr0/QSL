# QSL 0.4.0 serialization benchmarks

QSL, Protobuf through Prost, Cap’n Proto and Bincode encode and decode the same warmed fixtures. The suite covers a polymorphic API, socket events, heartbeats, bounded integers and borrowed text. Each codec contributes its fastest calibration candidate for each operation, followed by two independent confirmation passes.

| Report | Contents |
|---|---|
| [Results and method](OPTIMIZED.md) | Fresh output allocations, ready output buffers, owned decoding, zero-copy views and batches of 1, 8, 64 and 256 records |
| [Interactive dashboard](dashboard.html) | Confirmed latency, selected codec profiles, frame sizes and allocation counts. Download the HTML file to open it in a browser. |
| [Historical benchmark series](HISTORY.md) | Earlier Dyn, k6, TCP, layout, transmutation, StackString and QSL-LZ measurements |

All measured QSL schemas use automatic layout. The timed trusted QSL paths require immutable, prevalidated fixtures. The report documents those requirements and the exact contract for each operation.

## Reproduce on Windows

Use x86-64 Windows PowerShell, Rust and Node.js 20 or later. Run from this directory. The scripts use logical core 2. Archive an existing `results/optimized` or `results/optimized-batch` directory before recording a new run.

```powershell
./tools/setup-capnp.ps1
cargo fetch --locked --manifest-path optimized/Cargo.toml
cargo fetch --locked --manifest-path optimized-batch/Cargo.toml
./tools/run-optimized.ps1
./tools/run-optimized-batch.ps1
node tools/optimized-report.mjs
node tools/serve-reports.mjs 7360
```

Open [the local dashboard](http://127.0.0.1:7360/dashboard.html). The reporter generates Russian and English dashboards. The repository's `dashboard.html` is the English version.

[Single-record CSV](results/optimized/confirmed.csv) · [Batch CSV](results/optimized-batch/confirmed.csv) · [Measured QSL schemas](results/optimized/generated.qsl)
