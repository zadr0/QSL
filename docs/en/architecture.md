# QSL — Architecture and implementation reference

> **Scope:** grounded in the supplied `QSL.rar`, `qsl-demo.rar`, `qsl-bench(1).rar` and benchmark exports. **Implemented**, **Constraint**, and **Planned** describe the inspected sources, not the broad design roadmap. QSL does not automatically make arbitrary network input safe.

## 1. QSL as a schema-specific DSL

QSL is a domain-specific language for **binary serialization schema generation**. A `.qsl` file declares field types, order, ranges, optionality and layout. `.deq` provides typed compile-time numeric constants. `QslInterpreter` converts the normalized schema to Rust source, with offsets, masks and rank widths determined before runtime. The resulting Rust types implement serialization traits; there is no generic schema interpretation on every record.

The goal is to minimize work for a **particular schema and workload**, not to claim universal superiority. The result depends on memory layout, ownership, string size, allocation, CPU caches, compiler code generation, concurrency and transport. Moving arithmetic to code generation cannot remove the physical cost of copying data or waking threads.

Actual examples from `qsl-demo/schemas/fixed.qsl`:

```qsl
&Fixed {
    u32 id |default=1|;
    u64 created_at;
    u16 checksum;
}
&Dense[compact] {
    u8(0..7) flags;
    u8(0..31) rank;
    u16(0..1023) code;
}
```

`Fixed` has a **14-byte wire payload** with literal offsets `0,4,12`; `Dense` takes **18 bits = 3 bytes**. Rust structure padding does not become wire padding. Numeric `default` affects initialization, not a separate serialized field.

## 2. Exact compilation pipeline

1. `build.rs` calls `qsl::build_qsl_files(&[QSL paths], &[DEQ paths], OUT_DIR)`. File paths and output selection are in the build script, not DEQ.
2. `parse_deq` resolves typed `!define` numeric constants; `parse_qsl_with_constants` parses type declarations, ranges, flags, defaults, optionals and directives.
3. The normalized `QslSchema` holds ordered `QslField` entries with bit widths, `QslType`, `optional`, generator-assigned `tag`, and optional numeric `default`.
4. `QslSchema::validate` rejects incompatible flag combinations, invalid widths, unsupported borrows and contradictory constraints.
5. `QslInterpreter` selects fixed flat, compact flat, optional flat, sequential, or a specialized generic path; it emits `qsl_gen.rs` and the generated Rust structs and methods.
6. `include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"))` imports the result, which Rust compiles along with the application.
7. Runtime invokes `serialized_size`, `serialize_into_unsafe` or the matching `deserialize*` method, using the layout determined by the schema.

**Separation of concerns:** QSL DSL describes layout; `QslInterpreter` generates machine-compilable Rust; pointer readers/writers execute the record operation. The build-time parser is not run for every socket message.

## 3. Mathematical foundations

### 3.1 Integer range ranking

For the inclusive integer range `[MIN, MAX]`:

\[N = MAX - MIN + 1,\qquad B = \lceil \log_2 N \rceil,\qquad r = x - MIN\]

`B` is the bit length of `MAX-MIN` and can be computed with integer operations. `10..17` needs 3 bits, `-5..10` needs 4, and `5..5` needs 0 payload bits. Compact mode writes an **unsigned rank** `r` instead of the original integer representation. Runtime unchecked writers rely on the caller to satisfy `MIN ≤ x ≤ MAX`.

### 3.2 Offsets and alignment

Bit-position recurrence: `o_(i+1)=o_i+B_i`. Wire length is `ceil(total_bits/8)`. The generator/runtime use familiar constant arithmetic such as `(bits+7)>>3` and `(bits+7)&!7`. `alignment` pads **after each compact field** to the next byte boundary. A source rewrite of `div_ceil` is not automatically a speedup; LLVM often strength-reduces constant division.

### 3.3 Optional-field metadata

