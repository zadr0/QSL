# Quick Schema Language — reference

Schema-driven binary serialization for Rust.


## Purpose and invariants

This documentation describes QSL 0.4.0 with qsl-core 0.2.0.

QSL exchanges explicitly described data between peers that control the schema, value bounds and memory contract. Build-time analysis produces concrete Rust types and specialized binary methods. The project develops as an independent implementation based on practical experience and a logical model.

Its scope includes internal protocols, telemetry, repeated records and applications with known data bounds. Specialization requires precise ranges, lengths, ownership, optional states and peer compatibility.

Invariants:

1. Additional attributes/mechanisms are optional; required fields must precede optional fields.
2. Values restore exactly within their declared domains.
3. An absent optional has no value payload and does not acquire an initializer/default.
4. Optimization never authorizes a read of unavailable bytes or uninitialized slots.
5. The schema and wire-affecting configuration define the format; physical Self layout is separate.
6. Checked APIs validate; explicit unsafe APIs retain caller obligations.

Version negotiation, routing, encryption, authentication and delivery policy belong to the application/transport. QBProto and an automatic schema-set handshake remain unimplemented.


## First project and build.rs

This example was compiled and executed. The relative dependency path fits an application two directories below dev-qsl; adjust it for another location. Build and application dependencies must target the same implementation.

### Cargo.toml

```toml
[package]
name="qsl-array-usage-example"
version="0.0.0"
edition="2021"
[dependencies]
qsl={path="../../proofs/qsl"}
[build-dependencies]
qsl={path="../../proofs/qsl"}
[workspace]
```

### build.rs

```rust
use qsl::{InterpreterBuilderExt, QslBuilder};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = QslBuilder::new();
    builder.config(|c| {
        c.max_data_size(65535).stack_limit(256);
    });
    builder.schem(|s| {
        s.name("Packet").safety().compact().auto();
        s.field("prefix").u8(0, 7, 0, false);
        s.field("values").farray::<u16>(16, false);
        s.field("tail").u8(0, 255, 0, false);
    });
    let schemas = builder.build()?;
    let rust = schemas.use_interpreter()?;
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    std::fs::write(out.join("qsl.rs"), rust)?;
    Ok(())
}
```

farray::&lt;u16&gt;(16,false) declares exactly 16 elements. Type methods take optional as the final argument; numeric methods use (min,max,default,optional). Default initializes an object and never fills an absent optional during decoding.

Safety enables checked decoding, compact selects wire packing, and auto permits physical declaration reordering while retaining wire order. These settings are independent.

### src/main.rs

```rust
#![allow(dead_code, unused_mut)]
use std::mem::MaybeUninit;
include!(concat!(env!("OUT_DIR"), "/qsl.rs"));
fn main() -> qsl::QslResult<()> {
    let packet = Packet { prefix: 5, values: [42; 16], tail: 7 };
    let mut buffer = [0u8; 64];
    let used = packet.serialize_checked_to(&mut buffer)?;
    let mut output = MaybeUninit::<Packet>::uninit();
    let decoded = Packet::deserialize_checked_into(&buffer[..used], &mut output)?;
    assert_eq!(decoded.prefix, 5);
    assert_eq!(decoded.values, [42; 16]);
    assert_eq!(decoded.tail, 7);
    assert_eq!(used, 34);
    assert_eq!(std::mem::size_of::<Packet>(), 34);
    println!("Array example passed: wire={used} B, Self={} B", std::mem::size_of::<Packet>());
    Ok(())
}
```

Verified output: wire **34 B**, Self **34 B** with auto. The measured version of this wire model without auto has a 36 B Self. Smaller Self does not itself prove lower latency.

Textual field description:

```qsl
&Packet[safety,compact,auto] {
    u8(0..7) prefix;
    array<u16>(16) values;
    u8 tail;
}
```

