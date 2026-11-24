# QSL 0.4.0 codec benchmark

QSL, Protobuf, Cap’n Proto and Bincode process the same logical values with optimized Rust builds. Each codec contributes one configuration per workload and operation. The fastest calibration median selects that configuration. Two later runs measure the winners on different records.

[Dashboard](dashboard.html) · [Timing CSV](results/optimized/confirmed.csv) · [Historical benchmarks](HISTORY.md)

The timing tables compare selected code paths on valid fixtures. QSL's `_trusted` profiles verify the records before the timer, while the other codec adapters retain their parsing checks inside the timer. An ingress comparison requires a shared acceptance contract for untrusted input.

## Results

**5 confirmed leads, 2 confirmed deficits and 7 inconclusive comparisons** for QSL.

All timings below are in **ns per record**. Each cell is the mean of the two independent Criterion medians.

### Allocate and encode

| Workload | QSL profile | QSL | Protobuf | Cap’n Proto | Bincode | QSL result |
|---|---|---:|---:|---:|---:|---|
| Polymorphic API | `qsl_auto_bounded_compact_bitmap_trusted` | 160.63 | 394.21 | 134.78 | 188.76 | unresolved |
| Socket event | `qsl_auto_bounded_aligned_bitmap_trusted` | 196.67 | 448.55 | 147.78 | 197.05 | behind |
| Four bounded integers | `qsl_auto_compact` | 71.54 | 88.2 | 97.11 | 68.97 | unresolved |
| Heartbeat | `qsl_auto_compact_bitmap_trusted` | 97.08 | 128.31 | 105.53 | 100.95 | unresolved |

### Encode into a ready buffer

| Workload | QSL profile | QSL | Protobuf | Cap’n Proto | Bincode | QSL result |
|---|---|---:|---:|---:|---:|---|
| Polymorphic API | `qsl_auto_bitmap_trusted` | 55.55 | 265.47 | 52.92 | 58.56 | unresolved |
| Socket event | `qsl_auto_bounded_aligned_bitmap_trusted` | 62.53 | 296.51 | 61.51 | 68.13 | unresolved |
| Four bounded integers | `qsl_auto_pointing_trusted` | 11.25 | 13.9 | 46.71 | 10.81 | unresolved |
| Heartbeat | `qsl_auto_bounded_aligned_bitmap_trusted` | 30.73 | 67.23 | 46.57 | 27.39 | behind |

### Decode to an owned value

| Workload | QSL profile | QSL | Protobuf | Cap’n Proto | Bincode | QSL result |
|---|---|---:|---:|---:|---:|---|
| Polymorphic API | `qsl_auto_bounded_aligned_bitmap_trusted` | 311.28 | 748.47 | 612.05 | 462.59 | lead |
| Socket event | `qsl_auto_bitmap_trusted` | 345.15 | 841.03 | 636.52 | 441.99 | lead |
| Four bounded integers | `qsl_auto_trusted` | 15.67 | 30.85 | 66.89 | 15.19 | unresolved |
| Heartbeat | `qsl_auto_compact_bitmap_trusted` | 86.32 | 146.22 | 175.83 | 115.52 | lead |

### Zero-copy text view

| Workload | QSL profile | QSL | Protobuf | Cap’n Proto | Bincode | QSL result |
|---|---|---:|---:|---:|---:|---|
| 128-byte text | `qsl_auto_lifetime_aligned_trusted` | 9.31 | 89.21 | 129.5 | 42.34 | lead |
| 2,048-byte text | `qsl_auto_lifetime_trusted` | 9.3 | 198.84 | 268.53 | 157.13 | lead |

## Batch results

**19 confirmed leads, 2 confirmed deficits and 35 inconclusive comparisons** for QSL.

| Messages per batch | QSL leads | QSL behind | Inconclusive |
|---:|---:|---:|---:|
| 1 | 5 | 1 | 8 |
| 8 | 5 | 0 | 9 |
| 64 | 5 | 0 | 9 |
| 256 | 4 | 1 | 9 |

The dashboard shows latency per batch and per message. Each iteration processes all its independent frames. Buffer writers reuse one 32 KiB destination and consume the written prefix before proceeding to the next frame. The timer includes the batch loop and cursor update. Batch frames keep the same individual wire format. Requested bytes per batch are the mean per-message allocation requests multiplied by the batch size.

[Batch measurements](results/optimized-batch/confirmed.csv) · [Batch calibration](results/optimized-batch/tuning.json) · [Batch Criterion samples](results/optimized-batch/criterion.json.gz)


QSL leads when its 99% bootstrap median interval lies below all three competitors in both confirmation passes. A confirmed deficit means at least one competitor leads QSL by that rule. All other comparisons are inconclusive. The intervals describe repeatability within these runs. Treat differences on other CPUs or workloads as a separate measurement.

## What is timed

