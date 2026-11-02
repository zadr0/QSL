# QSL — specialized binary exchange

[Русский](README.ru.md) · [Complete guide](docs/guide.en.md) · [Interactive documentation](docs/index.html) · [API](docs/API.en.md)

QSL generates Rust for exchanging data described in advance. Types, domains, field order and ownership define the representation before execution. Additional mechanisms are explicit; [required][optional] ordering is mandatory.

**QPS is exact operand compression. opt_table performs length prediction.**

```qsl
&Message[safety,opt_table] {
    u32 id;
    str(0..4096) text|binary_opt=guard|;
    u16? code;
}
```

The guide covers the complete system: QSL/DEQ, default/compact representations, optional/bitmap/predict_seq/opt_table, String and UTF-8 wrappers, string LZ, both QPS paths, validation, inline, Rust templates, custom types, SerOut, RT, native memory, IPC and sockets. Limits accompany each API choice.

## General QPS

ByteQps<Q2/Q4/Q8/Q16> accepts arbitrary serialized bytes, including UTF-8 and arrays, and restores them losslessly. Its 16 B QPB3 header stores original length without quantization. Packed is selected only when the complete frame saves space; otherwise explicit raw is used. A separate static scalar specialization remains available.

The general path is explicit through ByteQps or serialize_qps_to / deserialize_qps_into. Existing Serializable formats do not change implicitly. Numeric QUANT_ACTIVE describes the specialization, not absence of the general codec. Four u8 values provide a u32 operand but do not guarantee compression of every 32-bit pattern.

## Measurements and limits

The [full report](docs/features.en.md) and [interactive charts](docs/index.html#measurements) show useful P=M/T, time, wire sizes, 99% CI and waveforms of actual Criterion samples. M is original useful bytes. New ownership, reuse and borrowing are labelled separately.

The main pass covers 95 cases; focused UTF-8/whole-record checks add 8. Only the 22 general-QPS cases were repeated after local optimization: 103 distinct cases, 125 observations in total. Earlier N=1/2/3/5/10 snapshots remain separate. Smaller frames do not guarantee faster operations; the general codec is not automatically promoted into a hot path.

Related material: [mathematics and CPU/memory](docs/mathematics.en.md), [architecture](docs/architecture.en.md), [checks](SOURCE_AUDIT.md), [RU/EN Zed extension](https://github.com/zadr0/QSL-Extension).

This repository distributes documentation and reproducible harnesses, not the complete Cargo workspace. Measurements describe captured local dev-qsl sources; hashes, raw data and commands are provided. QBProto handshake/R remains planned.
