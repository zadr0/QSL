# Strict comparison protocol / Протокол сравнения

## Current user-selected policy: ambient Windows / Текущий режим

The user waived the 5% pre-idle threshold on 2026-11-04. Use `-SkipIdleGate`. Three pre-run CPU observations remain; they do not block execution. Boundary noise measurements, `noiseAccepted` and raw traces remain unchanged, but in this policy noise is annotated instead of rejecting a valid measurement. Source hashes, successful exit, 15 s warmup, 10 s measurement, 200 samples, 99% CI, no sampler and the 8 GiB JobObject cap still apply. Old failed measurements are not retroactively accepted. Ambient and enforced-idle rows are never pooled. The first ambient controller pass runs the complete deterministic matrix; later retries select only missing cases.

Порог простоя отключён по указанию пользователя. В отчёте явно указан режим обычной фоновой нагрузки Windows. Показатели шума сохранены и не выдаются за простой. Предварительные контрольные два случая остаются отдельными наблюдениями; полный новый проход повторяет всю матрицу. Правила ниже описывают исходный режим с принудительным ожиданием простоя.

Frozen matrix: prepared encode and checked full traversal, Numeric at batch 1; Mixed at batches 1/4/16/64. QSL default + schema-optimized QSL, Cap’n Proto, FlatBuffers, Protobuf/prost and Bincode 2.0.1. Separate loopback request/ack tests compare framed TCP codecs with native gRPC/HTTP2 via tonic. No k6.

Every record has identical values, domains, optional states and UTF-8. Original useful bytes define M; P=M/T. Output wire size is separate. Verification touches every value and every text/data byte. Readers may borrow where the library supports it; output-ownership and allocation costs are reported explicitly. QSL trusted paths, if measured, are separate from checked paths.

Criterion: 15 s warmup, 10 s measurement, 200 samples, 99% CI. Fixtures and bounded output storage are prepared and touched before timing. Persistent connections exclude connect/handshake. Instrumentation and noise sampling never execute per record in the measured loop. An idle gate precedes a run; Windows cannot guarantee a permanently idle background. Collection overhead uses a separate paired control. OS counters and allocation probes have separate runs; unavailable I-cache counters remain unavailable.

Limits: 65535 B per record; 256 KiB per transport batch; 64 records per batch; UTF-8 <=4096 B, data <=256 B, each numeric value <=4095, optional a<=255, b<=65535. Cap’n Proto traversal/depth and FlatBuffers verifier limits are configured; Prost input is bounded before decode; Bincode decode has a size limit and checks consumed bytes; gRPC send/receive size caps apply. QSL validates the frame and total values before trusted pointer decoding. Malformed input controls are outside latency timing.

No engine optimizations are applied by this comparison. Existing schema options and reusable APIs are selected explicitly. Codec and transport results are separate; generic framed TCP is an application protocol, not Cap’n Proto RPC or a native FlatBuffers network standard. gRPC uses typed protobuf messages, not a wrapper around pre-encoded QSL.

Optimized numeric QSL uses the existing prepared APIs: serialize_quantized_to and QuantizedFrame::try_new + decode_into. Each read still validates the complete exact frame; 8 initialized private padding bytes enable native loads and are excluded from wire and M. Mixed QSL uses lifetime-based borrowing and the preserved combined optional graph. No unsafe unchecked-input benchmark is ranked against checked decoders.

Two real gRPC modes are retained separately: unary Submit and persistent bidirectional Flow. Both use typed generated protobuf records and bounded HTTP2 messages; streaming amortizes per-RPC header work without removing validation. TCP has one outstanding request/ack per connection; these are latency/throughput batch profiles, not a concurrency saturation claim.

Native schemas use the same known domain to avoid handicapping competitors: Cap’n Proto UInt16 fields and an inline FlatBuffers Values struct; packed repeated uint32 in proto3; u16 fixed arrays in Bincode; QSL uses declared u64(0..4095) ranks. All restore the original logical u64 values exactly. M stays the original fixture’s useful bytes. Protobuf’s timed codec/TCP path clears and merges into warmed native-owned buffers; fresh decode is tested but not the ranked prepared path. Ordinary proto3 defaults are retained; the packed list must contain exactly eight values.

The optimized mixed QSL schema narrows each known 0..4095 numeric field to u16, while the default QSL baseline retains u64. The original u64 fixture is range-checked before narrowing; no truncation is accepted. Bincode uses explicit little-endian fixed-int encoding over its narrowed u16 array, plus the 65535-B decode limit, avoiding variable-integer branches for these fixed bounded fields. This is an explicitly optimized profile, not a default-settings ranking.

Final timing builds use `-C target-cpu=native`, thin LTO and one codegen unit for every Rust dependency/codec. The executable and source hashes are retained. Preliminary plumbing runs built during preparation are not part of the final comparison.

The common consumer touches every scalar and hashes every text/data byte using the same native XXH3 kernel; it is application consumption, not a metrics counter. Four consumer-only controls expose this common work separately. Neither loop nor consumer controls are subtracted from timings. The gRPC acknowledgement uses fixed64 for its full-width checksum.

CPU metrics revision 2 separates known other-process CPU, unattributed system residual, DPC/ISR and system-wide context switches. Measurements are boundary-only. Network-induced kernel work is not automatically called foreign noise. Acceptance keeps the 5%-per-core pre-idle gate and 2% limits for other-process CPU and conservative residual after DPC/ISR; unavailable kernel counters keep the conservative raw residual. DPC attribution to individual drivers requires a separate trace and is not asserted by these counters. Unknown protected/transient processes are disclosed; this is bounded noise validation, not a guarantee of zero interrupts.

All codecs are linked into one timing executable. Whole-case peak RAM therefore includes the shared harness and prepared fixture/encoded rings; it is not the isolated deployment footprint of an SDK. CPU core utilization is not cycles per record. The idle trace is collected before starting the measured process, not during its hot loop.
