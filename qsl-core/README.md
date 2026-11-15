# qsl-core

The runtime of Quick Schema Language (QSL), always built with `#![no_std]`. It implements byte/bit I/O, serialization traits, bounded containers, validation, QPS, framing and storage contracts. No runtime implementation is copied into the lexer or Rust generator.

## no_std and allocation

`no_std` excludes the Rust standard library; it does **not** exclude `alloc`. The default `alloc` feature exposes `alloc::string::String`, `alloc::vec::Vec`, `SerOut`, boxed InlineString and owned QPS output. These APIs use the allocator supplied by the application. This crate never declares `#[global_allocator]`. Without default features, owned allocation APIs are omitted.

| Configuration | Runtime storage |
|---|---|
| Default / `features=["alloc"]` | Caller buffers plus allocator-backed containers |
| `default-features=false` | Caller slices, fixed/inline values, borrowed text and static providers |

`StackString<N,L>` and `InlineArray<T,N,L>` contain their bounded storage. `InlineString<N,L>` supports inline storage or `alloc::boxed::Box<InlineString<N,L>>` without changing its string codec. Heap constructors and checked restoration write directly into heap memory, without an N-byte stack temporary. Fixed native arrays use `[T;N]`. `L` defaults to u32; explicit length shifting selects a fitting NonZero word and preserves the wire. User-provided element types can themselves own memory.

## Layer boundaries

- `qsl-core` owns Serializable/BorrowSerializable, pointer cursors, buffers, primitive contracts/macros, UTF-8/LZ validation, dyn scratch, scalar and byte QPS, schema identities, and transport-independent framing.
- `qsl-rt` implements core `FramePool` / `FrameStorage` contracts with no_std local and thread-shared regions. An exhausted pool returns PoolExhausted immediately; no OS waiting is involved.
- `qsl-lexer` owns source/schema descriptions, DEQ, metadata, analysis and optimizer plans.
- `qsl-interpreter` emits specialized Rust with core/alloc paths.
- `qsl` reexports these APIs; compiler support is optional.

`RawSocket<S>` accepts an application implementation of `ByteTransport`; the application chooses its socket and platform. Core does not depend on std::net. `FrameStorage` and `FramePool` support caller-owned local providers and thread-shared pools. Standard alloc String/Vec currently use the application's global allocator; these traits do not turn them into allocator-parameterized containers.

## Caller-owned output

```rust
use qsl_core::{SliceOut, StackString};
use qsl_core::qps_bytes::ByteQps;
let text = StackString::<32>::try_from_str("Hello")?;
let mut encoded = [0u8; 80];
let mut restored = [0u8; 64];
let frame = ByteQps::<16>::encode_to(text.as_bytes(), &mut encoded)?;
assert_eq!(frame.decode_to(&mut restored)?, text.as_bytes());
let mut output = SliceOut::new(&mut encoded);
output[..5].copy_from_slice(b"Hello");
output.commit(5)?;
assert_eq!(output.frame(), b"Hello");
# Ok::<(), qsl_core::QslError>(())
```

Checked decoding validates input boundaries, UTF-8 and restored-data budgets. Each compressed run is checked against actual output capacity before writing. Unsafe APIs remain separate and require their complete valid-input/memory contracts.

## Limits and reuse

The compiler's STACK_LIMIT includes the selected length word and native alignment of inline string/array containers. For example, StackString<252,u32> occupies 256 bytes; capacity 253 requires 260 bytes. An explicit NonZero length can reduce this cost. Optional wrappers and enclosing fields are separate native storage; STACK_LIMIT is not a whole-stack/RSS limit.

Checked heap InlineString restoration reserves its full native object size against MAX_DATA_SIZE before allocating, including unused capacity. Two small texts in large fixed-capacity boxes therefore cost two full objects. Raw and LZ representations follow the same budget. Each LZ run is still checked independently against remaining capacity, and UTF-8 is validated before the value is published. A malformed first compressed token is rejected before allocating the box.

