# QSL 0.3.0 release verification

Source checks run on Windows 11 x86-64 with Rust 1.98. qsl, qsl-lexer and qsl-interpreter are 0.3.0; qsl-core is 0.1.0. The unchanged macro implementation remains 0.1.0. Tests, strict Clippy and warning-free Rustdoc cover all four libraries. Generated optional-mode contracts, four string representations and examples pass. Eight consumer tests pass with local path patches under both default and full features before publication.

registry-consumer/ intentionally contains exact registry dependencies with no path overrides or patches. Its post-publication tests and registry archive SHA-256 checksums are recorded separately. Local consumer checks are not represented as registry-only checks. Source-manifest hashes record checkout bytes; Git line-ending normalization can differ. No benchmarks are rerun for this metadata/release task; recorded timing data keeps its original source snapshot and OS-noise limitations.

## Published registry verification

All four versions were published successfully. Their local archive SHA-256 values match crates.io metadata, and docs.rs reports successful documentation builds for each. The independent registry consumer passed eight tests with default features and eight with full features, with every QSL package resolved from the registry. All four BM fixture frames are byte-identical to the 0.2.0 registry consumer. See registry-packages.json, registry-verification.json and registry-*.log. Package VCS metadata records the prepared source commit; the subsequent evidence commit adds records without changing library sources.
