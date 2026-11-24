# QSL All Features

QSL 0.4.0 only. This benchmark sweeps schema attributes and generated Rust paths by workload. It ran <!-- TOTAL -->.

[Safety QSL](QSL-SAFETY.md) · [Interactive tables](qsl-matrix.html) · [Four-codec comparison](OPTIMIZED.md)

## Fastest calibrated profiles

Each row shows one candidate selected by the calibration median for that workload and operation. The displayed time is the mean of the medians from two confirmation passes on different records, with the second pass in reverse order. A checked and a trusted winner can use different wire formats. [All calibration timings](results/qsl-matrix/calibration.csv) · [Confirmed timings, baselines and intervals](results/qsl-matrix/confirmed.csv)

<!-- TABLE -->

Frame size is the mean of 64 frames. Object size is Rust's `size_of` on this target. It includes inline buffers and excludes memory behind pointers. The write tests use initialized, reusable caller storage. The owned read tests include dropping the decoded object.

<!-- TRADEOFFS -->

## Native and cached paths

<!-- EXTRA -->

The native overlay returns a reference into aligned frame memory. Native copying returns an owned value. The cache and slot operations in this table time the generated writer followed by a copy to the caller buffer, and the slot operation releases its lease per record.

## Coverage

| Feature | Sweep |
|---|---|
| Aggressive representations | All 16 true/false masks for `shift`, `bitmap`, `pointing` and `group` on fixed scalars, optional scalars and optional strings. Each family also has a plain `aggressive` candidate, an `auto` baseline and compact controls. |
| Packing and layout | `compact`, `alignment`, `auto`, `auto[wire=1]`, and group candidates for the portable and Kaby Lake generator profiles |
| Presence | PS, static and legacy OT, optional bitmap, PS + OT, bitmap + OT, compact bitmap, the PS + bitmap combination and state-bit quanta Q1 through Q16 |
| Dyn v1 | `line` with widths 16/32/64, and both index modes with widths 32/64. All use revision 1 and identical optional scalar values. |
| Quantization | QPS with Q2/4/8/16 on fixed and optional layouts, including OT in the latter. The generator can choose a raw fallback. Interpretation uses the reported plan. |
| Strings and arrays | Heap String, StackString, shifted StackString, borrowed UTF-8, inline and heap QSL-LZ, optional vectors with heap and inline storage, shifted lengths and object bitmaps. The QSL-LZ workload includes both periodic and noisy text. |
| Native and memory | Whole-record transmutation copy and aligned overlay, a named native region, generated `loyal_cache` and the generated QSL-RT lease path with 1 and 16 slots |
| API controls | Safety entry points, inline policies `hint`/`always`/`never`, and field-level defaults, integer steps and sets |

Aliases share the same parser path. Numeric parameters such as byte caps, revision and slots have specific values as recorded in the [profile manifest](results/qsl-matrix/profiles.json). This is a sweep of the listed attributes on named workloads. The parser and generator decide support for each attempted combination.

<!-- PLANS -->

## Reproduction and environment

<!-- ENV -->

Each phase prewarms 64 typed records and their validated frames. The harness checks the checked and trusted writers for byte equality and checks both decoders by encoding their outputs back to the same frame. Trusted paths use immutable validated frames and adequate output space. Aligned backing storage also supplies initialized padding for generated readers that require it.

Run from the benchmark directory with Windows PowerShell, Node.js and Rust.

```powershell
cargo fetch --locked --manifest-path qsl-all-features/Cargo.toml
./tools/run-qsl-matrix.ps1
node tools/qsl-matrix-report.mjs
```

[Generated QSL schemes](results/qsl-matrix/schemas.qsl) · [Build and run environment](results/qsl-matrix/environment.json) · [Criterion raw samples](results/qsl-matrix/criterion.json.gz)