For files, select QslSchemaBuilder::from_files(...).use_interpreter(), or use:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    qsl::build_qsl_files(&["packet.qsl"], &["settings.deq"], out)?;
    Ok(())
}
```

The wrapper writes qsl_gen.rs. The programmatic example chose qsl.rs itself. The include! filename must match the generated file.


## From a schema to bytes

QSL analyzes schemas during the build. Runtime uses the generated Rust code.


Memory order, wire order and schema semantics are distinct. Physical auto can reduce padding; explicit auto-wire v1 changes canonical wire order separately. QPS compresses values, predict_seq analyzes length, and opt_table dispatches a reader by tag.
[Compiler](/internals/architecture/) · [Safety](/guides/safety/)


## Declarations and field order

A schema names a message and declares its fields in wire order. In the ordinary format, required fields have no tag; optional fields must follow them. The compiler assigns tags.

```qsl
&User[safety] {
    u64 id;
    str(1..100) name;
    u8? age;
}
```

`?` permits absence. An empty string differs from an absent string. Object initializers do not replace absent wire values.

Schema attributes follow the name in brackets; field modifiers affect that field. Additional mechanisms are optional; required → optional order is mandatory.

[Types](/language/types/) · [Builder arguments](/language/deq/) · [Attribute combinations](/reference/attributes/)


## Schema language and types

Choose a value type, then declare its constraints. The generated Rust representation may depend on storage policy: a string can become String, &str or bounded storage.
Family | Meaning
---|---
`u8…u128 / i8…i128` | 8–128-bit integers; intervals and discrete domains.
`f32 / f64` | Floating-point values using their selected codec.
`bool / char / flags` | Boolean, Unicode scalar and bit flags.
`str / string` | UTF-8 with fixed or bounded byte length.
`array<T>` | An array with element codec T and its own length.
`Custom` | A custom type with an explicitly registered contract.
[Domains](/language/numbers/) · [Strings](/language/strings/) · [Arrays](/language/arrays/) · [Custom types](/guides/custom-types/)


## Numeric domains

$$
N=max-min+1,\qquad w=\lceil\log_2N\rceil,\qquad rank=x-min.
$$


Widths use exact bit_length(max−min), without floating point. Constants use zero bits. Full u128 cardinality is represented without overflow. Signed domains use validated native bit patterns and appropriate wrapping arithmetic.

Raw stores original LE integers. Ordinary COMPACT stores ranks LSB-first. Equal Rust types do not imply raw/COMPACT compatibility. Alignment inserts byte barriers after fields without reordering them.

```qsl
&Discrete[safety,compact] {
    i32(-9..9) temperature|step=3|;
    i128 state|values=[-1,0,1]|;
}
```

A positive step maps $r=(x-\min)/s$ and must reach the inclusive upper endpoint. Sets are nonempty/unique, with canonical numeric order. Combining them requires each listed value to satisfy the interval/progression. Raw keeps native values and validates constraints. Checked decoding rejects spare ranks before reconstruction. Step/values are optional local syntax, not mandatory QSL mechanisms.


## Strings, arrays and ownership

A QSL string is UTF-8 text. Bounds are bytes: “Яa” occupies 3 B. Limits apply to restored text, so a forged compressed length cannot authorize an out-of-bounds write.

## How storage is selected

**String** owns a dynamic buffer. **&str** borrows validated raw text from a live input. **StackString** and **InlineString** keep bounded bytes inside the object. Fixed strings use **[u8; N]** in supported fixed mode.

Storage depends on lifetime, aggressive, the codec and STACK_LIMIT. The limit also applies to explicit inline modes. new creates a destination value; reuse reuses eligible storage. These are decoding strategies, not additional string types.

## When a string compresses {#compressed-strings}

QSL-LZ checks bounded repetition candidates and selects compression only when it actually saves space. Otherwise it stores raw bytes. Frequent characters alone do not guarantee savings: match length, distance and metadata matter.


Arrays have a [separate guide](/language/arrays/); [QPS](/guides/smaller-frames/) is a different reversible codec.

InlineString keeps its codec in stack storage or Box on the heap. Large heap strings restore directly into their destination. StackString and InlineArray store their bounded capacity inside the object. Explicit aggressive[shift=true] can select a NonZero length word for strings and arrays while keeping None distinct from Some(empty).

no_std excludes std, not alloc. qsl-core always uses #![no_std]; its default alloc feature provides alloc String/Vec, SerOut and heap restoration. The application supplies the allocator; QSL does not declare a global allocator. Disable default features for caller-buffer-only operation.

[Limits](/reference/limits/) · [Memory](/guides/manual-layout/)

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.


## Arrays and bit windows

An array stores its element count. Strings instead count UTF-8 bytes.

**Variable length:** `array<u16>(0..16)` permits zero to sixteen elements. **Fixed length:** `array<u16>(16)` requires exactly sixteen. The element codec determines wire size; the Rust container determines ownership and memory placement.

Vec owns a separate allocation. InlineArray keeps bounded capacity inside the object. Fixed arrays may use [T; N]. STACK_LIMIT constrains the choice; a Vec descriptor is not the array payload.

Size in bits:

$$
B_{\mathrm{array}}=H(n)+\sum_{i=1}^{n} w_i.
$$

H is length metadata, n the element count and w the element codec width. Codec alignment is additional. Array length never authorizes reading beyond the frame.

Nested arrays support up to 32 levels. Element domains and array lengths are checked independently. Reader registers depend on bit position and operations; they are not always 128-bit.
[Read windows](/internals/optimizer/) · [Memory](/guides/memory/)

InlineString keeps its codec in stack storage or Box on the heap. Large heap strings restore directly into their destination. StackString and InlineArray store their bounded capacity inside the object. Explicit aggressive[shift=true] can select a NonZero length word for strings and arrays while keeping None distinct from Some(empty).

no_std excludes std, not alloc. qsl-core always uses #![no_std]; its default alloc feature provides alloc String/Vec, SerOut and heap restoration. The application supplies the allocator; QSL does not declare a global allocator. Disable default features for caller-buffer-only operation.


## PS, OT and bitmap

Required values have no individual tag in the ordinary format. Optional metadata follows the selected codec: one presence bit, count/tags or BM. Absence never writes a default value.
Mechanism | Role
---|---
Default | Base presence encoding
predict_seq / PS | Payload length → numeric state/index → presence
opt_table / OT | Tag → static reader function
PS+OT | Agreed length graph plus tag dispatch
optionals=bitmap / BM | Optional-only wire mask
Length alone cannot distinguish every same-sized combination; tag skipping requires a proven unambiguous schema/format. OT does not replace PS, and QPS does not predict optional states.

BM v2 selects u8/u16/u32/u64/u128 from optional count. Its set-bit traversal defines payload order, which need not equal default order. Peers must agree on the revision.

Aggressive bitmap is physical Self storage: private MaybeUninit slots, native tag−1 presence bits and get/set/clear methods. Absent slots are not read; Drop touches only occupied slots. This is independent of wire BM.

Ordinary numeric optional storage may use separate payload/mask or a proven NonZero niche. Prefer try_new/accessors over assuming a public Option&lt;T&gt; field. New DomainInteger-specific niche lowering remains pending. opt_table=legacy and state_bits are explicit compatibility formats, not aliases for current PS/OT/QPS.


## Builder and DEQ

The Builder describes schemas in Rust. DEQ stores constants and shared settings. Both use the same analysis and format rules; matching inputs must produce the same result.

## How a schema is built

QslBuilder holds settings and schemas. A schem callback calls name, field and type methods; build validates the declarations. use_interpreter selects Rust generation. [Complete build.rs](/start/quickstart/).

## Methods and arguments



## DEQ settings

### MAX_DATA_SIZE {#max_data_size}

Default: **65535 B**. `max_data_size(bytes: u32)`.

Bounds input frames, restoration and aggregate data before allocation. Not an RSS limit.

### STACK_LIMIT {#stack_limit}

Default: **256 B**. `stack_limit(bytes: u32)`.

Separate strict limit for inline strings/arrays. A larger max_data_size does not increase it.

### QSL_RT {#qsl_rt}

Default: **false**. `with_rt()`.

Enables static slots only for eligible bounded schemas.

### QSL_RT_DEFAULT {#qsl_rt_default}

Default: **16**. `default_rt_slots(slots: u32)`.

Concurrent slots per eligible schema: 1–64. Exhaustion returns an error.

### USE_HASH {#use_hash}

Default: **xxHash**. `use_xxhash128()`.

xxHash selects XXH3-64; xxHash128 selects XXH3-128. [Hashes](/guides/schema-identity/) do not provide authentication.

### CPU_PROFILE {#cpu_profile}

Default: **portable**. `cpu_profile(CpuProfile)`.

portable or kaby_lake. Changes eligible instruction choices, not wire identity.

## Constants

Typed integer constants are evaluated during the build, including exact u128/i128 values. Duplicate names, overflow and invalid values produce diagnostics.

```qsl
!define NAME_MAX: usize = 100;
!config MAX_DATA_SIZE = 65535;
!config STACK_LIMIT = 256;
!config QSL_RT = false;
!config QSL_RT_DEFAULT = 16;
!config CPU_PROFILE = portable;
!config USE_HASH = xxHash;
```


## Macros, primitive wrappers and custom codecs

QSL supports three distinct extensions. A **template** contributes fields to a schema. A **primitive wrapper** gives an integer a Rust type and a range. A **custom codec** supplies its own fixed-size binary representation. Select the smallest contract that describes your type.

## Rust templates

Place `#[qsl::qsl_macro]` or its synonym `#[qsl::schem_macro]` on an ordinary Rust struct with named fields. The Rust struct remains available. The attribute implements `QslMacro`: during the build, it returns field descriptions, not serialized values.

