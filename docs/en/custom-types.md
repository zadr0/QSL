# QSL custom types: implementation and performance contract

**Sources:** `qsl/src/types.rs::QslCustom`, `qsl/src/interpreter.rs`, `qsl-demo/src/main.rs`, `qsl-demo/schemas/custom.qsl`. **Status:** manual QslCustom is implemented; a structure macro is planned but its implementation and public syntax are absent from the supplied sources.

## 1. Purpose

Built-in QSL scalars, ranked ranges, fixed strings and supported arrays are understood by the generator. A custom codec is for a user Rust type whose precise fixed-size binary layout is defined by the developer. This is a compile-time trait, **not** a dynamically-dispatched runtime codec registry.

## 2. Trait contract

```rust
pub trait QslCustom: Sized {
    const SIZE: usize;
    const COMP_SIZE: usize = Self::SIZE;
    unsafe fn qsl_write_unchecked(&self, to: *mut u8);
    unsafe fn qsl_read_unchecked(from: *const u8) -> Self;
    fn qsl_size(&self) -> usize { Self::SIZE }
    fn qsl_comp_size(&self) -> usize { Self::COMP_SIZE }
    unsafe fn qsl_comp_write_unchecked(&self, to: *mut u8) {
        self.qsl_write_unchecked(to)
    }
    unsafe fn qsl_comp_read_unchecked(from: *const u8) -> Self {
        Self::qsl_read_unchecked(from)
    }
}
```

`SIZE` and `COMP_SIZE` must equal **exact wire bytes read and written** in each mode. A changed `COMP_SIZE` value does not by itself transform encoding; override both compact hooks if compact data actually differs.

## 3. Exact uploaded Token example

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token(pub u16);
impl qsl::QslCustom for Token {
    const SIZE: usize = 2;
    #[inline(always)]
    unsafe fn qsl_write_unchecked(&self, to: *mut u8) {
        std::ptr::write_unaligned(to.cast::<u16>(), self.0.to_le());
    }
    #[inline(always)]
    unsafe fn qsl_read_unchecked(from: *const u8) -> Self {
        Self(u16::from_le(std::ptr::read_unaligned(from.cast::<u16>())))
    }
}
```

```qsl
&CustomPacket[safety] { Token code; }
&CompactCustom[compact] { Token code; }
```

This is a two-byte little-endian codec, not a generic transmute of arbitrary Rust data. `Token` has no pointers or invalid scalar bit patterns. The `[safety]` flag **does not** select a complete checked decoder in this supplied generator.

## 4. Generated integration

The interpreter folds custom `SIZE/COMP_SIZE` into compile-time field offsets and lengths. Typical generated decode expression: `Type::qsl_read_unchecked(reader.read_exact(<Type as QslCustom>::SIZE))`; writes invoke the corresponding raw-pointer hook. The compiler may inline the hook, but that depends on the target build. The generator rejects some variable arrays of custom values where pointer advancement cannot be statically established. A `predict_seq` optional custom codec must have `SIZE > 0`, because EOF cannot distinguish a present empty value from absence.

## 5. Unsafe invariants

1. `to` addresses `SIZE` exclusively writable bytes; `from` addresses `SIZE` readable bytes.
2. Read and write consume **exactly** those bytes, with no unreported variable suffix.
3. Pointer operations preserve their nonoverlap and lifetime contracts.
4. All produced Rust values have valid bit representations. Never construct references, enums, bools, nonzero types or other validity-constrained values from arbitrary unchecked bits.
5. Native-endian and ABI-dependent layouts require explicitly restricted platform compatibility. `repr(C)` alone does not remove padding or make arbitrary structures bitwise serializable.
6. Validate the buffer before an unsafe call; neither TCP framing nor the parsed `[safety]` flag proves schema validity.

## 6. Performance tradeoffs

For built-in scalar fields, the generator knows exact width, offset and endian representation and can emit literal pointer instructions. A custom hook might retain checks, dynamic helpers, temporaries, conversions, register pressure, missed inlining and branches. An `unsafe` outer function does **not** eliminate checks explicitly implemented inside the custom codec.

**Evidence limit:** the uploaded Criterion CSV has no isolated equal-contract experiment demonstrating a quantified slowdown for every custom codec. Any percent difference requires matching wire formats, values, output buffers, safety policies and measured assembly; it cannot be asserted from the general record benchmarks alone.

## 7. Proposed structure macro — not released

A future macro might derive constant wire sizes, reject hidden padding/pointers in direct-bit paths, emit direct little-endian loads/stores, encode bit-validity invariants and distinguish trusted from checked interfaces. **No public derive/attribute syntax exists in the supplied source**, so a sample `#[derive(...)]` would be speculative rather than an available API.

## 8. Controlled benchmark design

Compare a generated fixed schema, manual Token codec, future macro Token codec, and checked vs trusted custom paths. Hold the wire layout, data and output storage constant; verify byte equivalence before Criterion measurement. Track latency distribution, cycles/instructions, code size, inlining and throughput over arrays. Fuzz malformed records through a checked validator, never by invoking an unsafe decoder on arbitrary invalid bytes.
