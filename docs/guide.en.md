> **QSL 0.2.0:** [migration and current contracts](migration-0.2.0.en.md). BM changes format; ordinary non-BM tags are preserved. Windows-specific runtime modules and the native Cargo feature are removed. Historical measurement chapters retain their captured versions.


Trusted Serializable::serialize / BorrowSerializable::serialize trait methods are now unsafe. Callers must establish their preconditions. Generated RT/QPS serialize() methods remain safe Result APIs with value validation; use serialize_checked for general safe output.

# QSL. Complete project guide

[Русский](guide.ru.md) · [Architecture](architecture.en.md) · [API](API.en.md) · [Feature measurements](features.en.md)

## Checked input and identity

Checked deserialization is generated only for [safety]. `!config MAX_DATA_SIZE = 65535;` bounds the frame, QPS restoration and aggregate values. Strict/light hashes are computed during analysis; `!config USE_HASH = xxHash;` selects XXH3-64 and xxHash128 selects 128 bits. Both attributes are imported from qsl; implementation lives in qsl/macros. [Full safety contract](safety.en.md) · [Schema identity](schema-hashes.en.md).

## Purpose and boundaries

QSL's original task is exchanging data described in advance. Types, ranges, field order and ownership are known before execution. The generator turns these conditions into specialized Rust. A dynamic schema interpreter does not run on every encode or decode.

QSL is a codec constructor shaped by the project's conditions. Developers establish invariants, choose a format and prepare storage. The price is explicit responsibility for compatibility, capacities, lifetimes and validation. Performance comes from eliminating repeated work under those established conditions.

Additional mechanisms and attributes are opt-in. Field order is always mandatory: required fields first, optional fields last. Changing order, ranges, codec, quantum or configuration may change wire format. Both peers must agree on the representation.

Schema QPS packs scalar numeric fields only; strings, arrays and lengths are not quantized. The scalar specialization packs statically bounded numeric ranks. Length prediction belongs to predict_seq; opt_table selects a reader by tag. Quantization itself does not determine which fields are present.

## Package installation

```toml
[dependencies]
qsl = "=0.2.0"

[build-dependencies]
qsl = "=0.2.0"
```

[Four-file quick start](quickstart.en.md). Both dependencies are required: build.rs generates the codec and the application uses it.

## Minimal integration

Use the same QSL version in application and build dependencies. build.rs explicitly lists QSL, DEQ and Rust-template inputs. build_qsl_files / build_qsl_files_with_rust writes qsl_gen.rs under OUT_DIR; the application includes that file.

```rust
let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
qsl::build_qsl_files(&["schema.qsl"], &["schema.deq"], &out)?;
// In an application module:
include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
```

```qsl
&Message[safety,opt_table] {
    u32 id;
    str(0..4096) text|binary_opt=guard|;
    u16? code;
}
```

Checked deserialization requires [safety]. opt_table selects optional readers by tag; predict_seq,opt_table enables the length graph. These attributes do not make an arbitrary pointer safe or remove a custom codec's obligations.

## Build pipeline

1. DEQ parses typed constants and optional generation settings.
2. QSL parsing evaluates expressions, validates declarations and constructs schemas.
3. Rust templates expand into fields; cycles, names and final field order are checked.
4. Analysis determines domains, bit widths, event boundaries and permitted registers.
5. The interpreter emits structures, constants, methods and static tables.
6. rustc/LLVM selects instructions and the final native object layout.

Zed uses the same native parser when a compatible qsl-analyze is available. The JS fallback is labelled advisory. Editor discovery supports feedback; build.rs defines the actual build inputs.

## Schema language

A declaration is &Name[attributes] { type field|modifiers|; }. A question mark makes a field optional. Tags are compiler-assigned in declaration order and cannot be overridden. Duplicate names, invalid bounds and required fields after optionals are rejected.

A..B bounds are inclusive. Constant expressions support +, -, *, /, %, parentheses, named values and sizeof for supported integer primitives. Overflow and division by zero are build errors. DEQ sizeof describes the named integer, not the future layout of an arbitrary Rust struct.

