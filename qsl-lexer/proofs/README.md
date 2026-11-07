# Bitmap arithmetic

`Bitmap.lean` checks marker inversion and the table-index bound established by
the one-time Safety check. The nonzero-mask trailing-zero property is an input
assumption matching Rust integer semantics. This is not a proof of arbitrary
custom Rust codecs, pointer provenance, aliasing, layout promises or LLVM output.
Run with the existing project Lean toolchain: `lake env lean ../qsl-lexer/proofs/Bitmap.lean` from the top-level proofs directory.
