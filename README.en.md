# QSL — Quick Schema Language

QSL describes agreed data layouts and generates specialized Rust binary codecs at build time. The schema defines field order, value ranges, string and array bounds, ownership and validation. The application calls compiled methods; it does not parse schemas at run time.

QSL targets controlled internal protocols, telemetry and repeated records with known limits. It requires explicit data contracts and compatible schemas at both ends. Applications provide transport, TLS and version negotiation.

## Getting started

```toml
[dependencies]
qsl = "=0.4.0"

[build-dependencies]
qsl = "=0.4.0"
```

```qsl
&User[safety] {
    u64 id;
    str(1..100) name;
}
```

During the build, `qsl::build_qsl_files` writes Rust code to `OUT_DIR`. Include the generated file and use `serialize_checked` and `deserialize_checked`. The [runnable quick start](examples/quickstart) includes `Cargo.toml`, `build.rs` and `main.rs`.

## Memory and safety

`qsl-core` is always `no_std`. Its optional `alloc` feature provides `String`, `Vec` and `Box`; the final application supplies the allocator. Runtime I/O and validation belong to core; `qsl-lexer` analyzes schemas and `qsl-interpreter` emits Rust code.

When the proven maximum frame size is less than or equal to `STACK_LIMIT`, ordinary serialization returns `StackOut<N>`. A larger or unknown maximum selects `Vec<u8>`. This choice is made during code generation. `serialize_checked_to` writes into the application's buffer. Prepared unsafe reuse retains suitable capacities without growth.

Required fields precede optional fields. String lengths count UTF-8 bytes; array lengths count elements. Checked readers validate bounds, value domains, UTF-8 and the `MAX_DATA_SIZE` budget. Compressed-string decoding validates each token against the actual output capacity. Separate unsafe methods retain explicit caller obligations.

## Optional mechanisms

- `compact` packs values according to declared numeric domains.
- `predict_seq` derives an index from payload length; `opt_table` dispatches readers by optional-field tags; bitmap mode stores optional presence bits.
- QPS provides reversible numeric encoding with Q2/Q4/Q8/Q16; the default is Q16. It does not predict optional fields. QSL-LZ compresses strings through a separate codec with raw fallback.
- Rust templates expand into schema fields; custom primitives define their representation and validators.
- `auto` optimizes native layout while preserving wire order. Wire reordering requires explicit `auto[wire=1]` permission.
- `qsl-rt` provides bounded local and shared output pools. Exhaustion returns `PoolExhausted`.
- Strict and lightweight schema hashes are computed during analysis. They do not replace format negotiation or authentication.

Attributes are opt-in. The required-before-optional field order is mandatory. A declaration, its wire-affecting configuration and the selected codecs together define the binary format.

## Documentation and resources

[English documentation source](docs-site/src/content/docs/en/start/introduction.md) · [Русская версия](README.ru.md) · [Builder examples](examples/08-builders) · [BMI2 examples](examples/09-bmi2) · [Lean proofs](proofs)

[QSL on GitHub](https://github.com/zadr0/QSL) · [crates.io](https://crates.io/crates/qsl) · [Rust API](https://docs.rs/qsl) · [MIT license](LICENSE)
