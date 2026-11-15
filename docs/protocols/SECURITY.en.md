# Validation and resource controls

All timed readers validate and consume the complete logical record. Prepared buffers and warmup do not remove safety checks. The same application domains apply: eight numbers in 0..4095, UTF-8 of 1..4096 B, data of 0..256 B and two optional fields. Original u64 values are checked before narrowing. Codec frames are capped at 65535 B; transport batches are capped at 64 records and 256 KiB.

| Implementation | Input checks | Developer controls |
|---|---|---|
| QSL | Complete ValidationCursor, domains, UTF-8, tags/length and total budget before trusted decoding | MAX_DATA_SIZE, schema domains, ownership, output capacity; prepared QPS requires initialized private padding |
| Cap’n Proto | No-allocation reader, checked accessors, full traversal and depth/traversal limits | ReaderOptions: 8192 words, depth 16; application domains and lengths |
| FlatBuffers | Checked root/verifier before accessors, then application domains | max_depth=16, max_tables=16, max_apparent_size=65535; required Values, text and data |
| Protobuf/prost | Frame cap before decode/merge, SDK UTF-8 checks, exact eight values and application domains after decode | Message cap, repeated-field shape, private warmed scratch; publish only on Ok |
| Bincode 2 | Limited borrow_decode, fixed-int little-endian, exact consumed byte count | Matching types/configuration, 65535 limit, application checks after decode |
| gRPC/tonic | 256 KiB send/receive caps, typed Protobuf, validation of each record | Caps, concurrency/window/backpressure, unary or bidirectional stream |

Protobuf/gRPC repeated-field application limits are checked after native containers are constructed. Wire-size bounds are not heap-size bounds: small repeated messages can expand into larger objects. The 8 GiB JobObject cap covers the measured process and descendants. It does not prove every SDK returns Result on system OOM.

Tests cover byte mutations, truncation, oversized frames, invalid domains, non-truncating narrowing, prepared/exact QSL equivalence, owned/reused Protobuf equivalence and real TCP/unary/stream exchanges. A finite passing corpus is not a formal proof of every SDK or unsafe operation. The consumer checksum checks work; it is not authentication. No QBProto schema negotiation is implemented. The harness handshake identifies its codec and version; schemas are frozen in source.

All connections are persistent loopback without TLS or transport compression. Results do not estimate a secured remote production deployment. Framed TCP is not Cap’n Proto RPC. TCP framing/read errors close the connection. Applications must account for Prost scratch possibly being partially modified on failed merge and keep it private until successful validation.

SDK references: [Cap’n Proto ReaderOptions](https://docs.rs/capnp/0.27.3/capnp/message/struct.ReaderOptions.html), [FlatBuffers Rust](https://flatbuffers.dev/languages/rust/), [Prost Message](https://docs.rs/prost/0.14.4/prost/trait.Message.html), [Tonic caps](https://docs.rs/tonic/0.14.9/tonic/server/struct.Grpc.html), [Bincode configuration](https://docs.rs/bincode/2.0.1/bincode/config/struct.Configuration.html).
