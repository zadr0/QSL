# Generated QSL methods — implementation walkthrough

The fragments below are **explanatory pseudocode**, not literal compiler output from a particular `qsl_gen.rs` build. Actual dispatch is implemented in `qsl/src/interpreter.rs` based on all schema properties. Runtime method names and trait contracts are grounded in the supplied source.

## `Fixed`: constant flat path

```qsl
&Fixed { u32 id; u64 created_at; u16 checksum; }
```

```rust
// Explanation of direct writes; actual emitted Rust may be inlined differently.
unsafe fn fixed_write(value: &Fixed, out: *mut u8) {
    out.cast::<u32>().write_unaligned(value.id.to_le());
    out.add(4).cast::<u64>().write_unaligned(value.created_at.to_le());
    out.add(12).cast::<u16>().write_unaligned(value.checksum.to_le());
}
```

No runtime type directory is consulted: offsets 0/4/12 were computed earlier. However, values must still be loaded, stored and eventually moved through memory. This path does not guarantee lowest time on every CPU. `get_size` and `serialized_size` become 14; the trait write delegates to generated `__qsl_write_unchecked`, and pointer decode to `__qsl_read_unchecked`. See `emit_flat_write/read`.

## `Dense`: compact ranking

```qsl
&Dense[compact] { u8(0..7) flags; u8(0..31) rank; u16(0..1023) code; }
```

For `flags=6, rank=29, code=777`, the illustrative LSB-first packing equation is:

```text
packed = (flags & 0b111)
       | ((rank & 0b11111) << 3)
       | ((code & 0b1111111111) << 8)
wire_length = ceil(18 / 8) = 3
```

`emit_compact_flat_write/read` emits constant bit masks/shifts. A smaller wire representation can be offset by bit manipulation costs. The generator may lower to different instructions based on target CPU and LLVM.

## `SeqNumber[predict_seq]`: size-dependent optional tail

```rust
// Pseudocode valid only for a complete schema-valid message.
let id = read_u64_le(input);
let extra = if input.len() > 8 {
    Some(read_u32_le(&input[8..12]))
} else {
    None
};
```

For `len=9..11`, a naive presence test would lead to an incomplete field. Trusted decode **requires** an already validated record; checked network decode needs an explicit full-length check. Pointer-only `deserialize_unsafe(ptr)` intentionally panics because presence cannot be determined without a length.

## `TableFields[opt_table]`: directory and indexed read

```text
required u64 id | count=3 | u32 offset[0] | u32 offset[1] | u32 offset[2] | payload
                   03       01 00 00 00     00 00 00 00     05 00 00 00      a(4B) + b(2B)
```

Illustrative `offset(i)` logic: `raw=read_u32_le(directory+4*i); raw==0 ? None : Some(payload+raw-1)`. The table avoids searching for the field start but does not eliminate the field codec. `OptionalOffsets::parse` validates directory metadata, not each UTF-8 string or user type.

## Trait methods and costs

- `get_size()` — build-time minimum wire size, not always the actual message length.
- `serialized_size()` — exact size for ordinary fields, upper bound when compression may be selected; this is not always the written length.
- `serialize()` — one output allocation (Vec or native SerOut) and unchecked write.
- `serialize_into_unsafe(out_ptr)` — encode and report initialized wire length.
- `serialize_to(&mut buffer)` — reuse caller storage, without a runtime capacity check.
- `deserialize(&[u8])` — still **unsafe** despite accepting a slice.
- `deserialize_from_raw_parts(ptr,len)` — required for predict_seq size-dependent presence.
- `deserialize_into(ptr,len,out)` — directly initialize final aligned storage, useful for large InlineString values.
- `BorrowSerializable<'a>` — corresponding borrowed operations; input allocation remains valid throughout `'a`.

Full source signatures and line references are in [`API.en.md`](../API.en.md).