A template field named `meta` containing `id` produces `meta_id`. Nested templates expand recursively: `meta.location.zone` becomes `meta_location_zone`. Tags are reassigned after expansion. Consequently, optional fields in a template must still end up after every required field of the resulting schema. Cycles, duplicate names and unsupported optional template groups are errors.

~~~rust
#[qsl::qsl_macro]
struct Location { zone: u16 }

#[qsl::qsl_macro]
struct Meta { id: u64, location: Location }

let mut builder = qsl::QslBuilder::new();
builder.schem(|schema| {
    schema.name("Packet").safety();
    schema.field("meta").r#macro::<Meta>();
});
let schemas = builder.build()?;
~~~

The generated Packet contains `meta_id: u64` and `meta_location_zone: u16`. It does not contain a nested Meta field. Field expansion is separate from the native-region mechanism, which deliberately creates a nested Rust type.

Source-file import requires the `rust-macros` build feature and `from_files_with_rust` or `build_qsl_files_with_rust`. The importer reads supported Rust definitions; it does not execute arbitrary Rust expressions. The typed Builder uses `QslFieldType` for built-in fields and `QslMacro` for nested templates. Generic and conditional source declarations have restrictions; diagnostics must be resolved rather than treating an unknown type as a byte array.

## Integer wrappers

`qsl_primitive!` defines or binds a one-field tuple struct. The declaration form adds `repr(transparent)`, Copy and Clone. The binding form checks the existing type. Supported representations are the sealed integer primitives up to 64 bits; built-in QSL u128/i128 fields do not extend this macro contract.

~~~rust
qsl::qsl_primitive!(pub struct DeviceId(u32); range=(1,100););

let builder = qsl::QslSchemaBuilder::new()
    .with_primitive::<DeviceId>("DeviceId")?
    .with_source("&Packet[safety]{DeviceId id;}")?;
~~~

The macro checks size, alignment, tuple-field offset zero, the absence of Drop and the validity of the range at compile time. It implements `QslPrimitive` metadata and `QslCustom` reading, writing and validation. Register the same definition before parsing a schema that names it. A name alone does not prove the representation of an external type.

In raw mode, DeviceId uses the little-endian u32 representation. Its custom compact codec stores a byte-rounded rank. For this example, 100 values need 7 rank bits and occupy 1 custom-codec byte. This is different from claiming that every custom field is packed bit by bit with its neighbours.

$$
N=b-a+1,\qquad r=x-a.
$$

$$
w=\lceil\log_2 N\rceil,\qquad B_{\mathrm{custom}}=\lceil w/8\rceil.
$$

For a constant domain the rank width is zero. Spare ranks and out-of-range raw values are rejected by checked methods. The public tuple field can still be assigned an invalid value; only validation or a restricted application constructor makes a particular value valid. The macro does not automatically create an Option niche.

## Native primitive validation

`qsl_native_primitive!(DeviceId, u32)` additionally implements `NativeField` in the application. Use it after primitive registration when an explicit native region contains that wrapper. It checks representation metadata and validates the numeric domain before a checked native copy/view can publish a value. It does not make padding initialized, remove alignment requirements or permit reading an arbitrary Rust object as wire bytes.

`#[qsl_macro(aggressive_transmutate)]` is a separate native-struct form: fixed named fields, no generic parameters, repr(C), optional explicit alignment, Copy/Clone and leaf validators. In that form, a fixed InlineString field is lowered to its fixed UTF-8 byte array. Packed/Rust/transparent representation options are rejected. Inspect the resulting type before relying on its layout.

## Custom binary formats

Implement `QslCustom` only when the existing numeric or template contracts do not describe the format. Declare `SIZE` and `COMP_SIZE` and provide raw/compact unsafe read and write methods. `qsl_validate` must reject malformed input bytes; `qsl_validate_values` must reject values that cannot be written correctly. The defaults reject an unknown codec in checked operations.

