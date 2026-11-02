# QSL: current architecture

[Русский](architecture.ru.md) · [QSL measurements](performance.en.md) · [API index](API.en.md)

## Purpose and invariants

QSL generates specialized Rust for exchanging data described in advance. Schemas define types, ranges, string ownership and physical order. Static analysis and primitive selection happen at build time.

Attributes and additional mechanisms are opt-in. Order is always mandatory: `[required][optional]`. Tags are compiler-assigned incrementally. Data is never silently reordered.

**QPS is exact numerical operand compression with quantization. opt_table performs prediction from length. QPS does not require optional fields.**

| Mechanism | Selection | Meaning |
|---|---|---|
| QPS | qps, qps=Q2/Q4/Q8/Q16 | Reversible numeric rank packing |
| opt_table | opt_table | Static length/event/offset graph |
| predict_seq | predict_seq | Direct predictive path; standalone sequential behavior |
| quant | quant=Qn | Compatible explicit-quantum QPS alias |
| state_bits | state_bits=Q1/Q2/Q4/Q8/Q16 | Explicit legacy D/u32 compatibility |
| safety | safety | Checked data and value APIs |
| inline | inline=hint/always/never | Inlining policy, independent of encoding |

## Build

build.rs supplies QSL, DEQ and Rust-template paths. Parsing resolves typed constants and validates ranges/order. Rust declarations expand before layout and codec selection. QslInterpreter emits qsl_gen.rs, then Rust compiles the specialized types and methods.

```rust
qsl::build_qsl_files_with_rust(
    &["schema.qsl"], &["schema.deq"], &["models.rs"], out_dir,
)?;
```

Templates require the rust-macros feature in normal and build dependencies. Without them, use build_qsl_files.

## QPS mathematics

```qsl
&P[qps=Q16,safety] { u64(0..15) a; u64(0..15) b; }
```

Required-only schemas are valid. `[qps]` defaults to Q16. Q1 is not a quantum of new QPS. Do not specify both aliases qps and quant.

For `a <= x <= b`, rank `r=x-a`, width `B=bit_length(b-a)`:

```text
k = min(log2(Q), B)       d = 1 << k
u = r >> k               v = r & (d - 1)
c = u | (v << (B-k))
r = ((c & ((1 << (B-k))-1)) << k) | (c >> (B-k))
x = a + r
```

Both quotient and remainder are retained. There is no rounding. B=0 constants restore from the schema without payload bits; code generation separately handles extreme widths and avoids shifts by a register's full width.

Savings come from bounded domains and packing, not from shrinking arbitrary data by arithmetic. Eight u64(0..15) values occupy 4 payload bytes instead of 64. Q2/Q4/Q8/Q16 can have equal lengths but different exact bytes; distinct golden vectors are tested.

Unsupported types and no saving including private padding select explicit raw fallback. Analysis exposes QUANT_ACTIVE and a reason. The decision is per schema. A short individual frame need not shrink when fixed metadata is present.

None differs from Some(constant). QPS alone does not infer presence from length. With opt_table, known bit widths and Boolean prefixes supply offsets; compressed payload length never replaces restored object size.

## Buffers and methods

Packed frames and restored objects occupy separate regions. QPS does not reduce size_of::<T>(). Prepared pointer APIs require initialized QUANT_PADDING_BYTES, never transmitted. Exact-slice checked decoding requires no hidden padding.

| API | Contract |
|---|---|
| serialize_quantized_to | Check values/capacity and reuse caller storage |
| QuantizedFrame::decode_into | Restore into a separate MaybeUninit<T> |
| Active QPS serialize() | Result with QuantizedOut or an RT lease |
| QuantizedOut::encode | Allocate output SerOut and private padding |
| decode_boxed | Additionally allocate a Box for the restored object |
| quant_read_padded / quant_write_prepared | Trusted kernel; caller ensures ranges, padding, capacity and non-overlap |
| T as Serializable | General exact wire path with its own contract |

```rust
let value = P { a: 3, b: 15 };
let mut storage = [0u8; 1 + P::QUANT_PADDING_BYTES];
let frame = value.serialize_quantized_to(&mut storage)?;
let decoded = P::deserialize_checked(frame.as_slice())?;
```

Raw fallback keeps the original serialize() API. Do not assume every mode returns the same container. deserialize_into writes into final storage; live values require the appropriate reuse API.

## opt_table prediction

```qsl
&Packet[opt_table,safety] { i32 id; u8? a; u8? b; }
```

H is the known metadata span, B the consumed required prefix, L the optional-payload length. `I(L)=sum((L > boundary_j) as usize)`. Boundaries come from the schema; adjacent equivalent events merge. No dense per-byte array is required.