Without bitmap, one optional uses a presence bit; multiple optionals use a count and generator-assigned tags. Tags are one-based in declaration order and stored as `tag-1` in `ceil(log2(max_tag))` bits. With `max_tag=8`, tag metadata takes 3 bits. `optionals=bitmap` uses `ceil(total_fields/32)` little-endian `u32` words, with the first field at the most-significant bit.

### 3.4 Offset table cost

For `opt_table` with `K≥2` optionals, directory bytes are `1+4K`: `count:u8`, followed by `K × u32 LE` offsets and optional payload. Zero means absent; positive entries store `payload_offset+1`. Indexed lookup is `O(1)`; decoding all `K` fields is `O(K)`. A table trades additional wire bytes for easier indexed access.

## 4. Schema attributes — implementation and limitations

### `compact` — bit-ranked representation

**Implemented.** `&Dense[compact]` makes constrained integers occupy their computed rank width rather than the full Rust storage width. Fixed compact paths can be emitted with literal masks/shifts and fully unrolled 64-bit word operations. Compact-generated types expose `comp_size()`, `comp_write()` and `unsafe comp_read()` alongside the serialization trait methods.

**Constraints:** incompatible with the sequential optional formats `predict_seq` and `opt_table`. Variable-length fields still require length and payload processing; `compact` does not transparently compress mixed numeric-and-string objects. Values outside declared ranks violate the unchecked encoder contract. Additional bit operations may offset size benefits on particular hardware.

**Exact example:** `u8(0..7)` = 3 bits, `u8(0..31)` = 5 bits, `u16(0..1023)` = 10 bits, total 18 bits and three wire bytes.

### `alignment` — field-end byte alignment

**Implemented.** `&Schema[compact,alignment]` rounds the bit cursor to the next byte boundary after each field. The generator inserts corresponding writer operations and `reader.align()`. This changes the wire format and may simplify subsequent byte operations.

**Constraints:** wastes bits when preceding widths are not multiples of eight; does not mean 8/16/64-byte physical cache-line alignment. It offers no padding benefit to fields already byte-aligned. Measure cycles rather than assume an automatic speedup.

### `optionals=bitmap` — presence map

**Implemented.** A bit is present for **every declared field**, including required fields; required bits must be one, unused trailing bits zero. The first declared field maps to `1u32<<31`, the second to `1u32<<30`, and so on; each `u32` is written little-endian.

**Constraints:** costs four bytes per 32 fields, potentially excessive for one optional; incompatible with `predict_seq` and `opt_table`. Reordering fields changes bit identities. A bitmap indicates presence but does not automatically provide byte offsets into preceding variable-length fields.

### `predict_seq` — EOF-inferred single optional tail

**Implemented.** `&SeqNumber[predict_seq] { u64 id; u32? extra; }` encodes `[required fields][optional field if present]`, with **no tag or presence header**. Wire is eight bytes when absent and twelve when present; presence is inferred after decoding the required prefix.

**Constraints:** exactly **one optional**, after all required fields; incompatible with compact, bitmap and simultaneous opt_table. Present zero-byte fields are prohibited. A pointer-only `deserialize_unsafe(ptr)` intentionally panics: use length-aware `deserialize(&slice)` or `deserialize_from_raw_parts(ptr,len)`. `len>8` only detects an additional suffix; a truncated 9–11 byte message is **not** a valid `u32` suffix. The caller must validate complete trusted input. A source-level boolean comparison does not prove branchless assembly.

### `opt_table` — indexed optional directory

**Implemented.** For two or more optional fields following all required fields, the writer emits `[count:u8][count×u32 LE offsets][present payload]`. Entries are zero for absent values, otherwise `offset+1` relative to optional payload. Indices are assigned in declaration order. `OptionalOffsets::parse/offset/field_tail` provides a metadata checker and direct lookups.

