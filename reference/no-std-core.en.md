# no_std runtime: storage, generation and verification

Local dev-qsl, November 14, 2026; unpublished. No website work or new accepted measurements are included in this pass.

## Crate boundaries

| Crate | Responsibility |
|---|---|
| qsl-core | Always no_std: byte/bit I/O, serialization, containers, primitive contracts/macros, validation, UTF-8/LZ, QPS, dyn and storage traits |
| qsl-rt | Always no_std: local and thread-shared storage providers implementing core contracts |
| qsl-lexer | Build host: source/DEQ, schemas, Rust metadata, analysis and CompilationPlan |
| qsl-interpreter | Build host: specialized Rust generation using core/alloc paths |
| qsl | Runtime facade with an optional compiler feature |

Runtime code is not copied into the frontend or backend. ByteTransport/RawSocket provide OS-independent transport and u32 LE framing; applications supply concrete sockets. There are no Windows-specific socket or IPC modules.

## no_std versus alloc

no_std excludes the Rust standard library, not alloc. The default core alloc feature provides alloc::String/Vec, SerOut, boxed InlineString and owned QPS buffers. QSL never declares a global allocator. The final application supplies one; standard alloc containers currently use it, not a QSL-local allocator.

Use qsl with default-features=false for caller-buffer-only runtime; add features=["alloc"] when needed. Put the generator in build-dependencies. Host build.rs may use std and allocation without including them in the target runtime. FrameStorage/FramePool extend output storage; they do not change the allocator inside standard String/Vec.

## Container semantics

StackString<N,L> stores raw UTF-8 inside the object; InlineArray<T,N,L> stores a bounded initialized prefix and drops only initialized elements. Normal InlineString<N,L> selects QSL-LZ. Large bounded inline strings generate Box<InlineString<N>> and restore directly into heap memory without an N-byte stack temporary. Moving a container to Box preserves its selected codec.

Ordinary owned text/arrays use alloc::String/Vec; lifetime mode can use &str. UTF-8 lengths count bytes: “Яa” has length 3. Fixed strings/arrays use [u8;N]/[T;N] when native layout and strict limits permit; large fixed strings may use String.

Explicit aggressive[shift=true] selects len+1 in a fitting NonZeroU8/U16/U32 for supported inline strings/arrays. None differs from Some(empty); physical length storage does not change wire. Earlier measurements rejected automatic shifting.

Rust-source imports and typed Builder now agree: normal InlineString means QSL-LZ. This changes wire/hash for old Rust-source imports that treated it as raw. The existing aggressive_transmutate native-template rewrite is separate: it produces a fixed [u8;N] field and follows that actual type's native ABI.

## Strict limits

Default STACK_LIMIT=256 B includes the selected length word and container alignment. StackString<252,u32> occupies 256 B; capacity 253 requires 260 B. InlineArray<u64,32,u32> needs 264 B. Payload capacity alone is insufficient.

The policy bounds the inline container; Option wrappers, other fields, caller frames and total process stack are separate. loyal_cache additionally requires an admissible whole-schema layout and rejects an implicit heap fallback.

Default MAX_DATA_SIZE=65535 bounds input/restored frames and aggregate decoded storage. Checked Box<InlineString<512,u32>> restoration charges the full 516-byte native object before allocating, even for one active byte. Two such fields need 1032 B. Raw/LZ follow the same budget; allocator metadata and RSS are not measured exactly.

Every LZ token is checked before expansion: literal extent, distance, produced size and destination bound. UTF-8 is checked before publication. A forged length header cannot authorize an output overflow. A malformed first token is rejected before Box allocation; subsequent tokens are parsed once before expansion.

## New and reused destinations

Generated deserialize_into uses DeserializeDestination: &mut Self is initialized reuse, while raw-pointer/MaybeUninit destinations follow the creation contract. Never create an &mut Self reference to uninitialized memory.

For layouts without optional fields, lifetime, dyn or scalar QPS, generated trusted reuse retains required String, Box<InlineString> and primitive Vec buffers; inline containers update in place. Tests cover empty/nonempty transitions, UTF-8, raw/LZ, fixed strings, full-width u128/i16, compact byte-array barriers and NonZero length words. Owned buffer pointers remain stable.

The caller must provide a complete schema-valid frame, nonoverlapping input/destination storage and sufficient capacity for every retained field. Unsafe reuse does not check or grow capacity. REUSES_STORAGE does not promise allocation-free arbitrary custom codecs. Optional/dyn/QPS and checked creation retain separate paths; no general reuse guarantee is made for them.

## qsl-rt providers

LocalRegions<CAP,N> uses Cell instead of atomics and may live inside a function. It is not Sync, and its leases are not Send. StaticRegions<CAP,N> uses an atomic mask for thread sharing. Both supply 1–64 exclusive slots; Drop releases a slot and try_acquire immediately returns PoolExhausted when full. Generated RT serialize also returns an error rather than waiting.

