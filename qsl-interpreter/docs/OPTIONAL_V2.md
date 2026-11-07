# Optional dispatch v2 and native regions (0.2.0)

QSL 0.2.0 consists of the qsl-lexer frontend, qsl-interpreter Rust backend and portable qsl runtime. Bitmap encoding changes deliberately; peers coordinate migration. Windows-specific sockets/IPC modules, the native Cargo feature and the Windows-library dependency are removed. Native-layout generation remains platform independent.

## Three separate mechanisms

| Mechanism | Input | Purpose | Ordering |
|---|---|---|---|
| Default | Optional count and tags | Dispatch through a generated `match` | Existing ascending tag order |
| BM, `optionals=bitmap` | Native-width presence mask | Visit set bits only | Descending optional marker |
| OT, `opt_table` | A validated marker | Select a generated reader function | Follows the active format |
| PS, `predict_seq` | Payload length | Select a possible presence sequence | Independent of OT; disabled when BM is active |

QPS is operand-based reversible compression. It does not predict optional fields. None of BM, OT or PS enables QPS implicitly. Optimization attributes remain opt-in; required fields must precede optional fields.

## Bitmap wire revision 2

Width depends on the number of optional fields, not all fields: 0 / 1–8 / 9–16 / 17–32 / 33–64 / 65–128 fields use 0 / 1 / 2 / 4 / 8 / 16 bytes. The word is little endian. Optional marker 1 occupies bit `WIDTH-1`. Required fields do not occupy bits. A schema supports at most 128 optional fields in BM mode.

The reader uses `marker = WIDTH - mask.trailing_zeros()` and `mask &= mask - 1`. Thus the last declared present optional field is read first. The writer emits optional values in that same order; required fields retain declaration order. There is no per-absent-field iteration.

```qsl
&P[safety,optionals=bitmap,opt_table,predict_seq] {
    u8 id;
    u16? a;
    u8? b;
}
```

For `id=9, a=Some(0x1234), b=Some(7)` the bytes are `C0 09 07 34 12`: mask, id, b, a. PS is inactive for this schema; OT is active. Without OT the same markers select a `match`, with identical bytes.

**Migration:** this intentionally replaces the previous bitmap format. Old and new BM peers must be upgraded together or negotiated outside the frame. BM canonical schema identity includes revision 2 and its width; toggling inactive PS or OT does not change those bytes. A schema hash is not a protocol handshake or an authentication check.

Non-BM count/tag headers are preserved: tags are read from the existing metadata header, then a dynamic loop dispatches their values. Tags are not moved between payload values. One optional field keeps its existing one-bit presence header. Expanded or manually constructed schemas retain absolute tag semantics; dense optional ordinals are an internal dispatch detail.

## Safety and trusted entry points

For a nonzero BM, the checked path validates its largest marker before traversal. `WIDTH - trailing_zeros(mask) <= optional_count` rejects spare aligned bits. Every later marker is smaller, so a dense OT lookup is within bounds. Zero masks skip this check. Tagged input validates count, strictly increasing tags and membership before dispatch. A checked cursor then verifies field bounds, ranges, UTF-8, compressed output size, restored-data budget and end of frame.

`deserialize_checked`, `deserialize_checked_into`, `validate`, and checked serialization retain their contracts. A failed checked-into call does not publish a partially initialized destination. Materialized temporary fields are dropped on failure. Custom codecs must implement their existing validators; opaque validators reject safe decoding by default.

Unchecked `Serializable` / `BorrowSerializable` and raw destination entry points remain available. Their caller must provide a valid schema-specific frame, enough initialized storage for any documented register loads, correct lifetimes and aliasing. They do not become safe because another frame was previously validated.

The checked restored-data budget charges a compact custom value's `SIZE`, while consuming `COMP_SIZE` wire bytes. Custom arrays check their associated compact width before allocation and validate each element before checked writing. A custom codec's own hidden allocations remain its responsibility.

## One optional reader, several dispatchers

BM and ordinary tagged paths generate one cursor reader per optional field. OT contains those readers in optional-marker order. Default `match` calls the same readers. PS+OT reuses the existing PS-compatible OT handlers instead of copying their decode body into each predicted sequence. Checked and unchecked readers are separate implementations because their contracts differ. Validation-only walks may emit their own field checks; this is not a claim that every machine instruction occurs once in the final binary.