Validation must prove exactly the representation consumed by the unsafe decoder. A matching Rust name or object size is insufficient. The developer is responsible for initialized bytes, valid values, capacity, non-overlap and any codec-specific invariants. Core runtime contracts remain allocator-free; schema metadata and macro registration belong to the build-time lexer.

[Safety](/guides/safety/) · [Manual layout](/guides/manual-layout/)


## Manual layout: wire and object memory

Manual layout controls the **wire frame** and the **Rust object** separately. A compact message does not imply a compact object. Decide which representation must change before selecting options.

## Format contract

Declare types, integer ranges, UTF-8 byte bounds and array element counts. Every optional field must follow every required field, including after template expansion. This ordering rule is mandatory; optimization attributes are optional.

Both endpoints must agree on wire-affecting settings. Reordering declarations, changing domains, adding wire padding or selecting another codec may change bytes and schema identity. Physical-only layout choices can retain both.

## Wire packing and alignment

Raw integers use the declared little-endian width. `compact` stores exact ranks. For an inclusive range:

$$
w=\lceil\log_2(b-a+1)\rceil.
$$

Constants need zero rank bits. Ordinary compact uses LSB-first; dyn index uses MSB-first. `alignment` adds a byte boundary after a compact field; it does not align the address of a Rust object. A 3-bit field followed by a 10-bit field uses 2 packed bytes or 3 individually byte-aligned bytes. Measure the affected operation before treating that trade-off as a speedup.

## Physical order

Without physical auto, supported generated repr(C) fields retain declaration order. On the documented x86-64 target, u8/u64/u16 uses 24 object bytes; u64/u16/u8 uses 16. Each raw payload contains 11 value bytes. Reordering declarations also changes wire order.

`auto` selects physical placement while preserving logical/wire order. `auto[wire=1]` or `.auto_wire(1)` additionally authorizes versioned wire canonicalization. Generated size and offset assertions check the target layout. A useful manual starting point is decreasing field alignment while keeping all optional fields last.

$$
o_1=0,\qquad o_i=\operatorname{align}(o_{i-1}+s_{i-1},a_i).
$$

$$
\operatorname{align}(x,a)=a\lceil x/a\rceil.
$$

$$
S=\operatorname{align}(o_n+s_n,A).
$$

Here $s_i$ is field size, $a_i$ field alignment and $A$ object alignment. These formulas apply to known fixed repr(C) fields. External custom layouts cannot be inferred from names. There is no general API that assigns arbitrary physical offsets to every field.

## Padding

Schema `padding(N)` counts **bytes**; `padding_bits(N)` counts bits. Builder `.padding(N)` always takes **bits**; multiples of eight retain byte-padding behaviour. Checked input requires zero wire padding. Non-byte padding requires compact without dyn, QPS, predict_seq or state_bits. Wire padding is separate from uninitialized gaps in a Rust object; those gaps must never be transmitted.

~~~qsl
&Packet[safety,compact,alignment] {
    u8(0..7) kind;
    u16(0..1023) count;
}
&BitPacket[safety,compact] {
    u8(0..7) prefix;
    padding_bits(3) gap;
    u8(0..3) tail;
}
~~~

## Ownership and limits

Fixed byte fields use fixed arrays. Bounded UTF-8 may use StackString or InlineString; String/Vec own external storage in the application facade. STACK_LIMIT defaults to 256 B and applies to all strings and arrays, including explicit inline selections. Larger fields use supported dynamic storage instead of bypassing the limit.

`DeqBuilder.stack_limit(bytes)` controls that policy. `max_data_size(bytes)` controls frame/restoration/value limits, default 65535; it is a separate setting. A no_std core caller supplies its own buffers. Reuse can preserve String/Vec capacity, but growth can allocate.

## Native regions

`#region(aggressive_transmutate="Meta")` creates a nested fixed Rust type. Copy needs initialized valid fields, matching representation and aligned destination. Overlay additionally needs aligned immutable input with a lifetime covering the view. repr(C) alone does not prove wire compatibility: endianness, offsets, padding, ranges and UTF-8 still matter.

Start with checked copy/view APIs and inspect `NATIVE_WIRE_COMPATIBLE` or the region compatibility constants. A documented copy fallback does not authorize an unchecked overlay. Unsafe methods retain the complete caller contract.

## Review checklist

1. Required fields precede all optional fields.
2. Check wire lengths, headers, bit order and padding separately from Self size.
3. Inspect object offsets/alignment and owned allocations independently.
4. Distinguish physical auto from explicit auto-wire.
5. Test malformed lengths, spare ranks and nonzero padding with checked methods.
6. Measure only the affected read/write path with warmed buffers.

The analyzer reports planned Self size, not full function stack usage, allocator capacity or process RAM. Unknown custom sizes remain unknown.
· [Builder / DEQ](/language/deq/) · [Macros](/guides/custom-types/) · [Safety](/guides/safety/)


## I/O methods and safety

Generated methods vary by schema/mode; dyn/lifetime/QPS/native signatures differ. Generated Rust is authoritative.
Method | Contract
---|---
validate_values / validate(input) | Value budget/domain or available checked input validation
serialize_checked | Validated fallible output: `StackOut<N>` or `Vec<u8>`, selected by the schema bound.
serialize_checked_to | Caller slice, validation/capacity checks, exact written length
deserialize_checked | Checked materialization with safety
deserialize_checked_into | New MaybeUninit destination; dyn adds scratch
serialized_size | Required capacity; compressed strings may use a raw upper bound
serialize_into_unsafe | Caller guarantees values, capacity and nonoverlap
deserialize_from_raw_parts | Trusted ordinary length-aware record
deserialize_into/reuse/prepared_into | Unsafe destination/reuse contracts
comp_size/comp_write/comp_read | Compatibility COMPACT helpers, when emitted
Prepared unsafe reuse requires sufficient capacity before the call and preserves eligible String, Vec and boxed InlineString storage. It does not grow these buffers. An initialized mutable object and an uninitialized destination have separate contracts.