The text parser recognizes u8/u16/u32/u64 and i16/i32/i64. The internal model can represent i8, but direct i8 is not in this parser's primitive list. An unknown name can be a custom type requiring QslCustom; it does not automatically become a built-in number.

## Types and generated Rust

| Schema description | Rust representation | Main constraint |
|---|---|---|
| u8/u16/u32/u64, i16/i32/i64 | Corresponding primitive | Value must fit its domain |
| Integer T(A..B) | Same native primitive | compact/QPS may change wire width |
| flags(N) | u8/u16/u32/u64 selected by N | N=1..64; normal encoding uses storage width |
| str / string / String | String | Variable length; decoding may allocate |
| str(N), including A..A | [u8; N] | Exact UTF-8 byte count |
| str(A..B) | String or selected wrapper | Inclusive UTF-8 byte bounds |
| array<T> / vec<T> | Vec<T> | Text parser limits elements to primitives |
| array<T>(N) | Vec<T> | Fixed wire element count; native object is still Vec |
| array<T>(A..B) | Vec<T> | Inclusive element-count range |
| Byte array with lifetime | &'a [u8] | Source must outlive the value |
| padding(N) | () | N zero wire bytes |
| Custom | Declared Rust type | QslCustom supplies width and correctness |
| T? | Option<T> | None differs from Some, including constants |

Rust object size, encoded frame size and buffer capacity are separate quantities. repr(Rust) can reorder fields and add padding; rustc determines actual size_of and offset_of. Analysis does not invent a native size.

## Default representation

Normal integers use little-endian native storage width. str stores a u32 length followed by UTF-8 bytes; fixed strings omit length. Dynamic arrays store a count and elements, while fixed arrays have a schema-known count. Physical wire fields follow schema order regardless of the native object's layout.

One optional field uses presence metadata. With multiple optionals, default encoding records the present count and their tags. tag_bits=ceil(log2(max_tag)); count_bits=ceil(log2(optional_count+1)). Normal byte encoding rounds metadata to whole bytes. Header length can depend on the presence state.

The trusted pointer decoder assumes valid tags and sufficient input. Checked APIs validate the frame separately. The format does not transmit a universal type description with every request.

## compact and alignment

compact represents the rank x-min using B=bit_length(max-min). A single allowed value uses B=0. Fields may cross byte and machine-word boundaries; analysis chooses registers and emits specialized operations.

For example, ranges 0..7, 0..31 and 0..1023 use 3+5+10=18 bits, or 3 encoded bytes. Native Rust types and object sizes remain unchanged.

With compact, alignment completes each field at a byte boundary. This is wire alignment, not cache-line alignment of storage. Extra bits can simplify access while increasing frame size; paired measurements determine the tradeoff.

compact conflicts with predictive sequential layouts. Active scalar QPS does not arbitrarily combine with compact, alignment, bitmap or legacy directories; incompatible selections are rejected by analysis.

Default values initialize absent fields; they do not synthesize presence or bypass validation.

## Default: count/tags and match

Required fields precede optional fields. Without prediction or BM the frame preserves its existing packed optional-count/tag header, then required values and present optional values. Default dispatch uses a generated match. One optional field keeps a one-bit presence header. Tags remain incremental schema tags; sparse tags produced by region lowering are mapped explicitly.

## BM: optional-only set-bit traversal

Only optional fields occupy bits. Counts 0 / 1–8 / 9–16 / 17–32 / 33–64 / 65–128 use 0 / 1 / 2 / 4 / 8 / 16 header bytes. The mask is little endian; marker 1 occupies its MSB. Read `marker=WIDTH-mask.trailing_zeros()`, clear `mask &= mask-1`, dispatch marker and repeat until zero. Required values retain declaration order; present optional values are serialized and read in descending marker order.

For `&P[safety,optionals=bitmap,opt_table]{u8 id;u16? a;u8? b;}`, id=9, a=0x1234 and b=7 encode as `C0 09 07 34 12`. BM v2 differs from the former u32/all-fields mask. Both peers must migrate together; no automatic format detection is added.

Before traversal, Safety checks the largest marker once if spare bits exist. Empty masks skip it. A full-width mask has no spare bits. After a successful check every remaining dense OT index is bounded. Field data and end-of-frame checks still run through the checked cursor.