FrameStorage publishes initialized prefixes only. PreparedFramePool::ZEROED/new_zeroed additionally initialize full capacity for padded kernels; new() does not. QuantizedRegions accepts borrowed local pools without a static lifetime. These providers are not IPC memory or alloc::String/Vec allocators.

## Evidence and reproduction

The Windows no_std/no_main executable in qsl-core/tests/no-alloc generates real schemas at build time and runs raw/compact, UTF-8, inline/NonZero arrays, typed macros, borrowed text, optional bitmap/tag paths, native regions, QPS and RT. It exits with code 0. Its actual link command contains no Rust std or alloc rlibs. Windows exit and C memory primitives belong only to the test application.

The separate no_std library qsl-core/tests/alloc checks String/Vec/Box, forged LZ, symmetric checked budgets and buffer retention. Its test harness supplies std/allocator without adding std to the target library. Both core configurations and qsl-rt cross-check for wasm32-wasip2.

~~~text
cargo test --manifest-path qsl-core/Cargo.toml --all-features --locked
cargo test --manifest-path qsl-core/Cargo.toml --no-default-features --locked
cargo test --manifest-path qsl-rt/Cargo.toml --locked
cargo run --manifest-path qsl-core/tests/no-alloc/Cargo.toml --locked
cargo test --manifest-path qsl-core/tests/alloc/Cargo.toml --locked
cargo check --manifest-path qsl-core/Cargo.toml --target wasm32-wasip2 --no-default-features --locked
cargo check --manifest-path qsl-core/Cargo.toml --target wasm32-wasip2 --all-features --locked
cargo check --manifest-path qsl-rt/Cargo.toml --target wasm32-wasip2 --locked
~~~

Local logs and link-proof.json: .optimization/runtime-core-2026-11-14/. Lean proves arithmetic, not Rust ownership/provenance, UTF-8, absence of UB or speed. Miri/sanitizers and hardware counters require separate evidence. [Remaining acceptance work](unified-optimizer-2026-11-12/remaining.en.md) · [Optimization candidates, not enabled changes](optimization-candidates-2026-11-14.en.md).

## Schema-selected serialize output

Ordinary serialize()/serialize_checked() return Result with a concrete output type. A proven maximum frame size less than or equal to STACK_LIMIT selects StackOut; otherwise output is alloc::Vec<u8>. StackOut length/alignment metadata is excluded from this frame threshold. Unknown bounds also select Vec. This choice is fixed in generated Rust, with no runtime stack/heap branch.

StackOut<N> holds uninitialized byte capacity and a u32 written length. It exposes as_slice(), AsRef<[u8]>, Deref, len(), is_empty() and capacity(); only the initialized prefix is readable. A 64-byte maximum selects StackOut<64>. At STACK_LIMIT=256 B, a frame of 256 bytes selects StackOut and a frame of 257 bytes selects Vec. Transmitted headers count toward frame size; buffer-object metadata does not. Two strings bounded by 4 KiB select Vec even for short actual values; only the current required capacity is reserved.

Compression is not assumed to shrink the maximum: raw fallback and headers count. Small scalar QPS serialize() preserves a typed QuantizedLease backed by StackOut and its frame() decoder. Explicit QSL_RT retains its pool/lease contract. Caller-buffer serialization remains available. Trusted Serializable/BorrowSerializable::serialize returns SerializeOutput::Output; manually implemented records opt into SerializeOutput explicitly. Unchecked methods retain their separate contracts.

Internal code uses qsl_lexer::layout and qsl_core directly. Files containing only forwarding exports were removed; public facade exports stay in lib.rs. Canonical implementation paths are qsl_core::storage, qsl_core::serialize, qsl_core::payload_quant, qsl_lexer::layout and qsl_lexer::payload_quant.

### Configuring the output limit

DEQ `!config STACK_LIMIT = 128;` and Builder `config(|c| { c.stack_limit(128); })` set the same GenerationConfig.stack_limit. serialize has no separate threshold. StackOut capacity follows the proven frame maximum; STACK_LIMIT bounds wire bytes inclusively. Changing this limit preserves wire bytes and schema hashes.

At STACK_LIMIT=64, a 64-byte frame selects StackOut<64> and a 65-byte frame selects Vec. A zero limit still allows a proven zero-byte frame on the stack. Limits 63, 65, 128 and 256 are also tested. Compact numeric domains use encoded rank width: sixteen u128 fields restricted to 0..15 need an 8-byte frame, not 256 bytes.

Large borrowed schemas retain caller-buffer serialization in no_std builds without alloc. Missing heap-owned output does not disable serialize_checked_to/serialize_to; the no-allocator executable checks this path.
