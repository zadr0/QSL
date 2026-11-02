# QSL feature measurements

[Русский](features.ru.md) · [Guide](guide.en.md) · [Interactive charts](index.html#measurements)

Complete: 103 distinct cases, 125 measurement observations. The original survey has 95 main and 8 separate UTF-8/whole-record cases. Only 22 affected cases were repeated after the byte-kernel change; the other 81 retain their initial snapshot. Each JSON row identifies profile and source_manifest.

## What is measured

The primary index is P=M/T: M is original useful bytes and T is ns per operation. Wire size, capacity and object size_of do not replace M. Tables use mean wire size; min/max remain in JSON and charts. Optional-set M is averaged over uniformly represented states.

Each case: Criterion 0.8.2, 15 s warm-up, 10 s measurement, 200 samples and 99% CI. The measuring thread uses CPU mask 0x4 when permitted, otherwise the lowest allowed CPU, with above-normal thread priority. The OS is not isolated.

Windows 11 x86-64; i5-7400; RAM 24 GiB / 2133 MHz; Rust 1.98.0 / LLVM 22.1.8; release opt-level=3 with debug info, RUSTFLAGS unset. The measured process and descendants have an 8 GiB limit; compilation precedes that limit.

Peak job commit: 7.72 MiB.

## Comparison contracts

Value/frame/region preparation is outside timing. A new owned read includes construction and Drop; prepared reuse retains capacity. Borrowing creates a view and touches fewer physical bytes. Their P describes different result forms. Allocation/lease named in buffer cases is included.

Each option is compared with its baseline on the same values and states. Raw InlineString is the LZ baseline for noisy UTF-8. Validation without construction is a work component, not a complete decoder replacement. Byte QPS is compared with copying the same input bytes and exposes envelope cost.

Wire and final owned destinations in the main suite are 64 B aligned; source Vec objects retain natural alignment. Strings rotate through 16 values; optional schemas cover all states. Code-placement/cache effects are not declared an intrinsic attribute tax.

## Waveforms and intervals

T_i = sample_time_i / iterations_i; P_i = M / T_i. X is the Criterion sample number. These are batch averages, not individual-call distributions; neither operation p99 nor network latency follows from them. HTML smoothing affects only the displayed line.

For a positive time CI [T_low,T_high], the P interval is [M/T_high,M/T_low]. Frame-size plot intervals are the range of frames, not a CI. Original samples/estimates are retained.

## Arrays: count and ownership

32 useful u8 values. Fixed array<u8>(32) remains Vec, so a new decode can allocate. A borrowed byte array requires a live source frame.

### Read

M = 32 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| Vec_dynamic | 36.000 | 70.1072 | 0.4564 | 66.5445–75.2043 |
| Vec_fixed_count | 32.000 | 69.4541 | 0.4607 | 67.0060–72.2413 |
| bytes_borrow | 36.000 | 1.4763 | 21.6752 | 1.4409–1.5132 |

### Write

M = 32 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| Vec_dynamic | 36.000 | 4.4572 | 7.1795 | 4.3386–4.5837 |
| Vec_fixed_count | 32.000 | 5.6329 | 5.6809 | 5.0856–6.3520 |
| bytes_borrow | 36.000 | 4.7750 | 6.7015 | 4.6533–4.9055 |

## Output storage preparation

One 14 B frame. New SerOut includes allocation/drop; reuse excludes region creation; RT includes atomic acquisition/drop. NativeRegion creation and prefault occur before timing.

### Write

M = 14 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| RT_lease | 14.000 | 32.9021 | 0.4255 | 31.8329–34.2016 |
| allocate_SerOut | 14.000 | 85.6461 | 0.1635 | 83.7010–87.9124 |
| reuse_NativeRegion | 14.000 | 2.3305 | 6.0073 | 2.2706–2.4023 |
| reuse_SerOut | 14.000 | 3.1698 | 4.4167 | 2.9714–3.4313 |

## Custom codec

One u16 versus an equal-width QslCustom. Timed code is trusted; frames are checked beforehand. Custom methods and their validator remain implementation obligations.

### Read

M = 2 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| QslCustom | 2.000 | 3.1072 | 0.6437 | 3.0501–3.1710 |
| primitive | 2.000 | 3.2681 | 0.6120 | 3.1581–3.3939 |

### Write

M = 2 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| QslCustom | 2.000 | 4.2342 | 0.4723 | 4.0950–4.3830 |
| primitive | 2.000 | 3.0832 | 0.6487 | 2.9862–3.1940 |

## Inline policy

Same three-optional schema. The attribute affects calls/code size, not wire format. Actual results depend on rustc/LLVM and the call site.

### Read

M = 7.5 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| always | 8.500 | 15.0969 | 0.4968 | 14.5291–15.8301 |
| compiler_default | 8.500 | 15.7912 | 0.4749 | 15.4864–16.1857 |
| hint | 8.500 | 14.7816 | 0.5074 | 14.5397–15.0561 |
| never | 8.500 | 16.2061 | 0.4628 | 15.8017–16.6859 |

### Write

M = 7.5 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| always | 8.500 | 19.9288 | 0.3763 | 19.7625–20.1149 |
| compiler_default | 8.500 | 21.8614 | 0.3431 | 21.2894–22.7664 |
| hint | 8.500 | 23.3840 | 0.3207 | 22.8336–23.9595 |
| never | 8.500 | 21.7593 | 0.3447 | 21.0619–22.5535 |

## Layout and bit packing

Three equal-domain integers: M=4 B. compact uses 18 bits; alignment completes fields at byte boundaries. Native objects do not shrink with wire bytes.

### Read

M = 4 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| compact | 3.000 | 4.1328 | 0.9679 | 4.0036–4.2767 |
| compact_alignment | 4.000 | 3.2365 | 1.2359 | 3.1806–3.3031 |
| default | 4.000 | 3.0100 | 1.3289 | 2.9214–3.1025 |

### Write

M = 4 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| compact | 3.000 | 3.9388 | 1.0155 | 3.6939–4.3067 |
| compact_alignment | 4.000 | 3.7706 | 1.0609 | 3.6928–3.8667 |
| default | 4.000 | 4.1410 | 0.9659 | 4.0178–4.2727 |

## Rust structure expansion

Equal u32/u64/u16 fields. Expansion happens at build time; no nested message object is introduced. Tiny timing differences are not an inherent macro tax.

### Read

M = 14 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| expanded_Rust | 14.000 | 4.2596 | 3.2867 | 4.0802–4.4659 |
| manual | 14.000 | 3.5695 | 3.9222 | 3.4979–3.6571 |

### Write

M = 14 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| expanded_Rust | 14.000 | 3.4278 | 4.0843 | 3.3727–3.4903 |
| manual | 14.000 | 4.5338 | 3.0879 | 4.3020–4.8095 |

## Optionals: unique subset sums

One required u32 plus optional u8/u16/u32; all eight states. Mean M=7.5 B. Sizes 1/2/4 have unique subset sums; this differs from sparse equal-width fields in the preceding N control.

### Read

M = 7.5 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| bitmap | 11.500 | 4.5401 | 1.6520 | 4.4662–4.6330 |
| default | 8.500 | 14.6592 | 0.5116 | 14.3436–15.0510 |
| opt_table | 7.500 | 6.1250 | 1.2245 | 6.0313–6.2418 |
| opt_table_predict_seq | 7.500 | 6.2713 | 1.1959 | 6.1043–6.5096 |
| predict_seq | 7.500 | 6.9019 | 1.0867 | 6.6429–7.2085 |

### Write

M = 7.5 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| bitmap | 11.500 | 12.8241 | 0.5848 | 12.3880–13.4724 |
| default | 8.500 | 22.2680 | 0.3368 | 21.7334–23.0926 |
| opt_table | 7.500 | 6.4137 | 1.1694 | 6.1981–6.6937 |
| opt_table_predict_seq | 7.500 | 6.2335 | 1.2032 | 6.1049–6.3811 |
| predict_seq | 7.500 | 6.1852 | 1.2126 | 6.1004–6.2762 |

## General QPS: full byte range

The sequence covers 0..255; it is not cryptographic noise. This block codec can choose raw fallback with an explicit header cost; exactness remains mandatory.

### Read

M = 1024 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| Q16 | 1040.000 | 18.8961 | 54.1911 | 18.7532–19.0690 |
| raw_copy | 1024.000 | 18.0478 | 56.7383 | 17.8845–18.2295 |

### Write

M = 1024 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| Q16 | 1040.000 | 217.0342 | 4.7182 | 215.4037–218.9293 |
| raw_copy | 1024.000 | 16.6732 | 61.4159 | 16.5623–16.7936 |

## General QPS: 1024 bytes in 0..15

Q2/Q4/Q8/Q16; the 16 B header is included. Raw copy has no envelope. QPS time includes every pass and check, not just its arithmetic kernel.

### Read

M = 1024 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| Q16 | 592.000 | 258.0053 | 3.9689 | 256.4726–259.6755 |
| Q2 | 1040.000 | 19.4553 | 52.6333 | 19.3231–19.5999 |
| Q4 | 720.000 | 876.4771 | 1.1683 | 870.2937–883.4013 |
| Q8 | 592.000 | 456.7358 | 2.2420 | 453.7363–460.0314 |
| raw_copy | 1024.000 | 16.6808 | 61.3881 | 16.5235–16.8624 |

### Write

M = 1024 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| Q16 | 592.000 | 440.0472 | 2.3270 | 436.8138–443.5965 |
| Q2 | 1040.000 | 925.2369 | 1.1067 | 916.4309–935.3020 |
| Q4 | 720.000 | 1603.6457 | 0.6385 | 1590.4139–1619.3920 |
| Q8 | 592.000 | 832.0875 | 1.2306 | 826.5848–838.3821 |
| raw_copy | 1024.000 | 16.9716 | 60.3360 | 16.7468–17.2561 |

## QPS: UTF-8 without field prefixes

2048 B of UTF-8 alone: raw copy versus ByteQps Q16. The base is computed from text; original length stays in the explicit outer header.

### Read

M = 2048 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| Q16 | 1936.000 | 571.3408 | 3.5846 | 568.1478–574.9159 |
| raw_copy | 2048.000 | 27.8833 | 73.4489 | 27.6462–28.1412 |

### Write

M = 2048 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| Q16 | 1936.000 | 904.6144 | 2.2639 | 898.0614–912.5338 |
| raw_copy | 2048.000 | 28.0085 | 73.1207 | 27.7270–28.3436 |

## QPS: UTF-8 inside a whole record

u32, a UTF-8 string and a 32-byte array. Default checked APIs versus general QPS, including checks and restoration. Inner zero bytes and lengths can change the base and remove savings.

### Read

M = 2084 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| checked_default | 2092.000 | 1863.3718 | 1.1184 | 1806.5972–1940.2019 |
| Q16 | 2108.000 | 1800.4252 | 1.1575 | 1785.2283–1821.2796 |

### Write

M = 2084 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| checked_default | 2092.000 | 31.2032 | 66.7881 | 30.9964–31.4214 |
| Q16 | 2108.000 | 457.3166 | 4.5570 | 454.6054–460.1915 |

## Strings: 255 ASCII bytes

255 characters = 255 B. Schema MAX is 4096. New String/CopyGuard includes allocation/drop; reuse has prepared capacity. Borrowing has a different result contract.

### Read

M = 255 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| CopyGuard_new | 259.000 | 78.9262 | 3.2309 | 76.7463–81.7462 |
| CopyGuard_reuse | 259.000 | 9.0782 | 28.0892 | 8.8931–9.2874 |
| InlineString | 257.000 | 9.8084 | 25.9982 | 9.5890–10.0833 |
| String_new | 259.000 | 76.3834 | 3.3384 | 73.6953–79.4684 |
| String_reuse | 259.000 | 10.0320 | 25.4187 | 9.7984–10.2970 |
| bytes_borrow | 259.000 | 1.5578 | 163.6905 | 1.5066–1.6206 |
| str_borrow | 259.000 | 1.4975 | 170.2850 | 1.4675–1.5311 |
| string_LZ | 257.000 | 12.3412 | 20.6625 | 11.8625–12.9534 |
| view_borrow | 259.000 | 1.5034 | 169.6130 | 1.4714–1.5399 |

### Write

M = 255 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| CopyGuard_new | 259.000 | 9.6301 | 26.4796 | 9.3764–9.9416 |
| InlineString | 257.000 | 13.3248 | 19.1373 | 12.9133–13.8490 |
| String_new | 259.000 | 9.1957 | 27.7304 | 8.9599–9.4811 |
| bytes_borrow | 259.000 | 9.4668 | 26.9362 | 9.2333–9.7486 |
| str_borrow | 259.000 | 9.4515 | 26.9798 | 9.2500–9.6832 |
| string_LZ | 257.000 | 13.9049 | 18.3389 | 13.5923–14.2743 |
| view_borrow | 259.000 | 10.0823 | 25.2918 | 9.8172–10.3822 |

## Strings: 1024 UTF-8 scalars, noise

2048 B of pseudorandom UTF-8. A separate raw InlineString/LZ pair exposes compression-attempt and fallback cost. Not pooled with repetitive input.

### Read

M = 2048 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| InlineString | 2050.000 | 42.2299 | 48.4965 | 41.4060–43.1522 |
| string_LZ | 2050.000 | 44.8378 | 45.6758 | 43.3429–46.7553 |

### Write

M = 2048 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| InlineString | 2050.000 | 51.6763 | 39.6313 | 49.3339–54.5520 |
| string_LZ | 2050.000 | 53.4001 | 38.3520 | 50.5700–56.9738 |

## Strings: 1024 UTF-8 scalars, repeats

1024 two-byte scalars = 2048 B. Repetition does not suit every compressor. All variants receive the same values, including LZ, ownership and reuse.

### Read

M = 2048 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| CopyGuard_new | 2052.000 | 135.0154 | 15.1686 | 128.7680–142.6728 |
| CopyGuard_reuse | 2052.000 | 48.6836 | 42.0675 | 47.1371–50.4633 |
| InlineString | 2050.000 | 40.5403 | 50.5177 | 39.1576–42.2985 |
| String_new | 2052.000 | 102.5873 | 19.9635 | 99.0760–107.5682 |
| String_reuse | 2052.000 | 47.7050 | 42.9305 | 46.7305–48.9487 |
| bytes_borrow | 2052.000 | 1.5983 | 1281.3816 | 1.5429–1.6625 |
| str_borrow | 2052.000 | 1.5938 | 1284.9808 | 1.5461–1.6489 |
| string_LZ | 67.000 | 315.6066 | 6.4891 | 304.0331–329.0468 |
| view_borrow | 2052.000 | 1.5199 | 1347.4611 | 1.4879–1.5561 |

### Write

M = 2048 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| CopyGuard_new | 2052.000 | 46.4950 | 44.0477 | 45.2699–47.8620 |
| InlineString | 2050.000 | 51.2247 | 39.9807 | 49.1185–53.7917 |
| String_new | 2052.000 | 39.9749 | 51.2321 | 39.1508–40.9978 |
| bytes_borrow | 2052.000 | 40.8558 | 50.1275 | 39.9287–41.8721 |
| str_borrow | 2052.000 | 39.3467 | 52.0501 | 38.5355–40.2377 |
| string_LZ | 67.000 | 157.0205 | 13.0429 | 153.9566–161.3731 |
| view_borrow | 2052.000 | 40.6908 | 50.3308 | 39.7737–41.8178 |

## Validation cost

Trusted decode, validation without construction and checked decode. validate_only does not complete the decoder task; this compares work components, not interchangeable results.

### Read

M = 4 B.

| Mode | Wire, B | T, ns | P, B/ns | CI 99%, ns |
|---|---:|---:|---:|---:|
| checked_into | 4.000 | 54.2311 | 0.0738 | 52.1893–56.4766 |
| trusted | 4.000 | 1.9115 | 2.0926 | 1.8776–1.9459 |
| validate_only | 4.000 | 45.8246 | 0.0873 | 45.0274–46.7068 |

## Coverage boundaries

IPC and network transport are covered by the full guide and separate functional tests. Their system latency is not inferred from codec-only T. Earlier N=1/2/3/5/10 and scalar-QPS controls remain separate historical groups in the interactive page, with their own source snapshots and CPU policy.

## Register kernel: result and limit

| Case | Before, ns | After, ns | Before/after | Wire, B |
|---|---:|---:|---:|---:|
| qps_bytes/low_range_1024/write/Q16 | 1226.55 | 440.05 | 2.79 | 592 |
| qps_bytes/low_range_1024/read/Q16 | 2089.77 | 258.01 | 8.10 | 592 |
| qps_utf8/bytes/write/Q16 | 2282.74 | 904.61 | 2.52 | 1936 |
| qps_utf8/bytes/read/Q16 | 4464.94 | 571.34 | 7.81 | 1936 |

Wire format and data match; executables differ. Runs were sequential at different times, so the ratio cannot exclude OS/cache-state effects. Initial-kernel waveforms remain separate groups.

General QPS does not meet the 20–30% overhead budget. For 1024 UTF-8 characters (2048 B), prepared String decoding takes 47.71 ns, InlineString 40.54 ns and LZ 315.61 ns with a 67 B frame. The 25 ns target is also unmet. Borrowed views have another contract and do not prove copying in 1–2 ns. General QPS produces 1936 B for pure UTF-8; adding headers/numbers to the whole record instead yields raw 2108 B for a 2092 B inner frame. Neither string types nor grouping u8 guarantees compression.

## Commands and reproducibility

```powershell
.\qsl-bench\tools\run-payload.ps1 -Benchmark current_qsl -ManifestPath .\qsl-bench\features\Cargo.toml -ResultDirectory .\qsl-bench\target\criterion-features-repeat
.\qsl-bench\tools\run-qps-kernel.ps1 -Benchmark current_qsl -ManifestPath .\qsl-bench\features\Cargo.toml -ResultDirectory .\qsl-bench\target\criterion-qps-kernel-repeat
.\qsl-bench\tools\run-qps-utf8.ps1 -ResultDirectory .\qsl-bench\target\criterion-qps-utf8-repeat
cargo test --manifest-path qsl-bench/features/Cargo.toml --locked
cargo test --manifest-path qsl-bench/features/Cargo.toml --bench current_qsl --locked -- --test
```

[JSON](data/features.json) · [CSV](data/features.csv) · [Raw samples](data/features-raw/) · [Initial SHA-256](../reference/features-source-manifest.json) · [QPS SHA-256](../reference/qps-words-source-manifest.json) · [UTF-8 SHA-256](../reference/qps-utf8-words-source-manifest.json) · [Harness](../reference/features/)
