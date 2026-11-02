# QSL — schema-specialized binary codec generation

[Русский](README.ru.md) · [Interactive documentation](docs/index.html) · [Architecture](docs/architecture.en.md) · [API](docs/API.en.md)

QSL specializes Rust serialization for declared types, ranges, field order and ownership. Required fields precede optional fields; additional modes are opt-in.

**QPS compresses numeric operands losslessly. opt_table predicts from length. QPS does not require optional fields.**

```qsl
&Numbers[qps=Q16,safety] { u64(0..15) a; u64(0..15) b; }
&Packet[opt_table,safety] { i32 id; u8? a; u8? b; }
```
`[qps]` defaults to Q16; Q2/Q4/Q8/Q16 are supported. `quant=Qn` is a compatible alias. Select the old D/index format explicitly as `state_bits`; both peers must agree when changing modes. Q changes the exact quotient/remainder representation; savings come from ranges, not lost precision.

Implemented: checked APIs with [safety], owned/borrowed/byte/CopyGuard/inline strings, nested Rust structure macros, critical-length opt_table, optional static RT pools, and Windows regions with IPC. QBProto handshake/R remains planned. Wire-frame size is not Rust object size.

## Fresh measurements: QSL only

| Operation | Without QPS, ns | QPS Q16, ns |
|---|---:|---:|
| deserialize_into | 15.57 | 9.56 |
| serialize_into | 7.41 | 13.70 |

Frame 17–49 B → 3–13 B; prepared QPS APIs additionally require 16 B of private padding. This layout reads faster and writes slower. Warm-up 15 s, measurement 10 s, 200 samples, 99% CI; 8 GiB limit. Windows 11 x86-64, i5-7400, RAM 24 GiB at 2133 MHz. Four cases in one profile, with no other codecs or network tests. [Conditions, intervals, data and commands](docs/performance.en.md).

Source and test status: [audit](SOURCE_AUDIT.md). This repository publishes documentation and the harness, not a Cargo workspace. The matching local dev-qsl checkout was measured and source hashes are published. docs/index.html runs without a CDN; GitHub Pages can serve /docs.

Separate control: [default vs opt_table for 1/2/3/5/10 optional fields](docs/opt-table-control.en.md). Same values and inline policy; reading and writing are shown independently.