**Constraints:** incompatible with compact, bitmap and predict_seq; requires all mandatory fields before any optional. Directory size `1+4K` bytes; offsets fit `u32`; current validation supports no more than 64 optional indices. `OptionalOffsets::parse` **does not check field codecs or UTF-8**. One optional under `opt_table` uses predict_seq-style EOF layout, not a directory; adding a second optional changes the format. For ≥2 optional fields, appended fields preserve existing indices; changing required layout, ordering or field type requires a schema revision.

### `lifetime` — borrowing the input

**Implemented.** `&View[lifetime]{ string text; }` emits `View<'a>` containing `&'a str`, with the `BorrowSerializable<'a>` methods. The backing bytes must remain alive and immutable for the record lifetime. Scalar fields remain value types.

**Constraints:** custom and non-byte-array cases have borrowing restrictions. Borrowed data cannot outlive its source, and borrowing alone does not remove network framing or copying elsewhere. Per-field `binary_opt=true/guard` can introduce a lifetime only for that field, without changing unrelated owned strings.

### `safety` — parsed flag, not an automatic checked decoder

**Status: parsed and stored.** In the supplied generator, `FLAG_SAFETY` is not used to select a distinct validated read path. `deserialize` and `deserialize_unsafe` remain unsafe; optional checked I/O helpers are compiled under the separate `diagnostics` feature. Do **not** treat `&Packet[safety]` as validation of arbitrary network messages. Callers must independently establish field lengths, bounds, UTF-8, values and schema revision before trusted decode.

### `default=EXPR` — numeric initialization

**Implemented.** `u32 x|default=7|` sets the generated Rust `Default` value to seven; `u32? x|default=7|` initializes to `Some(7)`. When an optional field is absent **during decoding**, it remains `None`, not a default-valued `Some`. `unwrap_or_default()` is not required by the wire decoder.

**Constraints:** build-time numeric expressions with range validation; the current generator does not implement arbitrary string or custom-type default values.

## 5. String field modifiers and ownership

### Default `str` / `binary_opt=false`

An ordinary string generates `String` unless explicit schema lifetime yields `&'a str`. Owned decoding allocates a string and can dominate tiny record timings. Dynamic length measures **UTF-8 bytes**, not Unicode scalar values or grapheme clusters.

### `binary_opt=true` — borrowed `&[u8]`

Generates `&'a [u8]` and borrows input payload rather than allocating an owned Rust `String`. The schema still treats those bytes as UTF-8 string content; the unchecked path assumes the input is valid UTF-8 even though the Rust slice type does not prove it. This mode cannot directly borrow the decompressed bytes from compressed frames. Copying a borrowed slice into application storage remains an application-level cost, not a free operation.

### `binary_opt=guard` — UTF-8 view

Generates `Utf8Guard<'a>`, a borrowed **byte-slice view**, not an owning arena or automatic copied buffer. `from_str` is allocation-free; `try_from_bytes` validates UTF-8 once; `from_bytes_unchecked` requires caller invariants. `as_str`, `as_bytes` and `Deref<Target=str>` provide typed access. `read_guard_checked` and trusted `read_guard` are fundamentally different benchmark operations.

### `binary_opt=inline` — owned inline bytes

For `str(0..MAX)` with `MAX≤65535`, generates `InlineString<MAX>`, keeping the UTF-8 bytes and `u16` byte length inside one Rust object. No separate String allocation is needed. However, returning an inline value of capacity 4096 by value can move much more data than the actual string uses; `deserialize_into` can decode directly into caller-owned final storage.

### `decompress=true` — QSL-LZ

For a bounded dynamic string, generates `InlineString<MAX>` with deterministic QSL-LZ v1 encoding and no heap scratch. Compression is selected only if the **complete compressed frame** beats raw payload; otherwise raw mode is used. Compression cannot reduce every possible input, and it can cost more CPU cycles than it saves bytes. Fixed `str(N)` always generates `[u8;N]`; fixed compression **revision 2** uses either exactly N raw bytes with no header, or `0xFF + length + tokens`, never exceeding N bytes. Peers must use matching fixed codec revision.

### Fixed `str(N)` and mixed schemas

