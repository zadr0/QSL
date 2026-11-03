# Checked schemas and data limits

[Русский](safety.ru.md) · [Guide](guide.en.md) · [Schema hashes](schema-hashes.en.md)

Every generated schema provides checked methods. The safety attribute is no longer required and remains a compatible spelling. Trusted pointer methods retain their separate contract; external bytes enter through checked APIs.

```text
!config MAX_DATA_SIZE = 65535; // default, bytes
!config USE_HASH = xxHash;    // or xxHash128
```

DEQ settings apply to each generated schema and are exposed as `Schema::MAX_DATA_SIZE`. Positive values up to isize::MAX and evaluated integer expressions are accepted. Duplicate definitions, including separate DEQ files, are rejected.

## Bounded quantities

| Quantity | Check point |
|---|---|
| Entire incoming frame | Before reading fields |
| Restored inner QPS frame | From the original-length header, before writing restoration |
| Aggregate decoded values | One shared budget while walking fields |
| Checked encoded frame | Before returning output |

Each quantity must fit MAX_DATA_SIZE. It is not a separate allowance for each string: two restored 48 B fields exceed a 64 B aggregate budget even if their compressed frame is smaller than 64 B.

Numbers charge their original native width, UTF-8 strings their restored byte length, and supported u8 arrays their element count. A QPS constant occupying zero value bits still contributes its original width. Metadata is bounded by frame length. Checked subtraction prevents aggregate overflow.

This is a per-message data limit, not a process RSS cap. Rust object overhead, InlineString capacities, tables and simultaneously occupied RT slots require separate memory planning.

## One validation walk

1. ValidationCursor::with_limit checks frame length and creates the aggregate budget.
2. Generated code visits metadata and fields in declared order, checking available bytes, numeric domains, lengths, UTF-8, padding, optional metadata and complete frame ending.
3. LZ checks tokens, back-references and restored byte counts in bounded stack storage. Validation allocates no heap buffer or temporary String.
4. Only after cursor completion does the trusted pointer decoder construct the result in final storage.

Validation does not serialize a comparison object, hash a message or use catch_unwind. Bit fields use a native little-endian window up to 128 bits instead of a per-bit loop. Validation and result construction remain separate stages; this is not a claim that the entire decoder reads memory only once.

An error in any field prevents pointer decoding. A u32::MAX string length in a short frame is rejected before String construction. Under the default limit, a QPB3 header declaring 4 GiB is rejected before restoration storage changes.

## Generated methods

```rust
let mut wire = [0u8; 1024];
let used = value.serialize_checked_to(&mut wire)?;
let input = &wire[..used];
Packet::validate(input)?; // validation only
let value = Packet::deserialize_checked(input)?;
let mut output = std::mem::MaybeUninit::<Packet>::uninit();
let value = Packet::deserialize_checked_into(input, &mut output)?;
```

Each deserialize_checked call validates exactly once; it does not need a preceding validate call. deserialize_checked_into supports owned and borrowed types. Borrowed result lifetimes follow the input frame or QPS restoration region.

serialize_checked_to validates values once, checks destination capacity, then uses the native writer. serialize_checked returns Result<Vec<u8>> with fallible reserve. Its compressed-string capacity is a raw upper bound and is conservatively rejected if it exceeds the limit. Prepared output checks actual encoded length; output bytes can already change if that final check fails.

General QPS methods are available under `[qps]` without safety: serialize_qps_to and deserialize_qps_into / deserialize_qps. Envelope and original lengths are checked first, followed by restoration and one inner-schema validation. Malformed tokens may change restoration before returning an error; an oversized declared length is rejected before writing.

## Errors and external codecs

Limit violations return QslError::SizeLimitExceeded, insufficient output capacity returns BufferTooSmall, and malformed domains, UTF-8, padding or frame ending return their corresponding errors. Owned-output fallible reserve can return AllocationFailed.

QslCustom supplies qsl_validate for incoming bytes and qsl_validate_values for checked encoding. Default implementations reject an opaque codec. A validation cursor does not prove arbitrary external code safe.

## One public macro dependency

```rust
#[qsl::qsl_macro]
pub struct Meta { pub id: u64 }
#[qsl::schem_macro]
pub struct Trace { pub meta: Meta, pub code: u16 }
```

Both attributes are available through qsl without the rust-macros feature. Their implementation lives under qsl/macros; Rust requires a separate internal proc-macro component. rust-macros enables only Syn-based Rust-source import by the generator. Register inputs with build_qsl_files_with_rust; the attribute itself does not discover files or change the original Rust type.

Limits and checked API availability leave wire bytes unchanged and are excluded from strict/light hashes. Different local limits can accept or reject the same frame as a resource policy. Descriptor version remains 1.

## Verification

Passed: 77 unit tests, 10 serial native/IPC/socket tests, 2 wire vectors, 12 hash checks and 2 compile-fail examples. A separate fixture has 6 integration scenarios and 7200 malformed-frame validator calls across raw/compact/LZ/borrowed/opt_table/predict_seq/state_bits/legacy/QPS. Strict Clippy, analyzer build and 5 main-feature regressions passed. Benchmarks were not rerun; published timings retain their recorded source versions.

```powershell
cargo test --manifest-path qsl/Cargo.toml --locked --all-features -- --test-threads=1
cargo test --manifest-path reference/safe-schema-control/Cargo.toml --locked
```
