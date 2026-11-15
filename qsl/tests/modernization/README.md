# Historical flat-region contracts

This standalone harness targets the older 0.1.x flat native-region API. It is not
part of `cargo test --manifest-path proofs/qsl/Cargo.toml`. Its `region_0_*`
methods and flattened native struct literals do not match the published 0.3.0
nested-region API. Updating its dependency lock alone does not migrate these tests.

The 2026-11-12 run exposed this incompatibility (duplicate native definitions and
missing flat-region methods). The harness is retained; it is **not** reported as
passing. Current contracts are exercised by the root runtime tests and
`qsl-bench/unified`, including checked/unchecked native copy, overlay alignment,
malformed UTF-8, optional ownership, and published-0.3.0 wire fixtures.