Serializable trusted methods remain unsafe. Callers establish valid schema data, complete input, sufficient storage, alignment and lifetimes.

Checked cursors validate extents, overflow, domains/spare codes, UTF-8, presence, padding, restoration and aggregate budgets before publication. Aggregate checks only cover proven accessible islands. Unsafe callers guarantee complete valid frames, schema match, capacity/initialization, alignment, lifetime and nonoverlap; prepared QPS adds private storage requirements.
Error | Meaning
---|---
UnexpectedEof / BufferTooSmall | Insufficient input / destination or scratch
LengthOverflow | Length/offset arithmetic overflow
InvalidWidth / InvalidSchema | Unsupported width/configuration
InvalidValue | Domain, spare code or structural value error
InvalidUtf8 / InvalidPadding | Text/padding error
SizeLimitExceeded | Frame/restoration/value budget exceeded
AllocationFailed | Fallible allocation failed
PoolExhausted | Nonblocking pool acquisition failed
Custom codec correctness also depends on its own validator/implementation. A writer error does not promise transactional restoration of already changed output; failed output must not be published as a frame.

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.


## Memory and reuse

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.

String, Vec and Box use alloc in a no_std runtime; the application supplies the allocator. A bounded InlineString is embedded when its native size fits STACK_LIMIT and boxed otherwise. The schema capacity, not the current value length, determines this choice.

Prepared unsafe reuse requires sufficient capacity before the call and preserves eligible String, Vec and boxed InlineString storage. It does not grow these buffers. An initialized mutable object and an uninitialized destination have separate contracts.

<a id="reading-into-existing-memory"></a>

## Example {#example}

```qsl
&Record[safety] { u64 id; }
```

```rust
let value = Record { id: 42 };
let mut buffer = [0u8; 64];
let used = value.serialize_checked_to(&mut buffer)?;
let mut destination = core::mem::MaybeUninit::<Record>::uninit();
let restored = Record::deserialize_checked_into(&buffer[..used], &mut destination)?;
assert_eq!(restored.id, 42);
```

[STACK_LIMIT](/language/deq/) · [String](/language/strings/) · [RT](/internals/runtime/)

## QSL_RT {#rt-slot-pool}

qsl-rt provides 1–64 slots. LocalRegions uses no atomics; StaticRegions supports multiple threads. A live lease holds its slot until Drop; a full pool immediately returns PoolExhausted.

## StackOut / Vec {#cached-output-frames}

`STACK_LIMIT = 64`: `StackOut<64>`; 65 B → `Vec<u8>`.


## QSL-LZ and QPS

`str(1..100) name|decompress=true|;` selects a distinct text codec. LZ chooses raw representation when compression is not useful; compare sizes including that codec's headers. The current short raw-copy threshold is 32 B. InlineString's shared length/mode word is 1 B for max≤127, 2 B for max≤32767, then 4 B within supported limits. FixedCompressedString has its own contract.

Checked decompression validates every literal/match run, distance, actual restored extent and capacity. A declared length alone cannot authorize writing outside the destination. Restoration is bounded independently of compressed input size.

QPS retains exact operands, including required quotient/remainder information. Q2/Q4/Q8/Q16 are supported; bare qps selects Q16. Presence prediction remains separate.
Path | Contract
---|---
Scalar payload specialization | Known numeric domains/kernels, selected by PayloadQuantPlan
ByteQps / QPB3 | General exact byte groups, 16 B envelope, original length, explicit scratch/restoration
DomainInteger uses the applicable general path rather than a separate scalar specialization. For safety-enabled non-dyn schemas the generator emits a separate whole-payload QPB3 API, including UTF-8, arrays and wide numbers: serialize_qps_to and deserialize_qps_into, with borrowed restoration for Lifetime. Ordinary serialize retains the scalar-prefix wire and QSL-LZ. The application explicitly chooses the whole-frame API; original lengths restore exactly with the frame.

QPB3 has metadata and incompressible modes; arbitrary bytes cannot always shrink losslessly. Restoration storage can exceed wire size. Some prepared scalar kernels require initialized QUANT_PADDING_BYTES after the wire prefix. These are private storage, excluded from as_slice()/network output. Ordinary exact checked APIs and prepared padded APIs have different contracts.


## Hashes and compatibility

Analysis computes strict and light schema identities. Strict includes schema/field names; light omits names while preserving wire-affecting types, bounds, attributes and deterministic field order. xxHash is the default; xxHash128 is available through USE_HASH. Neither is authentication or a mathematical guarantee against all hash collisions.

Physical layout, CPU profile, inline policy and other options that do not change input/output bytes are excluded. Auto-wire v1 is included. Keep schema configuration with fixture bytes and generator version. A hash does not validate a received frame or make different revisions interchangeable.


## Compiler architecture

## Runtime boundaries

no_std excludes std, not alloc. qsl-core always uses #![no_std]; its default alloc feature provides alloc String/Vec, SerOut and heap restoration. The application supplies the allocator; QSL does not declare a global allocator. Disable default features for caller-buffer-only operation.
Crate           | Scope
--------------- | ------------------------------------------------------------------------------------------------------------------------------------------
qsl-core        | All runtime I/O, serialization/primitive traits and macros, containers, validation, QPS and transport-independent framing; optional alloc.
qsl-rt          | no_std static-region providers implementing core FramePool/FrameStorage; a full pool returns PoolExhausted immediately.
qsl-lexer       | Source collection, DEQ, schema descriptions, analysis and optimizer plans.
qsl-interpreter | Specialized Rust source generation on the build host.
qsl-macros-impl | Rust attribute expansion
qsl             | Runtime reexports; optional compiler support. Runtime-only mode needs no std.
Build scripts and test harnesses may use std without linking it into the target runtime. Actual generated schemas run in a no-allocator executable; a separate no_std library verifies alloc-backed storage. Both core profiles and qsl-rt cross-check for wasm32-wasip2.
[Memory](/guides/manual-layout/) · [Runtime](/internals/runtime/)


