# QSL deserialization modernization

Implementation contract, November 6, 2026. [Русский](MODERNIZATION.ru.md).

## Purpose and boundaries

QSL moves analysis of an agreed binary layout to build time. The schema defines declaration order, value domains and lengths. The generator emits concrete Rust code; runtime does not rediscover the schema. Attributes are opt-in. Required fields must precede optional fields, including after expansion of Rust templates.

Changing native storage does not necessarily change wire bytes. Frame length, `size_of::<T>()`, output capacity and process peak memory are distinct metrics. The analyzer's object size is not a function stack-frame size: the compiler may introduce temporaries or change how results are passed.

## Configuration and strict STACK_LIMIT

```text
!config STACK_LIMIT = 256;
!config MAX_DATA_SIZE = 65535;
!config QSL_RT = true;
!config QSL_RT_DEFAULT = 16;
!config USE_HASH = xxHash;
```

`STACK_LIMIT` bounds the maximum **data bytes of each inline buffer**, not the aggregate size of all fields. Length metadata and alignment add to native object size. Zero is allowed. This configuration does not change wire identity.

The limit is strict for automatic aggressive storage, explicit `binary_opt=inline`, `decompress=true`, fixed strings and arrays. Above it, COPY uses `String`/`Vec<T>` while preserving the string codec and array element width. Fixed native Transmutate storage cannot silently become a dynamic descriptor: generation fails and requires an explicit limit increase. LIFETIME raw strings borrow the source regardless of this limit; compressed strings cannot borrow their compressed representation and require COPY restoration.

## Ownership and initialized prefixes

| Case | Representation |
|---|---|
| Ordinary variable COPY string | `String` |
| Bounded aggressive COPY within the limit | `StackString<N>` |
| Explicit inline/LZ within the limit | `InlineString<N>` |
| Fixed string within the limit | `[u8; N]` |
| COPY string above the limit | `String`, unchanged wire codec |
| Raw LIFETIME / binary_opt=view string | Direct input `&str` |
| Bounded aggressive numeric array | `InlineArray<T, N>` when capacity is known |
| Compatible fixed array within the limit | `[T; N]` |
| COPY array above the limit | `Vec<T>` |

String lengths count **UTF-8 bytes**, not Unicode scalars. Array bounds count elements; storage selection multiplies by the element width. Inline length metadata is `u32`. Only `0..len` is initialized and readable. The remaining `MaybeUninit` storage is not zeroed. `InlineArray<T,N>` drops initialized elements only. Converting to `[T;N]` requires all N elements to be initialized.

New generated string paths contain no `Utf8Guard`/`Utf8CopyGuard` object. `binary_opt=guard` remains COPY compatibility syntax; `binary_opt=view` generates a direct `&str`. Legacy runtime wrappers remain available for existing clients. QSL-LZ remains a separate lossless codec with a raw branch when compression is unhelpful.

## Exact register reads

Consecutive required numeric fields are grouped automatically into chunks of at most 16 useful bytes. Ordinary groups currently call load_exact::<N>(), returning u128 with local zeroes above the N input bytes. The analyzer's registerBits rounds the logical width to 8/16/32/64/128 bits; it does not prove that the ordinary reader generates the corresponding minimum-width Rust integer type. A 25-byte group reads 16 + 9 bytes exactly. Concrete bit-reader/writer specializations have their own widths, and LLVM chooses physical instructions. A u128 does not imply one x86-64 hardware register.

Dynamic string/array contents terminate a group. A raw string's fixed length descriptor may join the preceding numeric group: `u64` plus a one-byte descriptor reads exactly 9 B, without requiring 16 readable bytes. Ordinary optional values with unknown offsets are not grouped this way. Static tables still select state and readers in combined modes.

The editor reports `readBytes`, `registerBits` and captured metadata. Absolute offsets remain unknown after variable content. It suggests grouping fixed required fields before variable fields; reordering changes wire format and remains an explicit developer decision.

## Safety: materialize and validate once

```qsl
&Packet[safety,aggressive] {
    u64 id;
    str(1..100) text;
    u32? status;
}
```

Checked deserialization is emitted only for `[safety]`. Explicit `validate`, value validation and checked serialization remain available controls. Schemas consuming untrusted input should enable safety.

`deserialize_checked` checks the minimum length first. A cursor then reads actual values while checking available bytes, numeric ranges, lengths, UTF-8, tags, valid predict_seq events and complete frame consumption. Ordinary COPY does not first scan the entire frame and then decode it again. A result is published only after success. `deserialize_checked_into` does not publish a result into the caller's uninitialized destination on failure.

`MAX_DATA_SIZE` bounds the input frame and aggregate useful data before allocation. A mixed QPS decoder deducts actual scalar-prefix useful bytes before allocating the tail; metadata is not charged again as useful data. Arithmetic overflow and fallible allocation return `QslError`. LZ checks restoration budget before growing its destination.