## OT: choose an optional reader

Standalone opt_table preserves default bytes and hashes. In ordinary tagged/BM paths its dense static table contains one reader per optional marker; required fields do not create gaps. Absolute schema tags are mapped to dense ordinals. With BM the marker directly selects that ordinal. A per-field cursor reader is shared with default match dispatch; checked and unchecked contracts retain separate readers.

There is no per-frame table allocation. A non-borrowed raw entry is a function pointer (one pointer on x86-64); checked dispatch has its own table. An indirect call is not guaranteed to outperform a match. Compact ordinary layouts and BM can use OT. The former optional_count*tag_bits<=64 restriction is removed from ordinary streaming headers.

## PS: a length-derived index

predict_seq derives presence from payload length, not tag dispatch. A standalone single optional uses EOF; multiple fields require unambiguous supported fixed-size subset sums. Invalid/ambiguous sums and a zero-size optional codec are rejected where length cannot distinguish presence. The trusted API needs input length: a pointer-only deserialize_unsafe cannot infer it.

## PS + OT

The combined mode preserves its existing length graph and wire contract. `I(L)=sum((L>c_j) as usize)` selects an event; forced fields use existing OT readers, ambiguous fields require the mode's presence metadata. The reserved metadata bytes are not silently removed. General graph limits remain 12 fields / 65535 payload bytes; equal fixed widths support up to 64 fields. Those are distinct from BM limits.

PS is inactive when BM is selected and cannot reorder its traversal. QPS does not predict: it is reversible operand-based scalar packing. Legacy offset directories and state_bits are separate formats with their own compatibility checks. Scalar QPS uses bit offsets rather than forcing packed fields through a byte reader table.

## Strings, UTF-8 and ownership

| Selection | Rust representation | Decode behavior |
|---|---|---|
| Ordinary string | String | Creates and fills owned storage |
| lifetime | &'a str | Borrows input bytes |
| binary_opt=true | &'a [u8] | Borrows UTF-8 bytes |
| binary_opt=view | Utf8Guard<'a> | Borrowed string-like view |
| binary_opt=guard | String / StackString | COPY compatibility syntax; no separate Guard |
| binary_opt=inline | InlineString<MAX> | Copies into inline capacity |
| decompress=true | InlineString<MAX>, or [u8; N] for fixed strings | Restores raw/LZ into final storage |

A Unicode scalar can occupy multiple UTF-8 bytes. Measurements therefore distinguish 1024 characters from their actual byte count. Schema bounds and P use bytes.

CopyGuard does not borrow its input frame. Initial storage can allocate; sufficient prepared capacity can be reused. InlineString avoids a separate string allocation, but a large MAX increases the native object and working set even for short values.

Borrowed modes need the appropriate lifetime and a live immutable source. Fixed strings remain [u8; N] regardless of binary_opt. Decoded bytes cannot be borrowed from compressed source storage; restoration needs its own region.

## String LZ

decompress=true selects the specialized bounded string codec. The schema and data determine raw/packed framing and widths. Compressed framing is selected when its complete representation saves space; an unprofitable result uses raw instead.

MAX is limited to 65535 restored bytes. Dynamic strings use bounded headers and appropriately sized back references; fixed strings have separate framing. This mechanism is separate from scalar specialization and general byte QPS.

Time depends on length, repetition and entropy. One repetitive string does not characterize random UTF-8. The fresh survey includes both noise and repetition. The project's 25 ns target is a goal for the specified string, not a demonstrated universal guarantee.

## QPS: two exact operand paths

QPS does not predict fields. It transforms values into reversible numeric operands. No lossless codec can shorten every possible input; grouping supplies more context but does not reduce the full domain's cardinality.

Four arbitrary u8 values contain 32 bits. If all combinations are valid, there is no uniquely reversible encoding shorter than 32 bits for every combination. Savings come from declared bounds or actual redundancy. The extension warns about a single u8 without grouping benefits.

The [qps] / qps=Q2/Q4/Q8/Q16 scalar specialization computes statically bounded ranks. Both quotient and remainder of r=x-min are retained, and fixed masks/shifts recover the value. Default: Q16. quant=Qn is a compatible spelling; both attributes together are rejected. Constant values can consume no value bits, while optional presence remains separate.