## Optimizer and native layout

The compiler owns a `CompilationPlan` per prepared schema. It contains the logical declaration, canonical wire schema, both order maps, physical memory facts, I/O windows, initialized extents, operations and register candidates. `PreparedSchema.plan()` exposes it for inspection. The Rust backend consumes the prepared I/O plan. None of these compiler objects are carried in a message.

Physical `auto` changes generated field declarations, with bounded exact search for small records and stable heuristics for larger ones. It preserves wire order. External Rust definitions and opaque ABI are barriers. `repr(C)` establishes order/alignment but does not initialize padding.

Explicit `auto[wire=1]` / `.auto_wire(1)` enables a different versioned contract. Within eligible required-field islands, fixed fields precede dynamic fields and names provide the stable sort key. Conditional/opaque barriers stay pinned. Both peers need the same canonicalization. The version affects schema identity; the CPU does not choose wire order.

A memory map distinguishes initialized data, padding, descriptors, metadata and conditional slots. Pointing copies only a complete proven data span with matching wire order and native-endian facts. Generated compile-time assertions check offsets, sizes, alignment and no-Drop requirements. A record with padding is never made serializable by copying the entire Self allocation.

`transmutate` COPY reconstructs owned values through the native contract. LIFETIME overlay additionally requires complete layout equality, proper alignment, initialized bytes, valid field representations and a live source. A mismatching overlay is an error. User codecs must not claim native layout without the explicit primitive/native contract.

### CPU profiles and register candidates

`CPU_PROFILE=portable` is the default DEQ setting. `CPU_PROFILE=kaby_lake`, the `GenerationConfig.cpu_profile` field, or `DeqBuilder.cpu_profile(CpuProfile::KabyLake)` selects a build-time cost profile. This setting is excluded from wire identity.

The report includes scalar/GPR, BMI2 PEXT/PDEP, SSSE3 byte shuffle, SSE4.1 PACKUSDW/PMADDWD and AVX2 candidates. Eligibility requires the appropriate shape, initialization map and normalization. Estimates cover dependencies, uops, registers, transfers and code bytes; they are scheduling proxies, not measured cycles or PMU counters.

Only the narrow four-u32/12-bit candidate has paired complete-record evidence. Explicit `aggressive[group]` and the Kaby Lake profile request its PEXT writer; the separately selected reader stays scalar after PDEP regressed. Generated `cfg(target_feature="bmi2")` gates the instructions; builds without it retain scalar. Other shapes/profiles do not automatically select intrinsics. The application controls compilation flags and deployment CPU requirements. Feature detection is outside per-field hot code.

Primary instruction references: PEXT, PDEP, PSHUFB, PACKUSDW, PMADDWD. Saturating operations are considered only after proving that saturation cannot occur.

### Physical shifted lengths

Explicit `aggressive[shift=true]` permits `StackString&lt;N, LengthWord&gt;` / `InlineString&lt;N, LengthWord&gt;` to store len+1. Capacities up to 254 use NonZeroU8; up to 65534 use NonZeroU16; larger supported capacities use NonZeroU32. Storage selection still obeys STACK_LIMIT and string codec bounds. Zero means absence only when an actual Option niche is used; an empty present string stores one.

This is a physical representation choice. Wire length and UTF-8 bytes stay unchanged. Measurements rejected automatic shifted string storage: new-value decoding slowed down and reuse showed no confirmed gain. Plain aggressive therefore does not automatically enable shifted string lengths.

### Exact padding

`padding(N)` is byte padding with the legacy byte barrier. `padding_bits(N)` is exact zero bit padding in COMPACT. Programmatic `.padding(N)` takes bits: byte multiples preserve the old byte behavior; other values lower to BitPadding. Dyn/QPS/predict_seq/state_bits combinations are rejected. Checked decoding rejects nonzero padding.

A three-bit prefix, three padding bits and two-bit suffix occupy exactly one byte. The untrusted reader validates every chunk before publishing a value. Large padding also participates in size budgets, including optional and LZ combinations.


$$
o_i=\operatorname{align}(o_{i-1}+s_{i-1},a_i),\qquad \operatorname{align}(x,a)=a\left\lceil\frac{x}{a}\right\rceil.
$$

$$
S_{\mathrm{Self}}=\operatorname{align}(o_n+s_n,A).
$$

Here o is the field offset, s its size, a its alignment and A the object alignment. Permutation may reduce Self gaps; it does not change the rank width of an individual field.


## Dyn v1 and wire format

Dyn identifies field extents so a decoder can skip unknown tags. It does not infer the semantics of unknown types or negotiate compatibility automatically.

The 12 B header is `QDY1`, mode:u8, width:u8, reserved:u16=0, revision:u32. Service integers are little-endian. Payload bit offsets are zero-based MSB-first: offset=32,size=4 addresses the high four bits of the fifth byte.
Mode | Representation
---|---
line16/32/64 | `[tag:uW][wire_span:uW][value]`, outer span in bytes
dynamic index32 | `[offset:u16][tag:u8][size:u8]`
dynamic index64 | `[offset:u32][tag:u16][size:u16]`
strict index32 | `[offset:u16][tag:u16]`
strict index64 | `[offset:u32][tag:u32]`
Offsets begin at payload after the header. The final u32 LE trailer stores the byte distance back from the trailer to the index table. Entries are ordered by payload position, making the next offset/table boundary the complete field extent. Index16 is excluded by the accepted v1 contract.

