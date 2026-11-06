> **2026-11-06 contract:** [strict STACK_LIMIT, opt-in Safety, scalar-only QPS and native COPY/LIFETIME](modernization-2026-11-06/MODERNIZATION.en.md). [Four fresh measurements](../qsl-bench/modernization/report.html). Historical timing tables retain their original snapshots.

# QSL architecture

[Complete guide](guide.en.md) · [Mathematics](mathematics.en.md) · [Measurements](features.en.md)

QSL separates build-time analysis from specialized execution: DEQ → parser → Rust field expansion → invariant checks → register/offset plan → generation → rustc/LLVM.

At runtime, the application selects ownership and regions, writes an agreed wire format and reads through checked or trusted APIs. [required][optional] ordering is mandatory; other modes are opt-in.

QPS transforms numeric operands and restores data exactly. predict_seq determines state from length; opt_table selects a reader by tag. The general byte envelope restores the inner frame before ordinary field offsets apply.

The complete guide contains method contracts, every module, type mappings, option limits, storage costs and errors. The API index contains 625 public declarations for this snapshot; feature gates apply.

Safety enables checked deserialization: the cursor materializes and validates values in one pass before publishing the object. [Contract](safety.en.md) · [Schema hashes](schema-hashes.en.md).

[Optional modes and compatibility](optional-modes.en.md).
