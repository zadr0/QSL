# QSL 0.2.0 source and verification scope

[RU migration](docs/migration-0.2.0.ru.md) · [EN migration](docs/migration-0.2.0.en.md) · [Source SHA-256](reference/crates-release-0.2.0/source-manifest.json)

The frontend, Rust backend and runtime are separate packages. BM revision 2 has optional-only masks, descending set-bit traversal and one spare-bit bound check. PS and OT keep their distinct contracts. Native regions become true nested Rust types. Custom scalar wrappers explicitly validate representation/domain promises and retain QslCustom hooks.

The native feature, windows-sys, Windows-only socket/IPC modules and direct platform wait FFI are removed. Portable SerOut and std TCP remain; RT uses standard Rust synchronization with atomic slot acquisition. Native-layout COPY/LIFETIME and unchecked methods remain independent of OS libraries.

## Release checks

The release evidence directory records frontend/backend/runtime tests, generated-code contracts, strict Clippy, Rustdoc, packaged artifacts and registry-consumer verification. The runtime additionally tests four blocked RT waiters waking after a held slot is released. The analyzer is checked at 11 mask-width boundaries, descending field order, u128 native presence and nested native-object sizes. The HTML documentation is checked for RU/EN controls, API search, finite graphs, byte/operand vectors and the BM model.

Lean4 verifies the explicit bitmap/index arithmetic assumptions. It does not prove pointer provenance, aliasing, user unsafe implementations, generated assembly, allocator behavior or scheduler semantics. Passing tests is not an exhaustive proof of memory safety. Rust 1.98 / LLVM 22.1.8, Windows 11 x86-64 / i5-7400 / 24 GiB 2133 MHz are the recorded build environment; other platforms are not claimed runtime-tested here.

## Timing evidence

Historical feature/example/protocol datasets retain their source snapshots and are not 0.2.0 results. The four [November 7 Criterion cases](qsl-interpreter/tests/optional-v2/results/2026-11-07.md) are not accepted as strict performance evidence: CPU activity was 33% before the run and 27–64% in later idle observations. No speedup is asserted, and the 25 ns UTF-8 target is not claimed achieved. No broad benchmark matrix or k6 run accompanies this release.

The previous audit is retained in docs/archive/before-0.2.0. Publication and registry-only checks are recorded after they complete; this file does not substitute for their logs.
