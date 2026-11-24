# Safety QSL

On each row, checked and trusted use the same generated type, the same wire format, the same bytes, and the same ownership contract. The trusted path reads immutable frames already accepted by QSL's validator and writes values already accepted by the checked writer. The destination for every normal write is the same kind of prepared caller buffer.

[QSL All Features](QSL-ALL-FEATURES.md) · [Interactive tables](qsl-matrix.html#safety) · [Safety CSV](results/qsl-matrix/safety.csv)

## Measured costs

All durations are in nanoseconds per record. The ratio divides checked read time by trusted read time from the same profile. ASCII and multibyte UTF-8 have separate rows. Displayed medians use the two confirmation passes described in [QSL All Features](QSL-ALL-FEATURES.md).

<!-- EXAMPLE -->

<!-- SAFETY -->

## Where time goes

- **Checked read** follows the generated validator and materializes the result. Depending on the schema it checks bounds, frame completion, integer domains, presence metadata, and UTF-8 or the restored size. The code can fuse validation and materialization.
- **Trusted read** uses the generated parser for a schema-valid frame. For borrowed text it can create the UTF-8 view without scanning its contents again. Owned text still needs storage for the characters, so ownership changes the baseline as well as the ratio.
- **Validate only** runs the same schema's validator and returns unit. **Validate + trusted read** times both operations together. This is the direct measurement for validating a frame and then taking the trusted path once. Its time can differ from checked read because it performs two traversals rather than the fused checked path.
- **Checked write** can call `validate_values`, compute the encoded size and check destination capacity before writing. **Validate values** measures the source validation separately. **Trusted write** receives a value already validated and sufficient initialized output space.
- **Dyn** parses its header and tags or index metadata. Width, mode and optional density can change the work done by checked and trusted paths. The scratch array belongs to the checked operation and is included in its time.

For an input boundary that accepts untrusted frames, use the checked row or the measured validation-and-read sequence. Trusted times apply after a complete validation and only while the frame retains the same contents and required alignment.

The [confirmed timings](results/qsl-matrix/confirmed.csv) contain the two 99% bootstrap intervals for every operation. Small differences can overlap. These measurements compare complete generated paths, so the checked-to-trusted ratio includes any differences in parsing and result construction within those paths.
