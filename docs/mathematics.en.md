> **0.2.0:** [current contracts](migration-0.2.0.en.md). Timing tables below retain their captured historical sources. The [November 7 pass](../qsl-interpreter/tests/optional-v2/results/2026-11-07.md) is not accepted as a strict result; it establishes no speedup.

# QSL: mathematics and physical cost

[Русский](mathematics.ru.md) · [Guide](guide.en.md) · [Charts](index.html#measurements)

## Information and reversibility

Four arbitrary bytes have 256⁴ = 2³² combinations. A reversible code for every combination cannot always use fewer than 32 bits. Grouping into u32 supplies a convenient operand and shared constraints; it does not itself reduce the domain.

Compression uses declared bounds or actual redundancy. For min≤x≤max, rank r=x-min requires B=bit_length(max-min) bits. A singleton domain has B=0. Widths of independent fields add, while native Rust types may stay large.

## Scalar QPS specialization

Q selects k=min(log2(Q),B), a=B-k. Both operands u=r>>k and v=r&((1<<k)-1) are retained as c=u|(v<<a). Restore r=((c&((1<<a)-1))<<k)|(c>>a), x=min+r. No remainder is discarded and no precision is lost. B=0 and extreme widths use separately generated valid shift cases.

Q may change bytes without changing length. Savings follow domains and packing, not an obligatory division of message length by Q.

## General byte groups

Use base a=min(x_i) and ranks r_i=x_i-a. For a group of n≤Q bytes: B=bit_length(OR r_i), d=2ᴮ, W=Σr_i·dⁱ. Every rank is below d; division by powers of d and remainders uniquely recover each digit. The last partial block uses its actual n.

Packed candidate size is C=16+Σ(1+ceil(n·B/8)). Select packed when C<M_input; otherwise raw uses M_input+16. The header stores original length without quantization. Header cost must not be removed to improve the reported ratio.

The shared base depends on every supplied byte. Inner lengths and zero-valued numeric bytes can change the text payload base and eliminate savings. UTF-8 alone and whole records are therefore compared separately. Repetition does not guarantee success for this codec; LZ exploits different redundancy.

## Prediction remains separate

The predict_seq graph combined with opt_table uses sets of admissible lengths. In an event, presence may be forced, possible or impossible. I(L)=Σ(L>c_j) turns crossed boundaries into an integer. Ambiguity requires metadata. Packed frame length does not replace ordinary-message L.

QPS invertibility and uniqueness of length interpretation are separate proof obligations with different inputs, invariants and limits.

## Performance index P

P=M/T: M is useful original bytes and T is ns/operation. Variable optionals use mean M under the same state distribution. P does not replace encoded size or native object size. A borrowed view may avoid copying most payload; its high P describes view creation.

For positive T∈[T_low,T_high], P∈[M/T_high,M/T_low]. This transforms an estimated interval rather than adding an independent measurement. Waveforms show sample averages, not individual-call latencies.

## Instructions, caches and memory

ASM instruction count is not cycle count. Operand dependencies, load latency, execution-port throughput, branches, calls and code size have different costs. LLVM may merge identical functions, remove a copy or select memcpy.

Small repeated messages may use registers and caches; larger sets encounter cache misses, pages and RAM bandwidth. A small-schema P is not a measurement of total physical memory speed.

Aligned buffer starts do not align every field. Disjoint regions are required by copy_nonoverlapping and ownership rules. Static expansion removes analysis work, but excessive inline can pressure the instruction cache.

Earlier saved ASM wrappers describe their own snapshot. The new general codec has a group loop, table selection and safe-wrapper checks. No per-byte if inside an arithmetic kernel does not imply zero branches in a complete operation.

BMI2 provides bit extraction/deposit through pext/pdep. CPU-path selection changes neither the reversible mapping nor wire vectors. Performance depends on microarchitecture; instruction availability is not proof of a speedup. The repeat changes only 22 byte-QPS cases, not the whole previous profile.

## Formal and executable checks

The [Lean4 file](../reference/proofs/QPS.lean) contains 22 checked arithmetic statements without sorry. Some concern QPS radix/ranks; others describe independent length/state models. This does not verify the complete Rust implementation.

[Wire vectors](../reference/proofs/qps_bytes_vectors.rs) check QPB3, unquantized length, exact group words and every quantum. Other tests cover mutations, overflows, bounds and typed restoration.

```powershell
# In the matching source checkout:
cargo test --manifest-path qsl/Cargo.toml --test qps_bytes_vectors --features native,rust-macros --locked
# In proofs/ with the pinned Lean toolchain:
lake env lean QPS.lean
```

## BM v2 and custom rank bounds

For N optional fields choose W in {8,16,32,64,128}. Marker i (1<=i<=N) occupies 1<<(W-i). For a nonzero mask, m=W-ctz(mask) is the largest present marker. If m<=N, then 0<=m-1<N; clearing the least-significant set bit can only decrease the next marker. This is the arithmetic bound behind a single spare-bit check and subsequent dense OT lookups. N=0 has no mask. [Lean assumptions](../qsl-lexer/proofs/README.md).

A scalar wrapper domain [a,b] uses rank r=x-a. Let B=bit_length(b-a) and k=ceil(B/8). Its custom compact codec stores k bytes; decode checks r<=b-a before constructing x=a+r. B=0 restores a constant without value bytes. Optional presence remains separate. This exact whole-byte codec does not imply automatic bit merging with neighboring fields.

P=M/T uses original useful bytes, not encoded bytes or buffer capacity. For T in [low,high], P lies in [M/high,M/low]. A wide interval or loaded CPU cannot establish a speedup by choosing a favorable point estimate.
