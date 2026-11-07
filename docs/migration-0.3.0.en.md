# Current branch: qsl-core and string representations

These source changes follow release 0.2.0 on main. Published 0.2.0 packages are immutable. Use a repository checkout and local dependencies for the new architecture. Moving modules does not itself change BM v2 or ordinary string wire formats.

## Crate boundaries

| Crate | Responsibility | Dependencies |
|---|---|---|
| qsl-core | Errors, schema contracts, shared analyses, I/O, UTF-8 storage, validation, buffers and low-level codecs | no lexer/interpreter dependency |
| qsl-lexer | Tokens, QSL/DEQ parsing, loading, template expansion and schema builder | qsl-core |
| qsl-interpreter | Rust-specific preparation and Rust source generation | qsl-lexer |
| qsl | Application facade, macros and compatible entry points | core, lexer, interpreter, macro implementation |

Old module paths contain re-exports and associated tests. Core owns each implementation once, including one QslError identity and I/O contracts. Core contains neither a source parser nor a Rust generator. A future LLVM backend can consume shared contracts without executing the interpreter.

## Strings

| Choice | Rust representation | Ownership |
|---|---|---|
| ordinary string | String | owned heap storage |
| aggressive, finite range within STACK_LIMIT | StackString<N> | inline capacity |
| binary_opt=inline within STACK_LIMIT | InlineString<N> | inline capacity and independent length header |
| binary_opt=view | &str | borrowed UTF-8 from the input |

New initializes a fresh destination. Reuse updates an initialized destination; prepare sufficient String capacity first. These are method modes, not additional string types. binary_opt=guard and binary_opt=true are rejected. Utf8Guard, Utf8CopyGuard and byte-string representations are removed. Explicit Rust &str fields preserve owned String siblings. Fixed native byte fields retain [u8; N].

STACK_LIMIT defaults to 256 B. Ordinary oversized inline strings fall back to String. Raising the budget is explicit. An &str result requires its source frame to remain alive and immutable.

## Safety

Checked methods still enforce bounds, UTF-8, domains, actual decompressed lengths and aggregate budgets. Unchecked methods remain available with caller obligations for valid frames, lifetimes, disjoint storage and capacity. Tests cover forged lengths, invalid UTF-8, every truncation and mixed ownership.

## Current measurements

Nine Criterion cases separate String new/reuse, StackString new/reuse, InlineString new/reuse and &str. The main set has 1024 Cyrillic scalars / 2048 useful bytes; the control has 255 ASCII scalars. This benchmark explicitly sets STACK_LIMIT=2048 for inline containers; it is not the default budget. Validation and buffer preparation precede timing; new String includes allocation and Drop. Warm-up 15 s, measurement 10 s, 200 samples, 99% CI, process memory limit 8 GiB. See the [interactive report](en.html#measurements).
