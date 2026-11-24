# QSL All Features

QSL 0.4.0 only. This benchmark sweeps schema attributes and generated Rust paths by workload. It ran 124 schema configurations, 123 measured profiles, 708 calibration cases and 285 independently measured finalists.

[Safety QSL](QSL-SAFETY.md) · [Interactive tables](qsl-matrix.html) · [Four-codec comparison](OPTIMIZED.md)

## Fastest calibrated profiles

Each row shows one candidate selected by the calibration median for that workload and operation. The displayed time is the mean of the medians from two confirmation passes on different records, with the second pass in reverse order. A checked and a trusted winner can use different wire formats. [All calibration timings](results/qsl-matrix/calibration.csv) · [Confirmed timings, baselines and intervals](results/qsl-matrix/confirmed.csv)

| Workload | Operation | Selected profile | ns / record | Frame B | Object B |
| --- | --- | --- | --- | --- | --- |
| arrays | Checked read | `inline` | 125.8 | 32.75 | 76 |
| arrays | Checked write | `inline` | 7.91 | 32.75 | 76 |
| arrays | Trusted read | `inline` | 24.68 | 32.75 | 76 |
| arrays | Trusted write | `inline_shifted` | 7.93 | 32.75 | 72 |
| domain | Checked read | `compact` | 4.86 | 2 | 8 |
| domain | Checked write | `auto` | 3.91 | 5 | 8 |
| domain | Trusted read | `auto` | 6.76 | 5 | 8 |
| domain | Trusted write | `auto` | 3.04 | 5 | 8 |
| dyn | Checked read | `line_64` | 35.26 | 39.5 | 16 |
| dyn | Checked write | `line_16` | 5.57 | 21.5 | 16 |
| dyn | Trusted read | `line_64` | 7.8 | 39.5 | 16 |
| dyn | Trusted write | `line_16` | 4.44 | 21.5 | 16 |
| fixed | Checked read | `inline_always` | 4.09 | 6 | 16 |
| fixed | Checked write | `aggressive_shift1_bm0_point1_group1` | 3.19 | 16 | 16 |
| fixed | Trusted read | `aggressive_shift0_bm0_point1_group1` | 2.47 | 16 | 16 |
| fixed | Trusted write | `auto` | 2.99 | 16 | 16 |
| layout | Checked read | `auto` | 3.72 | 11 | 16 |
| layout | Checked write | `auto_wire_v1` | 3.56 | 11 | 16 |
| layout | Trusted read | `declared` | 3.22 | 11 | 24 |
| layout | Trusted write | `auto_wire_v1` | 3.5 | 11 | 16 |
| native_region | Checked read | `fields` | 4.59 | 24 | 24 |
| native_region | Checked write | `fields` | 5.14 | 24 | 24 |
| native_region | Trusted read | `named_region` | 2.88 | 24 | 24 |
| native_region | Trusted write | `fields` | 5.06 | 24 | 24 |
| native | Checked read | `transmute_copy` | 3.15 | 16 | 16 |
| native | Checked write | `transmute_copy` | 4.75 | 16 | 16 |
| native | Trusted read | `transmute_copy` | 2.48 | 16 | 16 |
| native | Trusted write | `fields` | 4.68 | 16 | 16 |
| noise_1024 | Checked read | `qsl_lz_inline` | 107.72 | 774.5 | 1036 |
| noise_1024 | Checked write | `stack_string` | 20.37 | 776 | 1036 |
| noise_1024 | Trusted read | `stack_string` | 30.8 | 776 | 1036 |
| noise_1024 | Trusted write | `stack_string` | 20.5 | 776 | 1036 |
| optional_text | Checked read | `aggressive_default` | 81.3 | 104 | 1036 |
| optional_text | Checked write | `aggressive_shift1_bm0_point0_group0` | 10.27 | 104 | 1032 |
| optional_text | Trusted read | `aggressive_shift0_bm0_point1_group1` | 19.06 | 104 | 1036 |
| optional_text | Trusted write | `aggressive_shift0_bm0_point1_group1` | 9.62 | 104 | 1036 |
| periodic_1024 | Checked read | `shifted_stack_string` | 754.4 | 776 | 1032 |
| periodic_1024 | Checked write | `stack_string` | 19.09 | 776 | 1036 |
| periodic_1024 | Trusted read | `stack_string` | 29.05 | 776 | 1036 |
| periodic_1024 | Trusted write | `heap_string` | 20.76 | 776 | 32 |
| presence | Checked read | `wire_and_object_bitmap` | 25.43 | 4.5 | 12 |
| presence | Checked write | `bitmap_ot` | 5.52 | 4.5 | 12 |
| presence | Trusted read | `ps_ot_object_bitmap` | 4.14 | 3.5 | 12 |
| presence | Trusted write | `compact_bitmap` | 3.97 | 4.5 | 12 |
| quantized_fixed | Checked read | `qps_q16` | 54.23 | 6 | 16 |
| quantized_fixed | Checked write | `qps_q16` | 5.86 | 6 | 16 |
| quantized_fixed | Trusted read | `qps_q8` | 5.91 | 6 | 16 |
| quantized_fixed | Trusted write | `qps_q16` | 4.72 | 6 | 16 |
| quantized_optional | Checked read | `qps_q8_ot` | 51.4 | 4.5 | 12 |
| quantized_optional | Checked write | `qps_q8_ot` | 11.81 | 4.5 | 12 |
| quantized_optional | Trusted read | `qps_q16_ot` | 9.32 | 4.5 | 12 |
| quantized_optional | Trusted write | `qps_q8_ot` | 10.82 | 4.5 | 12 |

