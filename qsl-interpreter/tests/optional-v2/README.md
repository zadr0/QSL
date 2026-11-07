# Optional dispatch and portable runtime contracts

Run from the dev-qsl root. The root qsl directory is not needed: the harness patches qsl to the existing `proofs/qsl` and both compiler crates to their local directories. No Windows/native runtime feature is enabled or defined.

```powershell
cargo test --manifest-path qsl-lexer/Cargo.toml --all-features --locked
cargo test --manifest-path qsl-interpreter/Cargo.toml --all-features --locked
cargo test --manifest-path proofs/qsl/Cargo.toml --all-features --locked
cargo test --manifest-path qsl-interpreter/tests/optional-v2/Cargo.toml --all-features --locked
cargo clippy --manifest-path qsl-interpreter/tests/optional-v2/Cargo.toml --all-features --all-targets --locked -- -D warnings
```

The harness tests BM boundaries through 128 optional fields, identical match/OT wire, ordinary-tag compatibility, compact and borrowed values, compressed strings, arrays, custom primitive ranges and restored-data budgets, independent nested masks, real nested native regions, COPY and checked aligned borrowing. A deterministic malformed-input sweep checks errors without publishing partial destinations. This is focused regression coverage, not an exhaustive proof of unsafe Rust.

For Lean4, run `lake env lean ../qsl-lexer/proofs/Bitmap.lean` from the existing top-level proofs directory. Its assumptions and limits are documented beside the proof.

## Four focused Criterion cases

```powershell
$env:RUSTFLAGS = '-C target-cpu=native'
cargo bench --manifest-path qsl-interpreter/tests/optional-v2/Cargo.toml --bench dispatch --locked --target-dir .optimization/optional-v2-bench-target
```

`target-cpu=native` selects this CPU's instruction set; it does not enable a Cargo feature or any OS library. Criterion itself has Windows-only development dependencies when run on Windows. They are absent from the normal/build dependency graph of the QSL libraries.

Only BM match/OT and ordinary tagged match/OT are measured. Each case has one required u64 plus ten present optional u64 fields: M=88 original useful bytes, with 64 prepared frames. BM wire size is 90 B and tagged size is 94 B. P=M/T uses original useful bytes/ns, excluding metadata. The measured operation is trusted decoding, input selection and black_box consumption of the complete result; input construction, validation and allocation are outside the timed loop. All decoded fields here are scalars, so the decode loop does not allocate.

Warm-up is 5 seconds per case; measurement is 8 seconds, 100 samples and 99% confidence intervals. There is no network, k6, OS telemetry or live profiler inside the loop. These four cases do not establish results for absent-field distributions, different field counts, checked decode, strings, networking, cold caches or all QSL options. Record background CPU activity separately; do not describe an uncontrolled Windows run as noise-free.