General ByteQps<Q> accepts arbitrary serialized bytes, including numbers, UTF-8, arrays and custom encodings. Q is 2/4/8/16 byte lanes per group. It does not interpret fields or replace schema validation.

## Legacy ByteQps: format and mathematics

The explicit 16 B QPB3 header contains magic 4 B, quantum 1 B, mode 1 B, shared minimum 1 B, reserved 1 B and original u64 little-endian length 8 B. Length is not quantized. Raw mode carries the header and original bytes; its 16 B framing cost is visible.

For input bytes x_i, choose base a=min(x_i) and ranks r_i=x_i-a. Each group selects B=bit_length(OR r_i), d=2^B and operand W=sum(r_i*d^i). A group stores one byte B plus ceil(n*B/8) bytes of W. Restore r_i=(W>>(i*B))&(d-1), x_i=a+r_i. With B=0, the base alone restores every byte in the group.

Packed mode is selected only when 16+sum(1+ceil(n*B/8)) is smaller than the original input length. This comparison includes the entire header. Small or incompressible inputs use a raw frame, which can exceed input size by the explicit framing cost; the metrics show it.

Complexity is O(n), with O(1) auxiliary space when callers provide buffers. Finding the base and widths takes input passes. Q is fixed by a type parameter; width selects a specialized arithmetic kernel from a table. Kernels have no per-byte if. Safe wrappers retain format/bounds checks; group loops and the raw/packed decision remain. The whole codec is not claimed to have zero branches.

Outer ByteQps restores the exact inner frame, including string and array length prefixes. Inner fields are read after restoration. The scalar format and byte envelope are distinct; one must not silently replace the other.

Full groups use machine words instead of many tiny memcpy operations. On x86-64 with BMI2, implementation selection outside group kernels enables pext/pdep; other machines retain a portable path. Rust u128 denotes an operand type, not a promise of one 128-bit integer register. Partial groups use local tail storage; input slices need no hidden padding. QPB3 bytes are unchanged by this optimization.

After optimization, Q16 on 1024 B of values 0..15 produces 592 B, taking 440.05 ns to encode and 258.01 ns to decode. This general codec does not meet the 20–30% overhead budget or the 25 ns target. It remains an explicitly selected compression path; scalar specialization and ordinary decoding are assessed separately.

## Legacy ByteQps: region APIs

```rust
use qsl::qps::ByteQps;
let frame = ByteQps::<16>::encode_to(input_bytes, encoded_region)?;
let restored = frame.decode_to(restoration_region)?;
assert_eq!(restored, input_bytes);
```

Safety schemas implement CheckedRecord or CheckedBorrowedRecord. These unsafe traits connect a validator to the exact trusted decoder. encode_record_to checks values, writes the inner frame to scratch and encodes into a separate output. decode_record_into restores the inner frame, validates it and writes the object into MaybeUninit.

[qps] generates serialize_qps_to(scratch,out) and deserialize_qps_into(input,restoration,out). For borrowed schemas, deserialize_qps returns a value whose lifetime follows restoration storage. The type system prevents releasing that region while the value remains in use.

The general path is explicit through these APIs or ByteQps; existing Serializable formats are not automatically redefined. Numeric QUANT_ACTIVE indicates the scalar specialization, not the existence of all QPS APIs. An active scalar schema's convenient serialize() and outer serialize_qps_to have different output types and framing contracts.

Three logical regions exist: the source native object, encoded frame and scratch/restoration. Scratch and restoration may reuse storage in different phases. encode_to/decode_to do not allocate these buffers internally. ByteFrame borrows output. A small schema may not repay 16 B; the codec never assumes a smaller destination blindly.

## Specialized prepared methods

Numeric QuantizedFrame is distinct from legacy ByteFrame. Generated prepared reads now have QUANT_PADDING_BYTES = 0 and physically read exact available byte widths.

serialize_quantized_to checks values and capacity before its fast kernel. quant_write_prepared / quant_read_padded are unsafe: callers guarantee domains, padding, disjoint regions and aligned destinations. QuantizedFrame::decode_into restores into a separate larger native region.

