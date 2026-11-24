# Safety QSL

On each row, checked and trusted use the same generated type, the same wire format, the same bytes, and the same ownership contract. The trusted path reads immutable frames already accepted by QSL's validator and writes values already accepted by the checked writer. The destination for every normal write is the same kind of prepared caller buffer.

[QSL All Features](QSL-ALL-FEATURES.md) · [Interactive tables](qsl-matrix.html#safety) · [Safety CSV](results/qsl-matrix/safety.csv)

## Measured costs

All durations are in nanoseconds per record. The ratio divides checked read time by trusted read time from the same profile. ASCII and multibyte UTF-8 have separate rows. Displayed medians use the two confirmation passes described in [QSL All Features](QSL-ALL-FEATURES.md).

At 32 ASCII bytes, checked borrowed read took **26.79 ns** and trusted borrowed read took **2.86 ns**. At 8,192 ASCII bytes the corresponding times were **387.11 ns** and **2.73 ns**. Validation alone took **363.87 ns** at this length. The 8,192-byte multibyte corpus took **7,324.83 ns** to validate. Owning the 8,192-byte ASCII string raised trusted time to **598.02 ns** for the owned profile.

| Workload | Profile | Checked read | Trusted read | Validate only | Validate + trusted read | Checked write | Trusted write | Validate values | Read ratio |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| domain | `auto` | 5.04 | 6.76 | 4.65 | 8.47 | 3.91 | 3.04 | 2.59 | 0.74× |
| domain | `compact` | 4.86 | 7.02 | 3.35 | 8.94 | 4.83 | 3.26 | 2.57 | 0.69× |
| dyn | `index_dynamic_32` | 120.64 | 49.19 | 120.76 | 165.53 | 10.13 | 7.16 | 1.9 | 2.45× |
| dyn | `index_dynamic_64` | 109.08 | 47.46 | 105.98 | 160.62 | 9.14 | 6.88 | 1.86 | 2.3× |
| dyn | `index_strict_32` | 96.69 | 40.55 | 95.51 | 136.14 | 11.07 | 9.92 | 1.88 | 2.38× |
| dyn | `index_strict_64` | 81.28 | 37.54 | 77.07 | 112.91 | 12.16 | 9.78 | 1.89 | 2.17× |
| dyn | `line_16` | 50.13 | 7.99 | 46.5 | 50.66 | 5.57 | 4.44 | 1.86 | 6.28× |
| dyn | `line_32` | 48.25 | 8.05 | 44.61 | 50.37 | 5.61 | 4.51 | 1.88 | 5.99× |
| dyn | `line_64` | 35.26 | 7.8 | 32.97 | 37.4 | 5.96 | 4.75 | 1.9 | 4.52× |
| fixed | `auto` | 4.63 | 2.82 | 3.9 | 4.56 | 3.24 | 2.99 | 2.26 | 1.64× |
| fixed | `compact_group_native` | 53.4 | 3.41 | 45.27 | 49.39 | 3.78 | 3.34 | 2.27 | 15.67× |
| fixed | `compact` | 52.61 | 3.71 | 46.63 | 47.41 | 4.39 | 3.53 | 2.27 | 14.18× |
| noise_1024 | `qsl_lz_inline` | 107.72 | 35.29 | 76.29 | 102.33 | 22.38 | 21.07 | 2.75 | 3.05× |
| periodic_1024 | `qsl_lz_inline` | 2,005.55 | 134.06 | 1,981.39 | 2,007.41 | 78 | 71.06 | 2.74 | 14.96× |
| presence | `auto` | 59.38 | 7.44 | 51.87 | 57.83 | 11.38 | 10.12 | 2.29 | 7.98× |
| presence | `bitmap` | 26.26 | 5.97 | 20.33 | 25.68 | 5.59 | 4.48 | 2.17 | 4.4× |
| presence | `ps_ot` | 34.81 | 4.35 | 25.88 | 29.41 | 9.4 | 7.32 | 2.2 | 8× |
| text_1024 | `borrowed_aligned` | 82.13 | 2.84 | 77.74 | 81.28 | 23.33 | 23.05 | 2.19 | 28.97× |
| text_1024 | `borrowed` | 68.24 | 2.88 | 77.62 | 69.48 | 26.4 | 25.83 | 2.48 | 23.72× |
| text_1024 | `owned` | 159.38 | 78.55 | 76.42 | 133.43 | 26.53 | 26.44 | 1.9 | 2.03× |
| text_256 | `borrowed` | 37.04 | 2.96 | 34.06 | 35.05 | 12.33 | 12.94 | 2.49 | 12.51× |
| text_32 | `borrowed` | 26.79 | 2.86 | 23.73 | 25.02 | 5.58 | 5.42 | 2.53 | 9.36× |
| text_32 | `owned` | 97.87 | 53.93 | 24.54 | 76.49 | 5.24 | 5.04 | 1.98 | 1.81× |
| text_4096 | `borrowed` | 201.97 | 2.98 | 202.67 | 198.38 | 76.64 | 70.44 | 2.56 | 67.76× |
| text_8192 | `borrowed_aligned` | 392.71 | 2.85 | 380.92 | 394.68 | 201.09 | 203.36 | 2.17 | 137.63× |
| text_8192 | `borrowed` | 387.11 | 2.73 | 363.87 | 381.63 | 198.03 | 205.55 | 2.54 | 142.03× |
| text_8192 | `owned` | 972.28 | 598.02 | 360.29 | 954.46 | 208.67 | 203.21 | 2 | 1.63× |
| utf8_1024 | `borrowed` | 949.95 | 2.78 | 923.72 | 925.66 | 24.55 | 24.14 | 2.54 | 342.15× |
| utf8_8192 | `borrowed` | 7,239.09 | 2.87 | 7,324.83 | 7,194.76 | 195.81 | 198.57 | 2.57 | 2,521.53× |
| utf8_8192 | `owned` | 7,555.37 | 457.26 | 7,128.95 | 7,855.87 | 204.16 | 201.93 | 1.99 | 16.52× |

## Where time goes

- **Checked read** follows the generated validator and materializes the result. Depending on the schema it checks bounds, frame completion, integer domains, presence metadata, and UTF-8 or the restored size. The code can fuse validation and materialization.
- **Trusted read** uses the generated parser for a schema-valid frame. For borrowed text it can create the UTF-8 view without scanning its contents again. Owned text still needs storage for the characters, so ownership changes the baseline as well as the ratio.
- **Validate only** runs the same schema's validator and returns unit. **Validate + trusted read** times both operations together. This is the direct measurement for validating a frame and then taking the trusted path once. Its time can differ from checked read because it performs two traversals rather than the fused checked path.
- **Checked write** can call `validate_values`, compute the encoded size and check destination capacity before writing. **Validate values** measures the source validation separately. **Trusted write** receives a value already validated and sufficient initialized output space.
- **Dyn** parses its header and tags or index metadata. Width, mode and optional density can change the work done by checked and trusted paths. The scratch array belongs to the checked operation and is included in its time.

For an input boundary that accepts untrusted frames, use the checked row or the measured validation-and-read sequence. Trusted times apply after a complete validation and only while the frame retains the same contents and required alignment.

The [confirmed timings](results/qsl-matrix/confirmed.csv) contain the two 99% bootstrap intervals for every operation. Small differences can overlap. These measurements compare complete generated paths, so the checked-to-trusted ratio includes any differences in parsing and result construction within those paths.