`str(1024)` is `[u8;1024]` independent of `binary_opt` modes. There is no dynamic length in its raw representation. A numeric field alongside a `String` does **not** imply automatic compression of the whole record; QSL applies string compression only to explicitly marked string fields. `compact` ranks numbers separately. Bounded strings count **UTF-8 bytes**, not characters.

## 6. Detailed schema walkthroughs

### `Fixed` — literal byte offsets

```qsl
&Fixed { u32 id|default=1|; u64 created_at; u16 checksum; }
```

Generated `struct Fixed { id:u32, created_at:u64, checksum:u16 }` implements `Serializable`, including `get_size()=14`, `serialized_size()=14`, `serialize_unsafe`, `deserialize_unsafe` and companion methods. The fixed flat writer uses little-endian unaligned stores at literal offsets `0/4/12`. Rust struct padding is not serialized. The runtime bottleneck moves from schema interpretation to memory writes and subsequent transfer.

### `Dense[compact]` — static rank masks

```qsl
&Dense[compact] { u8(0..7) flags; u8(0..31) rank; u16(0..1023) code; }
```

Three constrained fields occupy 3+5+10 = 18 bits → 3 wire bytes. The generator emits constant masks/shifts and compact methods. The size reduction does not mean latency always improves; unpacking and alignment still incur instruction costs.

### `Packet` / `Tagged` — presence overhead

```qsl
&Packet { u64 sequence|default=0|; u16? priority; }
&Tagged { u32 id; u16? a; u8? b; }
```

`Packet` has one presence bit rounded to a byte in noncompact layout, followed by its required u64 and optionally u16. `Tagged` has count and assigned optional tags. When numeric payloads are tiny, metadata and conditional code can dominate the work. The indexed `opt_table` alternative trades a larger directory for direct access with many optional fields.

### `SeqNumber[predict_seq]` — one dependent tail

```qsl
&SeqNumber[predict_seq] { u64 id; u32? extra|default=7|; }
```

Wire size is **8 or 12** bytes. The length-aware read method determines optional presence from cursor position; missing fields decode as `None`, regardless of the default initializer. A nine-to-eleven-byte message is invalid and must not be passed to the unchecked decoder.

### `TableFields[opt_table]` — offset directory

```qsl
&TableFields[opt_table] {
    u64 id; u32? a;
    str?(0..100) text|decompress=true|;
    u16? b;
}
```

First, required `id`; next a directory with count three and three `u32` offsets; finally the present optional payloads. An offset of zero means absent. Generated fields remain `Option<T>`; `OptionalOffsets` verifies metadata, not the string codecs. Table cost can outweigh its advantage for a small K when every field is decoded sequentially.

### `ByteView`, `GuardView`, `StringView`, `InlineView`, `PackedView`

These five supplied demo schemas use the same 0..4096 UTF-8 byte range but produce different Rust ownership modes. StringView owns `String`; ByteView borrows `&[u8]`; GuardView borrows `Utf8Guard`; InlineView owns `InlineString<4096>`; PackedView owns `InlineString<4096>` and enables QSL-LZ framing. Benchmarks must distinguish full strings of 1024 Unicode scalar values from a rotation across 1024 short strings; actual byte size is 1024/2048/4096 across character categories.