Ordinary offsets cannot address packed bytes. With active scalar QPS, opt_table may provide Boolean bit-prefix offset functions. These use presence metadata rather than inferring contents from packed byte length.

## Safety and decoding methods

| Method | Work included | Preconditions |
|---|---|---|
| validate | Complete frame checks | Safe input; custom codecs need a validator |
| deserialize_checked | Checks and constructs a result | Matching schema and format |
| deserialize_checked_into | Checks and writes into MaybeUninit | Supported owned and borrowed schemas |
| deserialize_into / raw pointers | Trusted fast path | Valid frame, sufficient regions, aligned destination |
| deserialize_prepared_into | Updates a live object | Every reused field has sufficient capacity |
| deserialize_reuse | Retains supported buffers | Reuse behavior depends on the generated layout |

Writing a new owned String over a live object through the raw uninitialized path without dropping its old value leaks memory. Use prepared/reuse or correctly drop before replacement. REUSES_STORAGE identifies a supported generated shape; it does not create missing capacity.

Each schema cursor checks lengths, domains, metadata, UTF-8, padding, LZ and complete frame ending. It does not add implicit checks to every trusted call. Invalid built-in input returns QslError before unsafe reading.

Structural validation is not schema identification. Schemas with identical admissible wire forms can accept the same bytes. Type discrimination requires an agreed discriminator, version or handshake. QBProto and identifier R remain planned.

## Portable buffers and sockets

SerOut owns an aligned global-allocator allocation with capacity and initialized length. new initializes its byte range; with_capacity_aligned starts with length zero. Publish initialized bytes only. Ordinary Serializable/BorrowSerializable serialize always returns Vec<u8>; serialize_to takes a mutable byte slice and returns the written length. Pointer-based output supports caller-prepared storage under its unsafe contract.

The native Cargo feature, Windows NativeRegion/SharedRegion/NativeSocket/NativeListener and windows-sys were removed in 0.2.0. Portable RawSocket uses standard TCP framing. IPC/shared-memory allocation, TLS, routing and protocol negotiation belong to the application. Native-layout COPY/LIFETIME is a separate representation mechanism and remains available.

## QSL_RT

```deq
!config QSL_RT = true;
!config QSL_RT_DEFAULT = 16;
```

RT defaults off. DEFAULT counts simultaneously occupied slots per schema, not bytes or quantum. Valid counts are 1..64. Static regions require a finite known upper bound; payload including padding is limited to 64 MiB per schema.

A lease occupies one slot. serialize() waits on exhaustion; try_acquire remains nonblocking; Drop releases the slot. CAS acquisition and atomic release have their own cost. RT does not imply that source String/Vec fields never allocate.

deserialize_into remains available. General outer ByteQps receives scratch/output/restoration separately; the inner message's RT pool does not automatically become two outer QPS regions.

## Sockets

RawSocket uses Rust standard TCP with QSL binary framing. Windows-specific NativeSocket/NativeListener were removed in 0.2.0. Applications can supply their own transport around prepared byte buffers.

max_frame constrains accepted size before frame allocation/reading. Framing errors or interrupted operations can terminate connection usability; the application must not reinterpret an arbitrary stream middle as a fresh frame. Pointer receive requires sufficient writable storage.

TCP is a stream: one read need not return one message. QSL transport does not provide TLS, HTTP semantics, schema negotiation or authorization. These are application layers, and their costs are not part of codec-only timings.

## Rust templates and custom codecs

```rust
#[qsl::qsl_macro]
pub struct Meta { pub id: u32, pub tick: u64 }
```

&Packet { Meta meta; u16 code; } generates meta_id, meta_tick and code. The original Rust type can remain nested; its fields are copied into the generated structure. Nested and repeated templates are supported. qsl_macro/schem_macro mark Rust declarations; QSL has no [macro] attribute.

Limits: depth 64, no cycles or name collisions. Ordinary modes retain max_tag <=64; BM supports up to 128 optional fields and max_tag <=65535. Optional/default on a group reference is ambiguous; specify properties on members. Generic/cfg rules and computed capacities are not guessed. Bounded editor discovery is not full Rust import resolution.