For static values, index size is useful bits. For dynamic strings/arrays, offset points at the length header and size denotes only that header's bit width. UTF-8 length is bytes (`Яa` is 3); array length is elements. The value follows the header; its full extent comes from ordered offsets.

Checked dyn input validates header/reserved bits/revision policy, arithmetic, extents, padding and duplicate tags before publishing Self. It accepts caller scratch explicitly: u32 per index entry or u64 per line entry. It does not allocate a temporary tag Vec. Unknown fields are skipped by checked boundaries. A tag's changed meaning needs an agreed new tag/revision, not merely a matching byte count.

Compact bit fields are supported in permitted indexed forms. Unsupported combinations with QPS, PS, OT, BM, LZ, native overlay and private bitmap return a build-time diagnostic rather than silently changing the format. See .


## Runtime, RT and transport

no_std excludes std, not alloc. qsl-core always uses #![no_std]; its default alloc feature provides alloc String/Vec, SerOut and heap restoration. The application supplies the allocator; QSL does not declare a global allocator. Disable default features for caller-buffer-only operation.

no_std static-region providers implementing core FramePool/FrameStorage; a full pool returns PoolExhausted immediately.
API                             | Contract
------------------------------- | -----------------------------------------------------------
SliceOut                        | caller-owned initialized buffer
SerOut                          | alloc-backed reusable output
FrameStorage / FramePool        | core memory-provider contracts
StaticRegions / RegionLease     | qsl-rt; 1–64 exclusive slots, release on Drop
QuantizedOut / QuantizedRegions | scalar QPS output; initialized padding outside wire
RawSocket<S> / ByteTransport    | u32 LE length + payload; application transport, no std::net
InlineString keeps its codec in stack storage or Box on the heap. Large heap strings restore directly into their destination. StackString and InlineArray store their bounded capacity inside the object. Explicit aggressive[shift=true] can select a NonZero length word for strings and arrays while keeping None distinct from Some(empty).

Build scripts and test harnesses may use std without linking it into the target runtime. Actual generated schemas run in a no-allocator executable; a separate no_std library verifies alloc-backed storage. Both core profiles and qsl-rt cross-check for wasm32-wasip2.

[Memory](/guides/memory/) · [Safety](/guides/safety/)

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.


## Attributes and compatibility

Attributes and additional mechanisms are optional. Required fields must precede optional fields. The native analyzer rejects unsupported combinations before generation.
Attribute | Meaning
---|---
`safety` | Frame/value validation and checked APIs
`compact` | Rank-based bit packing; affects wire
`alignment` | Wire alignment according to the selected codec
`inline` | Explicit method inlining policy
`auto` | Physical layout of generated Self
`auto[wire=1]` | Versioned canonical wire order
`predict_seq` | Payload length → state index
`opt_table` | Tag → static reader
`optionals=bitmap` | Wire presence mask for optional fields
`aggressive[bitmap]` | Private MaybeUninit slots and native presence mask
`aggressive[group]` | Permit proven register/checked grouping
`aggressive[pointing]` | Copy a proven initialized span
`aggressive[shift=true]` | Physical len+1 string storage
`qps / qps=Q16` | Reversible operand compression, not prediction
`quant=Q2/Q4/Q8/Q16` | Compatible explicit quantum selection
`lifetime` | Borrow supported wire; input must stay alive
`transmutate` | Native contract with ABI/validity checks
`loyal_cache` | Bounded storage within its declared budget
`dyn[...]` | Explicit line/index v1 wire contract
`decompress=true` | String field using QSL-LZ
`step / values` | Exact discrete value domain[DEQ](/language/deq/) · [Limits](/reference/limits/)


## Serialization and decoding methods

Generated methods vary by schema/mode; dyn/lifetime/QPS/native signatures differ. Generated Rust is authoritative.
Method | Contract
---|---
validate_values / validate(input) | Value budget/domain or available checked input validation
serialize_checked | Validated fallible output: `StackOut<N>` or `Vec<u8>`, selected by the schema bound.
serialize_checked_to | Caller slice, validation/capacity checks, exact written length
deserialize_checked | Checked materialization with safety
deserialize_checked_into | New MaybeUninit destination; dyn adds scratch
serialized_size | Required capacity; compressed strings may use a raw upper bound
serialize_into_unsafe | Caller guarantees values, capacity and nonoverlap
deserialize_from_raw_parts | Trusted ordinary length-aware record
deserialize_into/reuse/prepared_into | Unsafe destination/reuse contracts
comp_size/comp_write/comp_read | Compatibility COMPACT helpers, when emitted
Prepared unsafe reuse requires sufficient capacity before the call and preserves eligible String, Vec and boxed InlineString storage. It does not grow these buffers. An initialized mutable object and an uninitialized destination have separate contracts.

Serializable trusted methods remain unsafe. Callers establish valid schema data, complete input, sufficient storage, alignment and lifetimes.

Checked cursors validate extents, overflow, domains/spare codes, UTF-8, presence, padding, restoration and aggregate budgets before publication. Aggregate checks only cover proven accessible islands. Unsafe callers guarantee complete valid frames, schema match, capacity/initialization, alignment, lifetime and nonoverlap; prepared QPS adds private storage requirements.
Error | Meaning
---|---
UnexpectedEof / BufferTooSmall | Insufficient input / destination or scratch
LengthOverflow | Length/offset arithmetic overflow
InvalidWidth / InvalidSchema | Unsupported width/configuration
InvalidValue | Domain, spare code or structural value error
InvalidUtf8 / InvalidPadding | Text/padding error
SizeLimitExceeded | Frame/restoration/value budget exceeded
AllocationFailed | Fallible allocation failed
PoolExhausted | Nonblocking pool acquisition failed
Custom codec correctness also depends on its own validator/implementation. A writer error does not promise transactional restoration of already changed output; failed output must not be published as a frame.

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.