### `Token` — manually implemented custom type

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token(pub u16);
impl qsl::QslCustom for Token {
    const SIZE: usize = 2;
    unsafe fn qsl_write_unchecked(&self, to: *mut u8) {
        std::ptr::write_unaligned(to.cast::<u16>(), self.0.to_le());
    }
    unsafe fn qsl_read_unchecked(from: *const u8) -> Self {
        Self(u16::from_le(std::ptr::read_unaligned(from.cast::<u16>())))
    }
}
```

This exact type occurs in `qsl-demo/src/main.rs`, used by `CustomPacket[safety]` and `CompactCustom[compact]`. `SIZE`/`COMP_SIZE` must match wire bytes; pointer validity, alignment and little-endian representation are part of the unsafe contract. Incorrect invariants can cause undefined behavior.

## 7. Custom codecs — contract, speed and planned macro

`QslCustom` is a **compile-time-sized** trait, not a dynamically dispatched object. Required `SIZE`, `qsl_write_unchecked` and `qsl_read_unchecked` provide the normal path. `COMP_SIZE`, `qsl_comp_write_unchecked`, and `qsl_comp_read_unchecked` have defaults; `qsl_size` and `qsl_comp_size` expose constant sizes. The generator can inline hooks, but **cannot eliminate validation that users write inside them**.

Potential slowdowns: extra range checks, missed inlining, temporary struct copies, endian transforms, variable internal arrays, and indirect helper calls. **The provided Criterion export does not contain a direct quantitative experiment proving that all custom codecs are slower than generated ones**; any such claim requires a controlled comparison of equal values and equal validation guarantees.

Constraints: fixed `SIZE` and accurately described compact size, valid input and output pointers, and known advancement rules. `predict_seq` forbids a zero-byte optional custom codec. The generator rejects some variable custom arrays whose pointer advancement cannot be fixed at build time. A native `transmute` fast path would require separate bit validity, padding, ABI, endianness and alignment guarantees; no universal transmute codec is implemented here.

**Planned:** a **structure macro** to generate `QslCustom` glue and eliminate hand-written boilerplate. Its exact public syntax and implementation are not present in the uploaded source set. It should not be documented as released or benchmarked until a macro and measured comparisons are available.

## 8. Buffers, direct storage and sockets

`serialize()` allocates a single output buffer (`Vec<u8>` or `SerOut` under the `native` feature). `unsafe serialize_to()` writes into caller-provided storage without independently verifying capacity. `SerOut::with_capacity_aligned(capacity,alignment)` offers reusable aligned native storage; `clear()` preserves capacity. `deserialize_into(ptr,len,out_ptr)` can decode inline fields in the final allocation instead of returning a full maximum-sized object by value.

`RawSocket` is **binary framed TCP**, not a raw IP socket. Each frame is `[length:u32 LE][QSL bytes]`. `receive(&mut storage)` bounds and reads a frame into caller memory; `send_record` encodes directly after its prefix; `TCP_NODELAY` is enabled. Frame validation is **not full schema/UTF-8 validation**. The k6 WebSocket campaign and RawSocket TCP are separate paths and should not be conflated.

## 9. Measured performance and methodology

All measurements below are extracted from the attached `qsl-bench/results/all-estimates.csv`; they are not portable guarantees. Criterion 0.8.2 measures prepared values and preallocated outputs, with separate read and write phases. Profiles differ in timing and changes. From the **`criterion-utf8-final`** profile, each operation handles **one complete 1024-scalar ASCII string**:

| Implementation | ASCII read, ns | ASCII write, ns |
|---|---:|---:|
| borrowed `&[u8]` | 1.48 | 19.65 |
| borrowed `Utf8Guard` | 1.47 | 19.43 |
| owned `String` | 89.94 | 20.36 |
| `InlineString` by value | 82.99 | 29.48 |
| `InlineString` `_into` | 19.06 | — |
| compressed `_into` | 19.97 | — |

From the **separate `criterion-utf8-complete`** profile: `ASCII read_guard=2.49 ns`, `read_String=93.64 ns`; Cyrillic `read_guard=1.74 ns`, checked UTF-8 guard validation `2777.32 ns`; four-byte `read_String=290.23 ns`. Different profiles must **not** be treated as a single synchronized run. `read_guard_checked` validates UTF-8; trusted `read_guard` does not. A ≤25 ns full-string target is **not** met for every implementation or input.

Optional-field results from `criterion-utf8-complete`: `optional/seven_table_read=13.49 ns` vs `optional/seven_tags_read=59.99 ns` for those exact layouts; `predict_seq_absent_read=2.17 ns` vs `legacy_absent_read=2.60 ns`. These are cases, not universal speed ratios. Tables cost extra bytes on the wire.

Criterion `records/Fixed/read/current=2.24 ns` vs `start=1.98 ns` documents a **regression**, despite `write/current=2.90 ns` vs `start=3.06 ns`. `records/User/read/current=95.90 ns` vs `start=104.00 ns` shows why an owned string is a much heavier test than a fixed scalar. The separate 16 MiB copy case `bandwidth_16MiB/current≈6.89 ms` measures a large transfer, not one field's serialization.

Prior WebSocket smoke results compare QSL with legacy QSL, JSON, MessagePack and Protobuf; their CPU and RAM include entire server and k6 processes, not isolated codecs. Transport overhead, useful payload sizes and batch shape influence rankings. Hardware performance counters and assembly inspection are distinct from Criterion source timings.

### Historical WebSocket run ≠ updated string fast path

The k6/WebSocket comparison used **the earlier QSL string implementation**. Borrowed `&[u8]`, `Utf8Guard`, and in-place/inline string optimizations were introduced afterward. Accordingly, its RPS, RTT, CPU and RAM do not characterize the new string path. Later Criterion string timings must not be substituted for a new end-to-end test. See [the WebSocket report](../performance.en.md) for the version comparison and limitations.

## 10. Where throughput can collapse

| Bottleneck | Mechanism | Available mitigation | Tradeoff |
|---|---|---|---|
| Owned UTF-8 read | heap allocation and copy | byte borrow, guard, lifetime | source must outlive borrowed result |
| Large inline return | moving `InlineString<MAX>` | `deserialize_into` | aligned uninitialized final storage required |
| Many optionals | tags/presence processing | `opt_table` | `1+4K` directory bytes |
| Single optional suffix | presence header | `predict_seq` | precisely one nonzero-byte tail and a known input length |
| Bounded integers | full Rust widths on wire | `compact` | bit operations and edge conditions remain |
| Poorly compressible text | token search/header | QSL-LZ raw fallback | not all strings shrink or meet time targets |
| Custom record | checks/inlining/temporary copies | unchecked `QslCustom` hooks | user owns pointer/format invariants |
| Large payload | memory bandwidth and cache | `serialize_to`, buffer reuse | memory movement is not free |
| Multiple cores | true sharing and false sharing | application-local regions | atomic region cluster not yet implemented |

Optimization is meaningful only in the context of **schema + operation + platform**. Hold wire format, prepared input, `target-cpu`, output storage, benchmark profile and confidence interval constant before claiming an improvement.

## 11. Implemented vs planned vs unsafe contracts

**Implemented in supplied sources:** QSL/DEQ parsing, `QslInterpreter`, compact/alignment/lifetime/bitmap, predict_seq, opt_table, binary_opt, decompress, QslCustom, OptionalOffsets, Utf8Guard, InlineString, native SerOut, RawSocket and diagnostics helpers.

**Not identical to implemented functionality:** `safety` is currently only a parsed flag in the inspected code; the proposed QslCustom structure macro is not present; the thread-local region distribution and atomic cluster with the `R` occupancy bitmask remain **architectural proposals**, not active Rust APIs.

**Precise ownership note:** supplied `Utf8Guard` borrows a byte slice; it does not own a copied arena. If a future owning guard or region layout is added, it requires separate lifetime rules, tests and API documentation.

## 12. Method-level index and source references

See the generated `reference/API.en.md` and `reference/API.ru.md` for all discovered public signatures, modules, source locations and adjacent documentation comments. Main groups: `Serializable`, `BorrowSerializable`, `QslCustom`, `QslInterpreter`, raw and bit I/O, optional offsets, string storage, checked diagnostics, `SerOut`, and `RawSocket`. Full benchmark data: `docs/data/all-estimates.csv`, `all-estimates.json`, `profiles.json`. Grounding: uploaded `qsl/src/*.rs`, `qsl-demo/schemas/*.qsl`, `qsl-demo/src/main.rs` and `qsl-bench/benches/*.rs`.