Frame size is the mean of 64 frames. Object size is Rust's `size_of` on this target. It includes inline buffers and excludes memory behind pointers. The write tests use initialized, reusable caller storage. The owned read tests include dropping the decoded object.

On this target, shifted lengths changed the inline-array object from **76 B** to **72 B**, and the inline-string object from **1036 B** to **1032 B**. Their respective wire sizes stayed the same. For periodic strings, average frame size went from **776 B** with heap String to **37.25 B** with inline QSL-LZ. Those averages include the 25% of fixtures with the text absent.

## Native and cached paths

| Workload | Profile | Operation | ns / record | Frame B | Object B |
| --- | --- | --- | --- | --- | --- |
| fixed | `loyal_cache_32` | CachedFrame write + copy | 3.16 | 16 | 16 |
| fixed | `rt_slots_1` | QSL-RT lease write + copy | 18.16 | 16 | 16 |
| fixed | `rt_slots_16` | QSL-RT lease write + copy | 18.53 | 16 | 16 |
| native | `transmute_copy` | Native copy | 3.09 | 16 | 16 |
| native | `transmute_overlay` | Checked read | 3.35 | 16 | 16 |
| native | `transmute_overlay` | Checked write | 4.71 | 16 | 16 |
| native | `transmute_overlay` | Trusted read | 2.66 | 16 | 16 |
| native | `transmute_overlay` | Trusted write | 4.8 | 16 | 16 |

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

QPS was active in **4** profiles. Code for a BMI2 PEXT path was found in **1** generated profiles. The profile manifest retains these plan facts for each candidate.

- `borrowed_field` · QSL at byte 52: unsupported field modifier `borrow`

## Reproduction and environment

Intel(R) Core(TM) i5-7400 CPU @ 3.00GHz, Windows 10.0.26200, rustc 1.98.0 (88d9e12ae 2026-08-18). Mean background load in logical cores was tune 1.57, confirm-1 1.36, confirm-2 1.75.

Each phase prewarms 64 typed records and their validated frames. The harness checks the checked and trusted writers for byte equality and checks both decoders by encoding their outputs back to the same frame. Trusted paths use immutable validated frames and adequate output space. Aligned backing storage also supplies initialized padding for generated readers that require it.

Run from the benchmark directory with Windows PowerShell, Node.js and Rust.

```powershell
cargo fetch --locked --manifest-path qsl-all-features/Cargo.toml
./tools/run-qsl-matrix.ps1
node tools/qsl-matrix-report.mjs
```

[Generated QSL schemes](results/qsl-matrix/schemas.qsl) · [Build and run environment](results/qsl-matrix/environment.json) · [Criterion raw samples](results/qsl-matrix/criterion.json.gz)
