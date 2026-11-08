# QSL 0.3.0 release verification

Source checks run on Windows 11 x86-64 with Rust 1.98. qsl, qsl-lexer and qsl-interpreter are 0.3.0; qsl-core is 0.1.0. The unchanged macro implementation remains 0.1.0. Tests, strict Clippy and warning-free Rustdoc cover all four libraries. Generated optional-mode contracts, four string representations and examples pass. Eight consumer tests pass with local path patches under both default and full features before publication.

registry-consumer/ intentionally contains exact registry dependencies with no path overrides or patches. Its post-publication tests and registry archive SHA-256 checksums are recorded separately. Local consumer checks are not represented as registry-only checks. Source-manifest hashes record checkout bytes; Git line-ending normalization can differ. No benchmarks are rerun for this metadata/release task; recorded timing data keeps its original source snapshot and OS-noise limitations.