Generated trusted reuse can retain required String, Box<InlineString> and primitive Vec buffers; bounded stack containers are updated in place. Pass an initialized destination with enough capacity for every retained field, valid nonoverlapping input and the correct schema. The uninitialized pointer destination has a distinct contract. This does not promise reuse for optional, dyn, QPS or arbitrary custom-codec fields, or allocation-free checked creation.

## Verification

```text
cargo test --manifest-path qsl-core/Cargo.toml --all-features --locked
cargo test --manifest-path qsl-core/Cargo.toml --no-default-features --locked
cargo run --manifest-path qsl-core/tests/no-alloc/Cargo.toml --locked
cargo test --manifest-path qsl-core/tests/alloc/Cargo.toml --locked
cargo check --manifest-path qsl-core/Cargo.toml --target wasm32-wasip2 --all-features --locked
```

The standalone Windows no_std executable generates real schemas at build time and runs without linking Rust std or alloc. The build script runs on the host; it is not part of the target runtime. Windows exit and C memory primitives belong only to that test application. A separate no_std library exercises generated alloc String/Vec and heap InlineString. Its unit-test harness supplies an allocator. Core and qsl-rt also cross-check for wasm32-wasip2; unit tests use std only under cfg(test).

Requires Rust 1.98 or later. MIT licensed.

## Schema-selected serialize output

Ordinary serialize()/serialize_checked() return Result with a concrete output type. A proven maximum frame size less than or equal to STACK_LIMIT selects StackOut; otherwise output is alloc::Vec<u8>. StackOut length/alignment metadata is excluded from this frame threshold. Unknown bounds also select Vec. This choice is fixed in generated Rust, with no runtime stack/heap branch.

StackOut<N> holds uninitialized byte capacity and a u32 written length. It exposes as_slice(), AsRef<[u8]>, Deref, len(), is_empty() and capacity(); only the initialized prefix is readable. A 64-byte maximum selects StackOut<64>. At STACK_LIMIT=256 B, a frame of 256 bytes selects StackOut and a frame of 257 bytes selects Vec. Transmitted headers count toward frame size; buffer-object metadata does not. Two strings bounded by 4 KiB select Vec even for short actual values; only the current required capacity is reserved.

Compression is not assumed to shrink the maximum: raw fallback and headers count. Small scalar QPS serialize() preserves a typed QuantizedLease backed by StackOut and its frame() decoder. Explicit QSL_RT retains its pool/lease contract. Caller-buffer serialization remains available. Trusted Serializable/BorrowSerializable::serialize returns SerializeOutput::Output; manually implemented records opt into SerializeOutput explicitly. Unchecked methods retain their separate contracts.

Internal code uses qsl_lexer::layout and qsl_core directly. Files containing only forwarding exports were removed; public facade exports stay in lib.rs. Canonical implementation paths are qsl_core::storage, qsl_core::serialize, qsl_core::payload_quant, qsl_lexer::layout and qsl_lexer::payload_quant.

### Configuring the output limit

DEQ `!config STACK_LIMIT = 128;` and Builder `config(|c| { c.stack_limit(128); })` set the same GenerationConfig.stack_limit. serialize has no separate threshold. StackOut capacity follows the proven frame maximum; STACK_LIMIT bounds wire bytes inclusively. Changing this limit preserves wire bytes and schema hashes.

At STACK_LIMIT=64, a 64-byte frame selects StackOut<64> and a 65-byte frame selects Vec. A zero limit still allows a proven zero-byte frame on the stack. Limits 63, 65, 128 and 256 are also tested. Compact numeric domains use encoded rank width: sixteen u128 fields restricted to 0..15 need an 8-byte frame, not 256 bytes.

Large borrowed schemas retain caller-buffer serialization in no_std builds without alloc. Missing heap-owned output does not disable serialize_checked_to/serialize_to; the no-allocator executable checks this path.
