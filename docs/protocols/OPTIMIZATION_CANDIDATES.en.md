# QSL optimization candidates — implementation unchanged

All 42 candidates found in this inspection are listed below. This does not claim that every possible optimization has been exhausted. The completed optional-mode correction is a separate behavior fix. None of these engine optimization proposals is implemented by the comparison harness.

P1: first validate with a narrow experiment. P2: schema/architecture dependent. P3: build-time or integration work. A branch, loop or division alone does not prove a bottleneck; inspect generated machine code and measure.

| # | Priority | Candidate | Source |
|---|---|---|---|
| 01 | P1 | Separate validation and result construction | [codegen_validation.rs:184](../../qsl/src/codegen_validation.rs#L184). |
| 02 | P1 | Repeated checked-write passes | [codegen_validation.rs:211](../../qsl/src/codegen_validation.rs#L211). |
| 03 | P1 | Aggregate budgets for fixed fields | [codegen_validation.rs:217](../../qsl/src/codegen_validation.rs#L217). |
| 04 | P1 | Small native headers | [validation.rs:104](../../qsl/src/validation.rs#L104). |
| 05 | P1 | Fixed required-prefix validation | [validation.rs:96](../../qsl/src/validation.rs#L96). |
| 06 | P1 | Indirect opt_table reader cost | [codegen_tagged_optional.rs:26](../../qsl/src/codegen_tagged_optional.rs#L26). |
| 07 | P1 | Optional result initialization | [codegen_tagged_optional.rs:6](../../qsl/src/codegen_tagged_optional.rs#L6). |
| 08 | P1 | Tags from the shared header | [codegen_tagged_optional.rs:30](../../qsl/src/codegen_tagged_optional.rs#L30). |
| 09 | P2 | Sparse entries for required tags | [codegen_qps.rs:157](../../qsl/src/codegen_qps.rs#L157). |
| 10 | P1 | Repeated length indexing after validation | [codegen_length_graph.rs:213](../../qsl/src/codegen_length_graph.rs#L213). |
| 11 | P2 | Unused event functions | [codegen_length_graph.rs:9](../../qsl/src/codegen_length_graph.rs#L9). |
| 12 | P2 | Linear critical-point indexing | [codegen_length_graph.rs:52](../../qsl/src/codegen_length_graph.rs#L52). |
| 13 | P2 | Static index width | [codegen_static_optional.rs:32](../../qsl/src/codegen_static_optional.rs#L32). |
| 14 | P2 | Architecture-dependent table budget | [static_index.rs:34](../../qsl/src/static_index.rs#L34). |
| 15 | P1 | Reused strings in deserialize_into | [interpreter.rs:2050](../../qsl/src/interpreter.rs#L2050). |
| 16 | P1 | Utf8CopyGuard capacity preparation | [strings.rs:1337](../../qsl/src/strings.rs#L1337). |
| 17 | P2 | UTF-8 validation | [strings.rs:929](../../qsl/src/strings.rs#L929). |
| 18 | P2 | Copies into inline storage | [strings.rs:211](../../qsl/src/strings.rs#L211). |
| 19 | P1 | Repeated LZ restoration | [validation.rs:128](../../qsl/src/validation.rs#L128). |
| 20 | P1 | LZ search budget | [strings.rs:439](../../qsl/src/strings.rs#L439). |
| 21 | P2 | Overlapping LZ copies | [strings.rs:707](../../qsl/src/strings.rs#L707). |
| 22 | P2 | Rounding and division | [strings.rs:594](../../qsl/src/strings.rs#L594). |
| 23 | P1 | Native QPS register selection | [codegen_payload_quant.rs:121](../../qsl/src/codegen_payload_quant.rs#L121). |
| 24 | P2 | selected_address and scalar_operand | [codegen_payload_quant.rs:84](../../qsl/src/codegen_payload_quant.rs#L84). |
| 25 | P2 | Zero/full QPS domains | [codegen_payload_quant.rs:14](../../qsl/src/codegen_payload_quant.rs#L14). |
| 26 | P2 | Repeated prepared-frame validation | [payload_quant.rs:202](../../qsl/src/payload_quant.rs#L202). |
| 27 | P1 | BMI2 kernel selection | [qps_bytes.rs:183](../../qsl/src/qps_bytes.rs#L183). |
| 28 | P2 | QPB3 min/max and write passes | [qps_bytes.rs:331](../../qsl/src/qps_bytes.rs#L331). |
| 29 | P2 | Checks inside every QPB3 group | [qps_bytes.rs:407](../../qsl/src/qps_bytes.rs#L407). |
| 30 | P2 | Cursor register width | [bit_io.rs:279](../../qsl/src/bit_io.rs#L279). |
| 31 | P2 | Encoded-size computation | [interpreter.rs:453](../../qsl/src/interpreter.rs#L453). |
| 32 | P2 | Generated code footprint | [interpreter.rs:2401](../../qsl/src/interpreter.rs#L2401). |
| 33 | P2 | CAS and shared occupancy cache line | [rt.rs:32](../../qsl/src/rt.rs#L32). |
| 34 | P2 | Slot false sharing | [rt.rs:11](../../qsl/src/rt.rs#L11). |
| 35 | P2 | IPC atomic allocation | [native/memory.rs:433](../../qsl/src/native/memory.rs#L433). |
| 36 | P2 | IPC publication | [native/memory.rs:566](../../qsl/src/native/memory.rs#L566). |
| 37 | P2 | System-call count | [native/socket.rs:325](../../qsl/src/native/socket.rs#L325). |
| 38 | P2 | Separate header/payload reads | [native/socket.rs:278](../../qsl/src/native/socket.rs#L278). |
| 39 | P3 | Memory reservation and commit | [native/memory.rs:345](../../qsl/src/native/memory.rs#L345). |
| 40 | P3 | Canonical descriptors and analysis-time hashing | [schema_hash.rs:75](../../qsl/src/schema_hash.rs#L75). |
| 41 | P3 | Repeated Rust-template expansion | [rust_macro.rs:200](../../qsl/src/rust_macro.rs#L200). |
| 42 | P3 | Repeated LSP analysis | [bin/qsl-analyze.rs:24](../../qsl/src/bin/qsl-analyze.rs#L24). |

Detailed hypotheses and per-item controls are in the [Russian inventory](OPTIMIZATION_CANDIDATES.ru.md), with matching numbers and source anchors.

Use one narrow experiment per hypothesis with the same data and wire contract. Compare default / tag-only opt_table / predict_seq / combined; checked / trusted; fresh / reused output. Collect ASM/LLVM and hardware events in separate diagnostic runs. Preserve size, length, UTF-8, range, tag and restored-size validation before unsafe decoding. Require repeatable improvement and retained source/raw evidence. Preliminary tag-dispatch timings did not include the new idle/noise controls and are not strict cross-protocol evidence.

## After core extraction: StackString reuse

The focused string harness exposes a separate reuse cost for StackString. This layout is not selected by prepared_read_layout; default Serializable::deserialize_reuse assigns a newly returned value. Candidate: write directly into initialized StackString storage without an intermediate return. This optimization has not been applied. Confirm the cause with assembly and an idle repeat; the first pass has background load.
