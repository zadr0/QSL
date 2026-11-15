# QSL schema identity

[Русский](schema-hashes.ru.md)

QSL computes two hashes while analyzing the fully expanded schema. Generated Rust contains literal constants. Reading and writing do not hash a schema, allocate hashing storage, or insert a digest into a record.

```text
!config USE_HASH = xxHash;    // default: XXH3-64
```

Set `!config USE_HASH = xxHash128;` in DEQ for XXH3-128. Explicit aliases `XXH3_64` and `XXH3_128` are accepted. This is a build setting, not a per-message switch. Duplicate settings, including definitions in separate DEQ files, are rejected. Omitting the setting selects XXH3-64.

## Two identities

| Value | Contents |
|---|---|
| Strict | Schema and field names, field sequence, types, domains, optionality and format parameters |
| Light | The same data format and constraints, without schema or field names |

Fields retain declaration order; they are not sorted by name. `&A{u32 id;}` and `&B{u32 sequence;}` have equal light hashes and different strict hashes. Reordering differently typed fields changes both. Renaming or exchanging otherwise identical field descriptors leaves light identity unchanged; the application assigns semantic meaning to names.

## Included

- Ordered fields after DEQ expression evaluation and recursive Rust-template expansion.
- Numeric types and domains, UTF-8 length bounds, array elements and lengths, padding and optional presence.
- Value and metadata formats: compact, effective alignment, bitmap, predict_seq, predict_seq+opt_table, the legacy directory and state_bits. Standalone opt_table preserves default bytes and is excluded as a reader-selection policy.
- String compression, InlineString's dedicated length header, QPS selection and Q2/Q4/Q8/Q16. The separate QPB3 envelope is represented: the identity describes the schema's selected codec family, not one serialized object.
- Used optional tags and tag width in the ordinary tagged format.

## Excluded

`inline`, `inline=always/never`, `safety`, lifetime, internal template markers, numeric `default`, `QSL_RT` and slot counts. These affect API availability, storage, memory preparation or compiler decisions rather than the selected encoding of explicitly supplied values.

`String`, borrowed `&str`, `StackString` with the ordinary codec and `binary_opt=view` normalize to the same UTF-8 descriptor when bounds match. Retired guard/true values are not valid current-branch string configurations. `binary_opt=inline` remains distinct because it selects a different length header in this implementation. `decompress=true` selects a string codec; fixed strings below the compression threshold retain raw-format identity.

`[qps]`, `[qps=Q16]` and `[quant=Q16]` are equivalent. `[opt_table]` now has the default hash. `[predict_seq,opt_table]` retains mode 3 and the hashes of former mixed `[opt_table]`. This changes attribute selection; add predict_seq for old frames. The static `state_bits=Qn` table quantum is excluded, while the state_bits wire format remains included. Alignment without compact is excluded.

This is a conservative identity of the canonical format and domains, not a complete equivalence proof for arbitrary decoder implementations. Different domains remain different contracts even when particular values produce equal bytes.

## Rust API

Every generated type, including borrowed and scalar-QPS types, provides:

```rust
let identity = Packet::SCHEMA_HASHES;
let strict: u128 = Packet::SCHEMA_HASH_STRICT;
let light: u128 = Packet::SCHEMA_HASH_LIGHT;
// Also: <Packet as qsl::SchemaIdentity>::SCHEMA_HASHES
```

`SchemaHashes` contains `version`, `algorithm`, `strict`, `light` and `complete`. XXH3-64 uses the lower 64 bits of u128; the upper bits are zero. Explicit `strict_hex()` and `light_hex()` calls format 16 or 32 hexadecimal digits; ordinary I/O does not create these strings.

Build-time users can call `QslSchema::hashes(algorithm)` and `hash_descriptor(SchemaHashKind::Strict/Light)`. The latter exposes the canonical descriptor for exact comparison. Explicitly calling these analysis functions performs analysis; generated serializers do not call them.

Compare algorithm, descriptor version and completeness together with a digest. Changing USE_HASH does not change the payload. These additions do not implement a QBProto handshake or automatic identifier transmission.

## Determinism and limits

Canonical descriptor version is **1**. Its prefix is `QSL-SCHEMA\0`, followed by a u16 LE version and strict/light discriminator. All lengths and sizes are u64 LE, numeric words i128 LE, and text is length-prefixed. The descriptor does not depend on native usize bytes, Debug output, HashMap iteration, source whitespace or comments. XXH3 uses seed zero. Changes to descriptor rules or represented wire codecs require a version bump.

An opaque QslCustom contributes its Rust type path and declared fixed size, and sets `complete=false`. QSL cannot inspect external method implementations: agree on that codec and version separately. Expanded Rust templates contribute their actual built-in fields rather than an opaque template name.

[xxHash](https://xxhash.com/) is non-cryptographic. The 128-bit option reduces accidental collision probability but provides neither authentication nor mathematical uniqueness. Canonical descriptors are available for exact structural comparison.

## Verification and Zed

`qsl-analyze` returns `hashes` for each schema; `--generate` emits matching Rust constants. JSON uses hexadecimal strings to preserve all bits in JavaScript. Capability `schemaHashVersion: 1` lets the extension detect an outdated analyzer.

Zed validates and completes USE_HASH values. Its RU/EN layout inspector shows both digests, algorithm, version and completeness. Advisory JavaScript analysis never invents a digest; hash data comes from the native analyzer.

```powershell
cargo test --manifest-path qsl/Cargo.toml --locked --all-features -- --test-threads=1
cargo test --manifest-path reference/schema-hash-control/Cargo.toml --locked
cargo build --manifest-path qsl/Cargo.toml --locked --release --bin qsl-analyze --features rust-macros
Get-Content examples/schema-hashes/packet.qsl -Raw | ./qsl/target/release/qsl-analyze.exe --generate examples/schema-hashes/schema.deq
```

Tests pin both algorithms' golden digests, check excluded settings and included format changes, expand nested templates, merge independent DEQ settings and repeat native analysis across processes. A separate fixture compiles owned/borrowed/QPS/RT output and compares real frames when switching hash width. No benchmark timings were added: hot I/O implementations are unchanged.

## Bitmap revision 2

BM canonical descriptors include revision 2 and mask width. Required fields no longer occupy bits; traversal is descending optional marker order. OT and BM-inactive PS are code-generation policies and do not alter that identity. Native/custom type identities remain incomplete when their independent codec semantics are opaque. Hashes do not negotiate versions or prove collision-free compatibility. [Migration](migration-0.2.0.en.md).
