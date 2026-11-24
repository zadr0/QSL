# QSL 0.4.0 codec benchmark

QSL, Protobuf, Cap’n Proto and Bincode process the same logical values with optimized Rust builds. Each codec contributes one configuration per workload and operation. The fastest calibration median selects that configuration. Two later runs measure the winners on different records.

[Dashboard](dashboard.html) · [Timing CSV](results/optimized/confirmed.csv) · [Historical benchmarks](HISTORY.md)

## Results

<!-- RESULTS -->

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

Calibration uses records 0–255 and confirmation uses records 1024–1279. The runner freezes one winner per codec, workload and operation before confirmation, and reverses the candidate order in the second pass. The batch suite repeats selection independently for each batch size. Each confirmation pass uses 100 samples per single-record finalist and 60 per batch finalist.

## Environment and raw evidence

- <!-- MACHINE -->. QSL 0.4.0, Prost 0.14.4, Cap’n Proto 0.27.3 and Bincode 2.0.1 build with `-C target-cpu=native`, ThinLTO and a single codegen unit. Timing processes run on logical core 2 with above-normal priority.
- Average background CPU use in equivalent logical cores by phase was <!-- LOAD -->. These measurements include activity from the desktop and other running processes.
- [Version lockfile](optimized/Cargo.lock) · [Environment and timing intervals](results/optimized/summary.json) · [Selection and calibration](results/optimized/tuning.json) · [Allocation probe samples](results/optimized/profiles.json) · [Criterion estimates and samples](results/optimized/criterion.json.gz)
