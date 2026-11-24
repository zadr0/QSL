# Historical benchmark series

These runs remain available so their inputs and timing contracts can be audited. The [current report](OPTIMIZED.md) uses a prepared, warmed encoding contract and independently selected codec profiles.

## Validation in the earlier comparisons

The first API and socket benchmarks measured each adapter as implemented. QSL ran checked serialization and deserialization on the envelope and its nested QSL frames. Its schema enforced limits such as a 192-byte string maximum and a 32-element limit for readings. The other adapters ran their codec's parsing checks and their own frame or traversal checks. The shared application logic checked the frame size and payload presence, and the HTTP endpoint also checked the payload variant and revision increment. Field-level limits differed by adapter. The timings include those differences, along with the extra work of building the nested QSL frames. [QSL schema](realistic/schemas/telemetry.qsl) · [QSL adapter](realistic/src/qsl_codec.rs) · [Shared API adapters](realistic/src/lib.rs)

The earlier scalar suite did add equivalent domain checks for specific cases. For example, the Protobuf, Bincode and Cap’n Proto numeric adapters explicitly enforced the same `0..=4095` range that QSL put in the schema. Treat this as a comparison of the particular contracts used by the adapters. [Scalar adapters](src/codecs.rs) · [Scalar QSL schemas](schemas/records.qsl)

| Series | Scope | Evidence |
|---|---|---|
| Four-codec scalar comparison | Four bounded integers, optional fields and UTF-8 strings. The CSV separates owned and borrowed reads. | [Source statistics](results/released/measurements.csv) |
| QSL Dyn | All seven supported combinations of line, dynamic index and strict index widths. Indexed compact cases and borrowed strings are separate. | [Source statistics](results/dyn/measurements.csv) |
| Native layout | Declared placement, `auto`, `auto[wire=1]` and a hand-ordered layout. Native field offsets and frame offsets are recorded. | [Layouts](results/layout/profiles.json) · [Timing](results/layout/measurements.csv) |
| Features | Checked transmutation copy and overlay, String, StackString and QSL-LZ for periodic, log and noisy text. | [Native reads](results/qsl-features/native.csv) · [String workloads](results/qsl-features/strings.csv) |
| First k6 series | HTTP encoding and decoding on three small structures, including all Dyn modes. | [k6 statistics](results/k6/measurements.csv) |
| First API and socket series | Same logical API and event fixtures. The earlier QSL adapter serialized variable children into nested frames, and the encode timer included the preparation and output allocations. | [Codec](results/realistic/codec.csv) · [HTTP](results/realistic/http.csv) · [TCP](results/realistic/socket.csv) |

The [earlier report snapshot](https://github.com/zadr0/QSL/tree/f950b52522d6102aa9c5bbae08aa92b76ac089ef/benchmarks/qsl-v040-comparison) preserves the historical narratives and dashboard.