QslCustom provides SIZE, COMP_SIZE and raw read/write methods. Checked decoding needs qsl_validate, whose default rejects the representation. User unsafe code is responsible for initialization, exact widths and valid Rust values. Unknown custom width limits RT and static size/offset estimates.

## DEQ, inline and build features

DEQ supports !define NAME: type = expression; and the documented !config settings. Duplicate constants/settings and unknown directives are errors. File paths stay in build.rs; typed values must fit their declared integer type.

inline, inline=hint, inline=always and inline=never explicitly control generated methods. Without an attribute, the compiler decides. This is independent of InlineString and prediction. always can increase code and instruction-cache pressure; never may preserve a call. Actual effects depend on the build.

rust-macros enables explicit marked Rust-source imports. diagnostics supplies diagnostic checked-cursor tools; it is not required for generated checked APIs. No native/Windows Cargo feature remains.

## Measurement method and P

The primary index is P=M/T: M is original useful bytes and T is nanoseconds per operation. B/ns is numerically equal to decimal GB/s. This measures useful processing throughput, not compression ratio.

Encoded size is shown separately as minimum, maximum and mean. For variable optional states, M is the mean useful byte count under the declared distribution. Framing, native padding and capacity are not counted as useful bytes. Frame size and throughput expose different memory/network tradeoffs; P alone cannot choose every option.

For a borrowed result, P measures creation of a view over the complete payload. Physical reads may be much smaller than M. This P therefore does not measure memory-copy bandwidth.

For positive T in [T_low,T_high], P lies in [M/T_high,M/T_low]. Compare only identical tasks. Validation without materialization, a borrowed view and a new String have different output contracts even when their M is identical.

The historical source survey (not a 0.2.0 run) has 103 distinct cases (95 main and 8 UTF-8/whole-record), with 125 observations after repeating 22 general-QPS cases. The other 81 cases retain their initial snapshot. Criterion, 15 s warm-up per case, 10 s measurement, 200 samples, 99% CI and an 8 GiB limit. The measuring thread is pinned to a permitted CPU with above-normal priority. Preparation is outside timing unless allocation/lease is explicitly named. Sources and executable are captured with SHA-256 before execution.

A waveform displays T_i=sample_time_i/iterations_i and P_i=M/T_i for each sample. It is not individual-call latency or network p99. Warm-up is absent from these points. Axes/units are labelled and complete sample/estimate data remain available.

Hardware: Windows 11 x86-64, i5-7400, RAM 24 GiB / 2133 MHz. Earlier profiles with different shapes, working sets or affinity policy remain separate snapshots; their numbers do not form a universal ranking.

## Modules and internal algorithms

| Module | Responsibility |
|---|---|
| parser / parser_util / deq | Syntax, expressions, settings and explicit inputs |
| types / compat | Internal/compatible schema models, domains and Rust types |
| rust_macro / schema_macro | Syn parsing and recursive field expansion |
| analysis / bit_io | Register planning, bit cursors and static boundaries |
| interpreter / codegen_* | Structures, methods, checks and static tables |
| serialize / io | Trusted API contracts, SerOut and byte cursors |
| validation / io_checked | Frame checks and explicit checked operations |
| strings | UTF-8 wrappers, bounded headers and LZ |
| length_graph / static_optional | Critical points, events, presence and offsets |
| optional / static_index | Legacy directories and state_bits/D compatibility |
| payload_quant / qps | Scalar operand specialization |
| qps_bytes | General byte groups, framing and restoration |
| rt | Static slots, CAS acquisition and leases |
| socket | Portable standard TCP framing; application controls external transport |

The API index lists public names, signatures, source paths and lines for this snapshot. Generator string literals are not misclassified as generator methods. Private algorithms are described by module; obtain your actual qsl_gen.rs through the build or editor expansion action.

## Limits, proofs and development

Lean4 checks arithmetic obligations for ranks, quotient/remainder pairs, non-overlapping radix groups and predict_seq boundary indices. Prediction theorems are not a property of QPS. The Lean file does not verify the entire Rust implementation, LLVM, allocator or IPC.

