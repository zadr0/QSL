# QSL 0.1.2 release verification

Published on 2026-11-07: qsl 0.1.2, qsl-lexer 0.1.0 and qsl-interpreter 0.1.0.
All three registry archives originate from clean source commit b8819127b4a0.
The registry archive checksums, source commit and English-documentation checks
are recorded in registry-verification.json.

The lexer, Rust backend and runtime package archives were verified and tested
with all features before upload. The registry-only application then passed
checked round-trip, DEQ-bound and truncated-input checks in default and full
feature configurations. All QSL packages in its lockfile resolve from crates.io.

```powershell
cargo run --manifest-path reference/crates-release-0.1.2/consumer/Cargo.toml --locked
cargo run --manifest-path reference/crates-release-0.1.2/consumer/Cargo.toml --locked --features full
```

The expected output reports 17 wire bytes and a passed checked round-trip.
The existing qsl-macros-impl 0.1.0 package remains the macro dependency.
LLVM generation is planned; this release supplies the shared builder/backend contract.