Trusted `deserialize_into` does not validate arbitrary bytes. Its preconditions include a complete valid frame of the agreed schema, initialized readable input, aligned exclusive output, no overlap, and correct destruction of any previous output value. Prepared-reuse destinations have additional storage requirements. Inlining does not remove these obligations.

## COPY versus LIFETIME Transmutate

`[transmutate,safety]` returns an owned native object. When native and wire layouts match, bytes are copied to aligned `MaybeUninit<Self>`, validated, then exposed through `assume_init`. `NATIVE_WIRE_COMPATIBLE` uses target-rustc offsets and size rather than assuming that the generator host layout matches the target. If layouts differ, COPY places validated fields into native locations.

`[transmutate,lifetime,safety]` exposes `deserialize_overlay` and `deserialize_checked`, returning `&Self` directly into input storage. Size, offsets, little-endian, alignment and value checks complete **before** forming a typed reference. Incompatible layouts are rejected during generation/compilation or by the checked API. The source must remain immutable and outlive the reference. The ordinary `Serializable` trait remains an owned-value interface; call the overlay API explicitly for a borrowed native view.

Whole-record overlays accept fixed required compatible fields only. COMPACT, QPS, dynamic and optional fields cannot be overlaid this way. Native padding is not useful data. `NATIVE_BYTES`, `NATIVE_ALIGNMENT` and `WIRE_MAX_BYTES` report separate contracts.

## Rust templates and nested regions

```rust
#[qsl::schem_macro(aggressive_transmutate)]
pub struct Meta {
    pub id: u64,
    pub value: u32,
    pub tag: qsl::InlineString<4>,
}
```

The macro retains a Rust type, provides repr(C), Copy/Clone and validated native-leaf metadata. In this mode `InlineString<4>` becomes `[u8;4]` with UTF-8 validation. Incompatible packed/Rust/transparent representations, dynamic storage and unsupported fields are rejected. Native metadata supports at most 64 leaf fields.

```qsl
&Target[safety] {
    Meta meta;
    str(1..10) name;
}
```

Template members expand to `meta_id`, `meta_value`, `meta_tag`; nested templates expand recursively. The original Meta remains a Rust struct. Register source files through `build_qsl_files_with_rust`. Cycles, collisions and required-after-optional order are rejected.

Alternatively, place `#region(aggressive_transmutate = "Meta");` before matching fields and `#end_region;` after them. Regions nest and contain fixed required fields only. `region_N_copy()` maps initialized leaves into native offsets. `region_N_view()` requires matching layout and alignment within the current object. With Safety, `deserialize_region_N_copy(input)` and `_view(input)` accept **the region bytes only**, without the surrounding frame. COPY reconstructs native padding from flattened leaves; input-view requires complete size and offset agreement. These wire-region APIs support little-endian. Validation never reads uninitialized native padding as data.

## Optional numbers and dispatch tables

Numeric optional fields are private: use `try_new(...)`, `field()` and `set_field(...)`. A domain excluding zero automatically uses `Option<NonZero*>`. Aggressive bounded domains use `value - min + 1`, reserving zero for None, choosing the smallest suitable type without widening the original scalar. Other full-domain numeric fields share a minimal presence bitmap. `Some(0)` remains distinct from None; signed domains restore exactly.

`predict_seq` computes a numeric length index and selects a valid sequence. `opt_table` maps a field tag to its reader; used alone, it preserves default wire bytes. `[predict_seq,opt_table]` retains the combined length graph and reader table. Impossible indices and inappropriate tags fail checked decoding. Ambiguous subsets cannot be recovered from length alone: required metadata is retained. Tables and specialized handlers are generated at build time.

## QPS: scalar numeric fields only

`[qps]` defaults to Q16; Q2/Q4/Q8/Q16 and the `quant` alias are supported. Operand packing is reversible. Strings, arrays, numeric array elements and every length descriptor are **not quantized**. Scalars must precede string/array fields; a violation is a schema error. QPS is not a predictor.

Mixed schemas pack a scalar prefix and use independent ordinary codecs for the tail. Variable packed-prefix sizes require an explicit 1/2 B boundary; fixed sizes are known from the schema. This new mixed representation has a distinct canonical identity mode, requiring coordinated peer updates. Existing ordinary and scalar formats are checked against golden vectors.

Generated prepared scalar-QPS reads no longer require a private readable tail: `QUANT_PADDING_BYTES = 0`. Wire size differs from restored Rust object size. Incompressible domains use raw fallback. A branch-free source formula does not prove branch-free machine code; inspect the actual specialization. The general runtime `ByteQps` remains a legacy API and does not define new schema quantization policy for strings.

## RT slots, cache and progress

`slots=1..64` overrides `QSL_RT_DEFAULT` for a particular schema. With `QSL_RT=true`, eligible bounded schemas receive a static output pool. `serialize()` waits when slots are exhausted: Windows WaitOnAddress, supported Linux futex, compatible waiting mechanisms elsewhere. A lease owns its slot until Drop. Active leases never overlap. `try_acquire` remains nonblocking. A thread holding all slots must not synchronously request another without arranging release: nobody could free one. A static pool is process-local; it is not an IPC region.