## Explicit scalar wrapper macros

A schema field-expansion macro and a scalar serialization contract serve different purposes. Existing user `QslCustom` codecs remain opaque and retain their byte format. They are not automatically reinterpreted as integers.

In a Rust module shared by the application and build script:

```rust
qsl_lexer::qsl_primitive!(pub struct DeviceId(u32);range=(1,100););
```

Or bind an existing one-field integer tuple struct:

```rust
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DeviceId(pub u32);
qsl_lexer::qsl_primitive!(DeviceId,u32,range=(1,100));
```

Supported representations: u8/i8, u16/i16, u32/i32 and u64/i64. The macro checks integer kind, size, alignment, field-zero offset, absence of Drop and domain bounds. It constructs values with typed Rust constructors. Normal encoding is little endian; compact encoding stores `value-min` in the minimum whole-byte width for the declared domain. This is byte-oriented custom compression, not automatic bit merging with neighboring built-in fields. A constant domain occupies zero compact bytes. Existing PS restrictions on zero-width optional codecs still apply.

Register the same type before parsing:

```rust,ignore
use qsl_interpreter::prelude::*;
let builder = QslSchemaBuilder::new()
    .with_primitive::<DeviceId>("DeviceId")?
    .with_source("&Packet[safety,compact,optionals=bitmap,opt_table]{DeviceId id;DeviceId? extra;}")?;
let rust = builder.use_interpreter()?;
```

Registration checks build-side ABI and retains normal custom width. Generated const assertions check target-side size, alignment, Drop and codec widths, including array elements. Primitive metadata is reapplied after explicit Rust-template imports. Registered primitives can also appear as QSL array elements; unregistered opaque array elements are rejected by the source parser.

For native regions, the application additionally opts into the existing runtime contract:

```rust,ignore
qsl_lexer::qsl_native_primitive!(DeviceId,u32);
```

The native macro validates scalar values before creating a checked view. Manual `unsafe impl QslPrimitive` / `NativeField` remains an explicit responsibility: layout checks cannot prove arbitrary Rust method semantics. No general user-type transmute or automatic merged-register kernel is introduced. Nested optional schemas inside a custom codec keep their own masks and marker scope.

## Real nested native regions

```qsl
&Parent[safety] {
    #region(aggressive_transmutate="Meta");
    u64 id;
    u32 value;
    str(4) tag;
    #end_region;
    str(3..10) name;
}
```

This produces `#[repr(C)] Meta { id:u64, value:u32, tag:[u8;4] }` and `Parent { Meta:Meta, name:String }`. Region fields are not flattened into Parent. The historical `aggresive_transmutate` spelling is accepted as an alias.

COPY owns the nested object and falls back to field reconstruction where native and wire layouts differ. LIFETIME borrows a typed region only when wire/native sizes and field offsets match, the host is little endian, input alignment is sufficient, and all field validators succeed. An ordinary Vec buffer does not promise alignment for arbitrary native types; misaligned checked borrowing returns an error. COPY does not require borrowed-input alignment.

Nested regions produce a type hierarchy. Identical definitions are reused; conflicting names, crossed/empty regions, recursive names, unknown native types, dynamic fields and optional fields inside native regions are rejected. Fixed strings use byte arrays. The configured stack limit applies to native string/array fields too. Native descriptors support at most 64 initialized leaves; numeric arrays use their existing coalesced leaf representation. Region nesting is capped at 64.

A Rust `#[qsl::schem_macro(aggressive_transmutate)]` template reuses its existing Rust type as a nested field; the generator does not emit a duplicate struct. Ordinary field-expansion macros keep their previous flattening behavior. Explicit `repr(align(...))` is accounted for; extra padding can make LIFETIME unavailable while COPY remains usable.

## Evidence and limits

See the standalone [test harness](../tests/optional-v2/README.md), the [Lean arithmetic proofs](../../qsl-lexer/proofs/README.md), and the focused benchmark report produced by that harness. Lean proves arithmetic index bounds under documented integer assumptions; it does not prove Rust pointer provenance, user trait promises, allocator behavior or generated assembly. Passing deterministic malformed-frame tests is not an exhaustive memory-safety proof.