General-QPS tests include every byte value, short tails, UTF-8, high bases, repetition, mutated frames, invalid widths/lengths and restoration of multiple types. These tests do not make arbitrary custom unsafe code sound.

Planned QBProto begins with agreement on data limits and schema identity. A unique identifier cannot be justified merely by compressing products of S×F without a precise encoding of the set, order and configuration. R needs a separate protocol definition and collision analysis; no finished implementation is claimed.

QSL develops as an internal solution shaped by architecture and usage conditions. Specialized invariants exchange generality for flatter generated code. Correct representations, reproducibility and measured tradeoffs matter more than a universal speed promise.

## Descriptor model

QslType is the internal generation model; ParserQslType adapts the earlier AST. This does not mean every variant has its own text syntax.

| QslType | Rust | Descriptor fields |
|---|---|---|
| None | () | No encoded value |
| Byte | u8 | No tuple fields |
| Integer | integer primitive | bits: storage width; signed: signedness; min: legacy rank base |
| RangedInteger | integer primitive | bits/signed: storage; min/max: inclusive domain |
| Flags | u8/u16/u32/u64 | Flags.0: usize, declared bit count |
| FixedString | [u8; N] | FixedString.0: usize, exact UTF-8 byte count |
| String | String or borrowed &str | String.0: usize, legacy minimum; text str uses zero |
| RangedString | String or borrowed &str | min/max: UTF-8 byte bounds |
| CompressedString | InlineString<MAX> | min/max: restored UTF-8 byte bounds |
| BinaryString | &'a [u8] | min/max: UTF-8 byte bounds, optional upper bound |
| GuardString | Utf8Guard<'a> | min/max: borrowed UTF-8 byte bounds |
| CopyGuardString | Utf8CopyGuard | min/max: owned byte-buffer bounds |
| InlineRawString | InlineString<MAX> | min/max: raw inline byte bounds |
| FixedCompressedString | [u8; N] | FixedCompressedString.0: usize, restored fixed byte count |
| Array | Vec<T> or borrowed byte slice | Array.0: Box<QslType>, element; Array.1: usize, minimum count |
| RangedArray | Vec<T> or borrowed byte slice | element: Box<QslType>; min/max: element counts |
| FixedArray | Vec<T> or borrowed byte slice | FixedArray.0: Box<QslType>, element; FixedArray.1: usize, exact count |
| Padding | () | Padding.0: usize, zero-byte count |
| Custom | declared Rust type | rust_type: String; fixed_wire_size: Option<usize> |

rust_type(borrowed) determines the generated type. fixed_wire_size reports known wire size, not Rust size_of. rank_width/rank_base provide width and base for the selected codec. Unknown width is not replaced by zero.

## Errors

| QslError | Meaning |
|---|---|
| UnexpectedEof | Insufficient input |
| BufferTooSmall | Supplied region is too small |
| LengthOverflow | Length arithmetic overflow |
| InvalidWidth | Unsupported width or quantum |
| InvalidValue | Value/metadata outside the allowed domain |
| InvalidUtf8 | Invalid UTF-8 |
| InvalidSchema | Invalid or incompatible schema |
| InvalidPadding | Invalid nonzero padding bits/bytes |
| PoolExhausted | All static slots are occupied |

File and socket operations also use std::io::Error. Schema validation does not authenticate a sender.

## Executable examples

[Seven programs and commands](../examples/README.md) · [39 historical measurements](examples.en.md) · [Charts and waveforms](../examples/performance.html). Timings retain their explicit source snapshots; historical tables are not replaced.

Ordinary tagged/BM OT now supports compact and streaming tag headers; see the current optional chapter.

## Native region types and scalar polymorphism

Named regions now produce actual nested types, never flattened parent fields. COPY owns them; LIFETIME checks exact native/wire layout, endianness, alignment and values before creating a reference. Explicit scalar wrapper macros validate layout/domain promises without automatically reinterpreting arbitrary custom codecs. Compact custom values charge original SIZE to the restored-data budget while consuming COMP_SIZE; array elements are validated before checked writing. [Complete contracts and runnable examples](migration-0.2.0.en.md).