## Errors and validation boundaries

Generated methods vary by schema/mode; dyn/lifetime/QPS/native signatures differ. Generated Rust is authoritative.
Method | Contract
---|---
validate_values / validate(input) | Value budget/domain or available checked input validation
serialize_checked | Validated fallible output: `StackOut<N>` or `Vec<u8>`, selected by the schema bound.
serialize_checked_to | Caller slice, validation/capacity checks, exact written length
deserialize_checked | Checked materialization with safety
deserialize_checked_into | New MaybeUninit destination; dyn adds scratch
serialized_size | Required capacity; compressed strings may use a raw upper bound
serialize_into_unsafe | Caller guarantees values, capacity and nonoverlap
deserialize_from_raw_parts | Trusted ordinary length-aware record
deserialize_into/reuse/prepared_into | Unsafe destination/reuse contracts
comp_size/comp_write/comp_read | Compatibility COMPACT helpers, when emitted
Prepared unsafe reuse requires sufficient capacity before the call and preserves eligible String, Vec and boxed InlineString storage. It does not grow these buffers. An initialized mutable object and an uninitialized destination have separate contracts.

Serializable trusted methods remain unsafe. Callers establish valid schema data, complete input, sufficient storage, alignment and lifetimes.

Checked cursors validate extents, overflow, domains/spare codes, UTF-8, presence, padding, restoration and aggregate budgets before publication. Aggregate checks only cover proven accessible islands. Unsafe callers guarantee complete valid frames, schema match, capacity/initialization, alignment, lifetime and nonoverlap; prepared QPS adds private storage requirements.
Error | Meaning
---|---
UnexpectedEof / BufferTooSmall | Insufficient input / destination or scratch
LengthOverflow | Length/offset arithmetic overflow
InvalidWidth / InvalidSchema | Unsupported width/configuration
InvalidValue | Domain, spare code or structural value error
InvalidUtf8 / InvalidPadding | Text/padding error
SizeLimitExceeded | Frame/restoration/value budget exceeded
AllocationFailed | Fallible allocation failed
PoolExhausted | Nonblocking pool acquisition failed
Custom codec correctness also depends on its own validator/implementation. A writer error does not promise transactional restoration of already changed output; failed output must not be published as a frame.


## Limits and source map

Limit | Meaning
---|---
MAX_DATA_SIZE, default 65535 | Input/restored frame, aggregate decoded-value and allocation/work guards; not exact process RSS
STACK_LIMIT, default 256 B | Strict inline string/array policy; impossible explicit storage is diagnosed
64 compiler-assigned field tags | Optional dispatch/bitmap domain
32 text array nesting levels | Parser complexity bound
Explicit unsafe methods | Caller must establish all data/memory/CPU/lifetime conditions
Unsupported option combinations | Build error, not an implicit new wire contract
LLVM backend / QBProto handshake | Future work
Exact serializer sizes are checked before allocation/writing. LZ capacity is an upper bound, so value/padding budgets and actual output size are checked separately. Native allocation guards use the Rust element size and bound zero-sized work; a tiny wire element cannot authorize a huge Vec allocation. These are data/storage guards, not exact allocator overhead accounting.

Sources: [schema/types](/source/qsl-lexer/src/types.rs), [domains](/source/qsl-core/src/domain.rs), [analysis/IR](/source/qsl-lexer/src/analysis/compilation.rs), [layout](/source/qsl-lexer/src/layout.rs), [register planner](/source/qsl-lexer/src/optimizer/registers.rs), [kernels](/source/qsl-core/src/optimizer/kernels.rs), [parser](/source/qsl-lexer/src/parser.rs), [Builder](/source/qsl-lexer/src/builder.rs), [Rust backend](/source/qsl-interpreter/src/interpreter.rs), [checked codegen](/source/qsl-interpreter/src/codegen_checked.rs), [validation](/source/qsl-core/src/validation.rs), [runtime buffers](/source/qsl-core/src/serialize.rs), [RT](/source/qsl-rt/src/lib.rs).

 records the outstanding end-to-end matrix and external instrumentation. This document describes the working tree and is not a claim that every specification acceptance check is complete or published.

### Generated-profile regression: independent read/write selection

The final four-u32/12-bit comparison uses the same whole-crate +bmi2 build for both sides. Reverse-pass means: write 3.063→2.575 ns, read 2.669→3.042 ns. Both orders agree, with disjoint 99% CIs. PEXT writing is accepted for explicit KabyLake + aggressive[group]; PDEP reading is rejected and the scalar generator is retained. The earlier standalone read improvement did not survive integration. The report retains the rejected reader under its original source snapshot. It does not describe the currently selected reader. This is trusted full-record latency, excluding validation and allocation; M=16 B and wire=6 B. .


## Terminology

## Wire {#wire}

The bytes exchanged by peers, distinct from Rust object layout.

## Self {#self}

The generated Rust struct and the native positions of its fields.

## Payload {#payload}

Field-value data; headers, tags and alignment are counted separately.

## Capacity {#capacity}

Buffer capacity. Current initialized length may be smaller.

## Checked {#checked}

An API that validates input or values and returns Result on failure.

## Unsafe {#unsafe}

The caller must establish the documented data and memory preconditions.

## Rank {#rank}

The ordinal of a valid value, allowing a domain to use fewer bits.

## Padding {#padding}

Alignment gaps. Self padding is not initialized payload data.

## Optional {#optional}

A field may be absent. Absence is not default substitution or an empty value.

## Scratch {#scratch}

Caller-provided temporary workspace, excluded from the transmitted frame.

## Invariant {#invariant}

A required condition for a schema/method, such as a valid length.

## Qps {#qps}

Operand compression with exact restoration; it does not predict field presence.


Project: https://github.com/zadr0/QSL
