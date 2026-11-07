# QSL 0.2.0

[Русский](README.ru.md) · [Complete guide](https://zadr0.github.io/QSL/guide.en.html) · [Quick start](docs/quickstart.en.md) · [GitHub Release](https://github.com/zadr0/QSL/releases/tag/v0.2.0)

QSL is a schema language and specialized Rust binary-codec generator for data described in advance. Build-time analysis resolves field order, domains, lengths, ownership and validation into concrete structures and methods. Schema parsing is outside the hot path.

It targets controlled internal protocols, telemetry and repeated records. Applications provide version negotiation, TLS, routing and external IPC storage. QSL supplies the format and caller-prepared buffer interfaces. An LLVM backend and QBProto handshake are planned, not implemented.

```toml
[dependencies]
qsl = "=0.2.0"

[build-dependencies]
qsl = "=0.2.0"
```

```qsl
&User[safety,aggressive,optionals=bitmap,opt_table] {
    u64 id;
    str(1..100) name;
    u8? status;
}
```

Required fields precede optionals; string bounds count UTF-8 bytes. Here name uses bounded inline storage and status has generated optional accessors; construct with try_new. Without aggressive, a variable COPY string normally uses String. safety enables checked deserialization; explicit unchecked methods remain available.

| Capability | Contract |
|---|---|
| default / opt_table | Match or a static optional-reader table; ordinary tag bytes are preserved |
| bitmap | Optional-only 8–128-bit masks and set-bit traversal |
| predict_seq | Length-derived index; inactive under BM |
| qps=Q2/Q4/Q8/Q16 | Reversible scalar packing; Q16 by default |
| Strings | String, bounded inline storage, &str / &[u8], independent QSL-LZ |
| Memory | serialize_to, deserialize_into, SerOut and bounded RT slots |
| Native layout | COPY and checked aligned LIFETIME; named regions create nested types |
| Custom types | QslCustom, validating scalar-wrapper macros and reusable Rust templates |
| Safety | Cursor bounds, ranges, UTF-8, decompression limits, MAX_DATA_SIZE=65535 |
| Compiler | qsl-lexer → one QslSchemaBuilder → qsl-interpreter |

**Migrating from 0.1:** BM changes wire format and requires coordinated peer migration. The native Cargo feature, Windows sockets/IPC regions and windows-sys are removed. Ordinary serialize returns Vec<u8>; serialize_to takes a mutable slice. Native layout and SerOut do not depend on Windows. [Complete migration and constraints](docs/migration-0.2.0.en.md).

STACK_LIMIT defaults to 256 data bytes per inline buffer. Larger owned buffers use String/Vec while retaining their codec; fixed native layouts require an explicitly sufficient limit. Attributes are opt-in and do not guarantee a speedup. Schema hashes describe format identity, not version negotiation or authentication.

[Interactive reference](https://zadr0.github.io/QSL/en.html) · [Architecture](docs/architecture.en.md) · [API](docs/API.en.md) · [Safety](docs/safety.en.md) · [Custom types](docs/custom-types.en.md) · [Examples](examples/README.md) · [Release notes](docs/releases/0.2.0.en.md).

Recorded hardware is Windows 11 x86-64, i5-7400 and 24 GiB / 2133 MHz. Historical measurements retain their source snapshots. The November 7 pass is rejected as a strict result because of CPU activity and establishes no new speedup. The 25 ns target for 1024 UTF-8 characters is not claimed achieved.

[MIT](LICENSE) · [crates.io](https://crates.io/crates/qsl/0.2.0) · [docs.rs](https://docs.rs/qsl/0.2.0/qsl/)
