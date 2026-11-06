# QSL

[Русский](README.ru.md)

## English

QSL is a data-description language and a specialized Rust code generator for binary exchange. Its original task is exchanging data described in advance. Both peers know field order, domains and codecs before application startup.

At build time QSL analyzes bounds, bit widths, register groups, tables and wire identities, then emits concrete methods. Runtime values and lengths remain dynamic; schema interpretation is outside the hot path.

### Where it fits

QSL targets controlled exchange between coordinated components: internal protocols, IPC, telemetry and repeated records. Its specialization is a build-time-known format with explicit memory invariants. Developers choose ownership, bounds, input checking and prepared storage. Routing, TLS, version negotiation and a QBProto handshake are not automatically supplied by a schema.

~~~qsl
&User[safety,aggressive] {
    u64 id;
    str(1..100) name;
    u8? status;
}
~~~

Required fields always precede optionals. Here name is an inline COPY string bounded by 100 **UTF-8 bytes**. Read status through status(), change it through set_status(...), and construct the record through try_new(...). Without aggressive, variable owned strings normally use String. Without safety, checked deserialization is not generated.

### Available choices

| Goal | Mechanism |
|---|---|
| Reduce repeated read work | Automatic exact register groups |
| Select ownership | String, inline storage, direct &str / &[u8], fixed arrays |
| Pack scalar numbers losslessly | qps=Q2/Q4/Q8/Q16; Q16 by default |
| Compress a string | Independent QSL-LZ; QPS never quantizes strings, arrays or lengths |
| Read optional fields | predict_seq: length index; opt_table: tag → reader |
| Control untrusted input | safety, one cursor, MAX_DATA_SIZE = 65535 |
| Prepare storage | serialize_to / deserialize_into, RT slots, loyal_cache |
| Use native layout | Separate COPY/LIFETIME Transmutate and nested regions |
| Reuse definitions | Nested Rust qsl_macro / schem_macro field expansion |

Attributes are opt-in. STACK_LIMIT defaults to **256 data bytes per inline buffer**, including explicit inline/LZ requests. Larger COPY buffers use String/Vec; fixed native Transmutate requires an explicitly sufficient limit. Bounded RT pools wait for a free slot. Trusted pointer methods require valid frames and correct memory. Low-level serialize trait methods are now unsafe; serialize_checked is the general safe output API.

Wire changes require coordinated peers. Strict and light hashes are computed during analysis; xxHash is not cryptographic. Performance depends on the schema, architecture, working-set size and checked/trusted contract.

[Contract and migration EN](docs/modernization-2026-11-06/MODERNIZATION.en.md) · [Complete guide](docs/guide.en.html) · [Runnable examples](examples/README.md) · [Focused measurements and charts](qsl-bench/modernization/report.html)

Fresh run, 1024 UTF-8 scalars / 2048 B: String 130.38 ns, inline string 63.20 ns; the 25 ns target was not achieved. Numeric record: default 2.68 ns / 24 B, Q16 4.15 ns / 5 B. These are trusted reads without validation. The report retains 99% intervals, outliers and source snapshots. The protocol comparison remains a historical **16/100** result, not a completed full pass.
