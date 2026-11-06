# QSL modernization verification · 2026-11-06

[RU contract](docs/modernization-2026-11-06/MODERNIZATION.ru.md) · [EN contract](docs/modernization-2026-11-06/MODERNIZATION.en.md) · [Four-case report](qsl-bench/modernization/report.html)

Current implementation: typed exact-size register group reads (u8/u16/u32/u64/u128 selected by analysis); strict per-buffer STACK_LIMIT; initialized-prefix owned storage; single-pass opt-in checked decoding; separate native COPY/overlay and nested regions; native optional niches/ranks/shared bitmap; scalar-only QPS with exact reads; per-schema blocking RT pools; compile-time cache capacity.

## Evidence and scope

- Windows 11 x86-64: 85 core unit tests, 10 native tests, 2 byte-QPS vectors, 12 schema-hash tests and 2 compile-fail documentation tests passed.
- Modernization contracts: 33 tests and 5 compile-fail checks passed in debug and release. Includes protected-page 9-byte access, no private QPS tail, no-allocation checked inline read, malformed frames, native layout/UTF-8, nested regions, strict large-buffer fallbacks, cached-frame data limits, padding boundaries before neighboring fields, unsafe trusted-writer contracts, always-present unchecked native/region APIs, and an instrumented proof that unchecked regions bypass validators.
- Six optional-mode compatibility tests pass against preserved before.rs golden code. Numeric/UTF-8 combined wire bytes and hash identities are retained.
- Four example tests and all seven runnable examples pass. The RT example demonstrates blocking and resumption, not PoolExhausted from serialize().
- Root, contract and examples pass Clippy with -D warnings. Root formatting check passes.
- Zed server: 32 RU/EN/native/LSP tests pass. Tree-sitter: 10 corpus cases pass. Extension release and Clippy target wasm32-wasip2.
- Lean4.35 verifies QPS.lean and Modernization.lean. Arithmetic models exclude Rust provenance/aliasing, atomic memory-order verification, LLVM and OS wait primitives.
- Benchmark source/binary SHA-256, raw Criterion samples, confidence intervals, settings and assembly are retained. Eight GiB Job Object was attached before fixture preparation. No hot metrics sampler. The earlier 5% idle gate was waived; ambient noise is not represented as absent.
- String's 25 ns target was not achieved. Q16 saves wire but reads slower than default for the measured numerical schema. No broad protocol matrix was rerun.

Linux/macOS wait behavior is source-inspected, not runtime-tested here. Browser visual QA is not claimed; the standalone report is checked through offline DOM events, all 4×2 case/metric combinations, 200 sample points and finite coordinates.

[Current source manifest](reference/modernization-source-manifest.json) differs intentionally from [measured source snapshot](qsl-bench/modernization/results/2026-11-06/source-manifest.json): subsequent parser/linter and validation-contract corrections do not rewrite recorded timing evidence.

Historical protocol comparison remains 16/100 qualified observations. Historical feature/example datasets retain their original source snapshots. They are not fresh results for this generated API.

## Unchecked API correction

Safety adds checked entry points and preserves explicit unsafe native/region methods with the _unchecked suffix. Both attribute settings and legacy unchecked names are exercised. Region validators are instrumented: all four unchecked variants make zero validator calls; all four checked variants invoke them, and malformed UTF-8 remains rejected. Layout-compatibility flags are associated constants, not checks in the unchecked path. COPY also tests the type-constant fallback for differing native/wire offsets.

The ordinary reader's u128 result type and the analyzer's rounded registerBits are now distinguished explicitly in the RU/EN contract. Benchmarks were not rerun for this API correction; historical measurements retain their original source snapshots.


## Typed register selection

Ordinary generated groups, QPS prepared reads and hybrid-frame descriptors now use exact typed loaders. A 3-byte window returns u32; 5 bytes return u64; 9 bytes return u128 without a 16-byte input read. The legacy load_exact return type remains u128 for compatibility; generated kernels no longer call it. Crossing signed fields and length descriptors preserve their bit patterns. Unit tests cover all 1–16-byte windows; protected-page tests also pass in release. Zed native analyzer rebuilt; 32 server tests pass.

[Two numerical reruns and five ASM probes](qsl-bench/modernization/results/2026-11-06-register-width/README.md) retain new raw samples and a 54-file measured-source snapshot. No speedup is established: default 2.788 ns and Q16 4.304 ns are about 4% above the earlier separate run; decoder inspection probes have identical instructions. These results do not replace the four-case historical table.


## Checked LZ restoration bounds

A shared checked token parser validates packed literal/offset reads, backward distances, checked decoded-length arithmetic and the actual output/schema bound before each run. Both heap and inline decoders charge actual restored bytes to the common budget before reserve or writes. Malformed first tokens now fail without heap allocation. Existing per-run decoded-size checks are retained and centralized; the packed header is never treated as proof of restored length.

Adversarial tests cover variable and fixed LZ, inline/String storage, MAX+1 expansion, forged packed lengths, truncated inputs, invalid offsets and two 40000-byte strings whose combined restoration exceeds the default 65535-byte budget. Destination bytes and canaries remain unchanged on checked_into errors. Exact-limit frames still decode successfully. Invalid-frame tests invoke checked APIs only; unsafe string decoders retain their trusted-input contract. No new timing claims are made for this validation change.