The wait predicate is availability, rather than a wrapping epoch counter. Release publishes a free slot before incrementing availability and waking waiters. Tests cover blocking, exclusive concurrent ownership and resumption. Lean models slot-count conservation only.

`[loyal_cache]` uses a proven wire upper bound. `cache=N` may choose a larger capacity; insufficient capacity is rejected. `serialize_cached(&mut CachedFrame<N>)` establishes capacity at monomorphization, validates values, and writes without a repeated capacity comparison. Input bounds remain checked. Alignment and bookkeeping affect `size_of::<CachedFrame<N>>()`; `CACHE_BYTES` is data capacity, not total RAM consumption.

## Editor, validation and migration

Zed 0.8 provides RU/EN diagnostics, UTF-8 constant-byte checks, size/offset inlays, field hovers, outline, wire/native memory inspectors, register groups, native regions and generated Rust views. Build the native analyzer from the matching QSL version with `--features rust-macros`. One invalid schema no longer hides valid neighboring schemas; normal builds still reject an invalid source file. Unknown external Rust layout is reported as unknown rather than guessed.

Regenerate code after upgrading: numeric Option fields use accessors, checked decoding requires safety, and large inline fields change native type while retaining their wire codec. Negotiate new mixed QPS frames by identity. Historical results and source snapshots retain their historical contract and are not current measurements.

Verification commands and four narrow Criterion cases live in `qsl/tests/modernization`, `qsl/tests/optional-modes`, and `qsl-bench/modernization`. Warm-up: 15 s; measurement: 10 s; samples: 200; confidence: 99%; process limit: 8 GiB. P = M/T uses original useful bytes, not compressed wire bytes. Trusted-read timings exclude validation and do not establish untrusted-input throughput. Actual measurements and commands are attached separately.

`proofs/Modernization.lean` checks arithmetic models for read bounds, initialized prefixes, ranks, None/Some, capacity, mixed-frame boundaries and slot conservation. These are not formal verification of the entire Rust/LLVM program.

## Trusted-writer and neighboring-field boundaries

Serializable::serialize and BorrowSerializable::serialize are now unsafe trait methods: callers guarantee domains, exact lengths and an aggregate encoded size fitting isize::MAX. Use serialize_checked / serialize_checked_to for safe writing. Generated RT/QPS serialize() methods remain safe Result APIs with value validation. This is an explicit API migration, independent of the safety decoding attribute.

region_N_view() additionally checks that the entire native target, including trailing padding, ends before the next parent field. Matching initialized leaves alone is insufficient: an overlay must not cover a neighbor. Leaf-wise COPY remains available when padding layouts differ.


## Unchecked APIs remain independent of safety

The safety attribute adds checked entry points without disabling direct unsafe access. General Serializable::deserialize, deserialize_from_raw_parts and deserialize_into, together with their borrowed equivalents, remain unsafe and do not invoke a validation pre-pass.

| Mode | Always available unsafe entry point |
|---|---|
| Native COPY | deserialize_transmutate_copy_unchecked(input) → Self |
| Native LIFETIME | deserialize_overlay_unchecked(input) → &Self |
| Region COPY from an object | region_N_copy_unchecked(&self) → Native |
| Region view from an object | region_N_view_unchecked(&self) → &Native |
| Region COPY from wire | deserialize_region_N_copy_unchecked(input) → Native |
| Region view from wire | deserialize_region_N_view_unchecked(input) → &Native |

Names are generated when the corresponding schema mechanism is enabled; region methods exist for every region N. These methods do not check lengths, alignment, UTF-8, domains or MAX_DATA_SIZE, and do not call the NativeField validator. COPY transfers initialized leaves without reading padding as data. Selecting native COPY versus ordinary decoding through NATIVE_WIRE_COMPATIBLE is a type constant, not input validation. predict_seq still uses length to choose a state: unchecked removes validation, not computations necessary to interpret the format.

Callers guarantee a complete valid frame, valid values, a valid readable allocation and compatible layout. Views additionally require alignment, immutable storage and a lifetime covering the returned reference; a native view must not cover a neighboring field. Wire-region APIs take the region bytes only, not the parent frame.

REGION_N_COPY_COMPATIBLE, REGION_N_VIEW_COMPATIBLE, REGION_N_WIRE_COPY_COMPATIBLE and REGION_N_WIRE_VIEW_COMPATIBLE support strategy selection before a call. They describe layout compatibility, and the self-view flag also accounts for the next field's boundary. Actual address alignment and value validity remain caller obligations. Unchecked methods do not read these flags or assert their value.

Existing safe entry points retain their behavior. Without safety, the legacy unsafe names deserialize_transmutate_copy and deserialize_overlay forward to the corresponding _unchecked methods. With safety, those legacy names remain checked Result APIs. The new explicit _unchecked names keep the same contract when the attribute changes.
