# QSL architecture

[Complete guide](guide.en.md) · [Mathematics](mathematics.en.md) · [Measurements](features.en.md)

QSL separates build-time analysis from specialized execution: DEQ → parser → Rust field expansion → invariant checks → register/offset plan → generation → rustc/LLVM.

At runtime, the application selects ownership and regions, writes an agreed wire format and reads through checked or trusted APIs. [required][optional] ordering is mandatory; other modes are opt-in.

QPS transforms numeric operands and restores data exactly. opt_table derives events from length. The general byte envelope restores the inner frame before ordinary field offsets apply.

The complete guide contains method contracts, every module, type mappings, option limits, storage costs and errors. The API index contains 624 public declarations for this snapshot; feature gates apply.

Every schema generates checked methods. A native cursor validates the complete frame and aggregate budget before pointer decoding; QPS checks original length before restoration. [Contract](safety.en.md) · [Schema hashes](schema-hashes.en.md).
