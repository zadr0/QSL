# Changelog

## 0.2.0 — 2026-11-15

- Runtime and I/O stay in an always-no_std crate with optional alloc.
- Add schema-selected StackOut output, caller-buffer contracts, direct heap InlineString construction and checked LZ restoration.
- Preserve explicit trusted APIs, prepared reuse, optional length niches and deterministic wire identity.

### Development notes — 2026-11-14

- Keep runtime I/O, codec/primitive contracts, validation and owned containers in always-no_std qsl-core; make alloc explicit and compiler support optional in the facade.
- Add no_std qsl-rt storage providers, core buffer contracts, direct heap InlineString restoration and optional NonZero InlineArray lengths.
- Verify actual generated code in separate no_std allocation-free and alloc fixtures.


### Development notes — 2026-11-13

- Derive per-array cursor windows from reachable bit residues; remove the schema-wide u128 fallback for COMPACT arrays.

- Add exact NumericDomain schema metadata, native 128-bit scalar types, wide cursor helpers and checked domain-width validation.
- Canonicalize compatible step=1 scalar hashes while preserving distinct array byte-barrier formats.



## 0.1.0 — 2026-11-08

Initial extraction of shared QSL schema/runtime contracts: errors, types, configuration, wire analysis, hashing, byte/bit I/O, checked validation, String/&str/StackString/InlineString storage, reusable buffers and portable codecs. No lexer or interpreter dependency.