Here I=(L>0)+(L>1). L=0 identifies no optionals; L=2 forces both; L=1 needs metadata to distinguish a and b. EVENT_TABLE selects specialized functions. A deterministic handler skips tag reads although the reserved byte remains on wire. Frames occupy 5/6/7 B.

For str?(3..33) name and str?(1..255) description, lengths above 33 force description while allowing name. Above 255 both are present. Native length words occupy metadata; the last variable field comes from the remainder. Real ambiguity needs metadata.

CRITICAL_POINTS, EVENT_TABLE, OFFSET_TABLE and reader tables sit beside the type. OFFSET_TABLE stores recipes. `[opt_table,predict_seq]` replaces the indirect call with direct expressions without changing bytes. Inline is independent. Generic budget: 12 fields / 65535 B; equal fixed widths support 64 fields. Fallbacks are explicit. `[opt_table=legacy]` preserves the old per-frame directory.

## Strings

| Description | Rust | Ownership |
|---|---|---|
| str(N), equal bounds | [u8; N] | Fixed byte array |
| Ordinary/bounded str | String | Owned; decoding may allocate |
| str with lifetime | &'a str | Borrows the frame |
| binary_opt=true | &'a [u8] | UTF-8 byte view |
| binary_opt=view | Utf8Guard<'a> | Borrowed guard |
| binary_opt=guard | Utf8CopyGuard | Copies bytes into reusable SerOut |
| binary_opt=inline | InlineString<MAX> | Inline capacity and length |

str(A..B) counts UTF-8 bytes, not Unicode characters. Fixed strings always remain arrays. binary_opt applies to strings. Borrowing requires the input to stay alive; CopyGuard is owned and does not remove initial storage cost.

decompress=true independently enables bounded QSL-LZ. Selected compressed frames are smaller than raw; otherwise raw fallback is used. QPS does not replace LZ. Fresh numeric cases do not measure a 1024-character string.

## Rust structure macros

```rust
#[qsl::schem_macro]
pub struct Clock { pub ticks: u64 }
#[qsl::qsl_macro]
pub struct Meta { pub clock: Clock, pub kind: u16 }
```

```qsl
&Message[safety] { Meta meta; u32 id; u8? extra; }
```

Original Rust types remain nested. Only Message gets meta_clock_ticks, meta_kind, id and extra. No duplicate types are emitted. There is no QSL [macro] attribute. Expansion checks cycles, collisions, ownership, final order, 64 fields and depth 64. Inputs are explicit; arbitrary generics, cfg and computed capacities are not guessed.

## Validation, RT and networking

[safety] generates validate, validate_values, deserialize_checked and checked decoding into final storage for owned types. Bounds, metadata, ranges, UTF-8, LZ and exact ending are checked. Malformed built-in data returns QslError before trusted reads. Custom codecs need their own correct validator. The attribute does not add checks to every trusted call.

```deq
!config QSL_RT = true;
!config QSL_RT_DEFAULT = 16;
```

RT defaults off. Only finite schemas receive static slots. Here 16 counts simultaneous occupied buffers per schema, not QPS quantum. Exhaustion returns an error without allocating another buffer. Padding and the 64 MiB per-schema budget are included; deserialize_into remains available.

The native feature supplies Windows NativeRegion, SharedRegion with IPC, and blocking NativeSocket. Shared publication uses release/acquire; processes exchange offsets, not absolute pointers. TCP framing does not replace QSL validation. None of these mechanisms was included in the fresh Criterion run.

## Editor, compatibility and proofs

[QSL-Extension 0.5](https://github.com/zadr0/QSL-Extension): RU/EN, inlays, field hover, outline, completion, definitions and Ctrl+. layout/events/Rust actions. QPS needs no optional tail. Exact analysis follows Rust-template expansion. Advisory estimates are labelled; the native bridge exposes qpsCodecRevision=2. Updating server files requires restarting its process.

Before revision 2, qps selected a different D/index format. Use state_bits to preserve those bytes. quant=Qn retains its numeric operand format and aliases new QPS. Schema, configuration, boundaries and generator revision are part of peer agreement; no len-based format guessing occurs. QBProto handshake/R remains planned.

Lean 4 verifies 18 arithmetic theorems, not all Rust/LLVM/allocation. Checks passed: 69 core unit, 10 serial native, 2 compile-fail and 3 isolated QPS regressions. Four fresh measurements, ASM, harness and hashes are published separately. Intentionally invalid local View demo files were not repaired just to make the demo build pass.

[Separate baseline control](opt-table-control.en.md) compares default and opt_table at five optional-field counts without QPS.