| Operation | Input at timer start | Result at timer end |
|---|---|---|
| Allocate and encode | Prepared codec-native value | Independently owned `Vec<u8>`. The timer includes allocation and destruction of the result. |
| Encode into a ready buffer | Prepared value and a reusable 32 KiB caller buffer | Bytes written into the buffer and consumed as a slice. Any internal allocations remain inside the timer. |
| Decode to an owned value | Pre-encoded immutable frame | Complete owned logical value. The timer includes materialization and destruction. |
| Zero-copy text view | Pre-encoded immutable frame | Record ID and a UTF-8 view whose text bytes alias the frame. Timed consumers observe the ID, length and three bytes. |

Each workload rotates through 256 prepared fixtures. The harness touches each input, round-trips the complete record and warms the output buffer before Criterion samples the operation. The allocation probe measures the same operation in a separate executable. It counts Rust global allocator allocation and reallocation requests, including bytes requested by reallocations. Wire and allocation columns are arithmetic means across 256 fixtures.

## Workloads and comparable contracts

- **Polymorphic API.** Five payload branches with nested optional records, optional lists and UTF-8 text. QSL uses a branch selector and a generated flat layout per payload. Explicit parent presence distinguishes an absent parent from a present parent with empty children. All four adapters reconstruct the same owned model.
- **Heartbeat.** Device status and periodic counters, including sparse optional state.
- **Socket event.** Polymorphic event records with optional context and variable payloads. The timer measures serialization only. The [historical series](HISTORY.md) includes separate HTTP and TCP tests.
- **Four bounded integers.** Four logical integers in `0..=4095`. QSL tests native 32-bit storage and compact 12-bit encoding. The same range permits 16-bit storage in Bincode and Cap’n Proto. Prost tests varint and fixed32.
- **Text view.** A record ID and either 128 or 2,048 UTF-8 bytes. QSL borrows with `lifetime`. Prost takes a UTF-8 view over a shared `Bytes` field. Bincode borrows `&str`. Cap’n Proto views an unpacked message. Every fixture checks the entire decoded string and verifies that its pointer range falls inside the encoded frame.

The normalization and construction of codec-native inputs precede encoding timings. For Cap’n Proto this includes building the message. QSL's bounded variants apply additional declared domain limits to the shared fixture values. Speed and wire size are distinct metrics. The tables choose by time, and writers and readers may select different wire formats. An application chooses a compatible pair for its link.

## Configuration and validation

All QSL schemas enable automatic native placement via `auto`. In a profile name, `bitmap` means `optionals=bitmap`, `bounded` means tighter declared domains, and `aligned` adds `alignment` to `compact`. The fixed case adds `aggressive[pointing=true]` and `aggressive[group=true]`. The text views use `lifetime`. QSL sets the Kaby Lake CPU profile for eligible groups. [Generated flat schema](results/optimized/generated.qsl) · [Fixed and text schemas](optimized/schemas/small.qsl)

The alternative configurations include Bincode fixed and variable integers, Protobuf fixed32 and varints for bounded integers, and Cap’n Proto packed and unpacked messages where each satisfies the operation contract.

A profile ending in `_trusted` calls a generated QSL unchecked method. Its typed source values were checked beforehand. Its reads operate on immutable frames established by a checked encode and decode. The harness verifies that the unchecked writer produces the same bytes. The trusted timings apply to that prevalidated boundary. Use checked entry points for network or otherwise untrusted bytes. Checked candidate timings remain in the calibration data.

In the zero-copy read, QSL's trusted reader reuses the prior UTF-8 validation. Prost converts its decoded byte field with `from_utf8`, Bincode decodes an `&str`, and Cap’n Proto calls `to_str` on the borrowed text. All four return views into the encoded frame. The time difference combines the costs of validation and decoding in each path. [Zero-copy adapters](optimized/src/zero_copy.rs)

For matched validation, define the same allowed domains, field and container limits, and malformed-input behavior for the common model. Include each codec's parsing and any additional application checks in the timed operation. Measure trusted reads as a separate series with the validation cost accounted for at the boundary where data becomes trusted.

Calibration uses records 0–255 and confirmation uses records 1024–1279. The runner freezes one winner per codec, workload and operation before confirmation, and reverses the candidate order in the second pass. The batch suite repeats selection independently for each batch size. Each confirmation pass uses 100 samples per single-record finalist and 60 per batch finalist.

## Environment and raw evidence

- Intel(R) Core(TM) i5-7400 CPU @ 3.00GHz, Windows 11 Pro 10.0.26200, rustc 1.98.0 (88d9e12ae 2026-08-18). QSL 0.4.0, Prost 0.14.4, Cap’n Proto 0.27.3 and Bincode 2.0.1 build with `-C target-cpu=native`, ThinLTO and a single codegen unit. Timing processes run on logical core 2 with above-normal priority.
- Average background CPU use in equivalent logical cores by phase was single record (tune 2.31, confirm-1 2.16, confirm-2 1.85) and batches (tune 1.39, confirm-1 1.37, confirm-2 1.36). These measurements include activity from the desktop and other running processes.
- [Version lockfile](optimized/Cargo.lock) · [Environment and timing intervals](results/optimized/summary.json) · [Selection and calibration](results/optimized/tuning.json) · [Allocation probe samples](results/optimized/profiles.json) · [Criterion estimates and samples](results/optimized/criterion.json.gz)
