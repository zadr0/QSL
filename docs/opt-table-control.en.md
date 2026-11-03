# Control: default vs opt_table

> Historical profile: `opt_table` here labels the former mixed length graph. `[predict_seq,opt_table]` preserves its format; new standalone `[opt_table]` dispatches readers by tag. [Separation and migration](optional-modes.en.md).

[Русский](opt-table-control.ru.md) · [QPS measurements](performance.en.md)

One required u32 plus N optional u64 fields. N = 1, 2, 3, 5, 10; each pair uses identical logical values, field order and explicit inline/safety policy. QPS is off. This separates the table effect from compression. The earlier QPS profile has a different shape and working set; do not compare its absolute times against this group.

In this run opt_table reads slower at N=1 and N=5, faster at N=2, N=3 and N=10. It writes faster at all five sizes. This is why opt_table remains an explicit schema choice, not a blanket speed recommendation.

## deserialize_into

| N | default, ns | opt_table, ns | Change |
|---:|---:|---:|---:|
| 1 | 3.5515 | 3.9202 | +10.4% |
| 2 | 8.7726 | 6.3678 | -27.4% |
| 3 | 12.7686 | 8.7599 | -31.4% |
| 5 | 14.7990 | 19.2911 | +30.4% |
| 10 | 45.8993 | 34.9577 | -23.8% |

Positive changes mean the table took longer. Negative changes mean shorter time. Close estimates must be read with their CI; this is not a universal selection threshold.

## serialize_into

| N | default, ns | opt_table, ns | Change |
|---:|---:|---:|---:|
| 1 | 5.1046 | 3.4460 | -32.5% |
| 2 | 11.2394 | 5.0700 | -54.9% |
| 3 | 20.4810 | 6.3964 | -68.8% |
| 5 | 36.3314 | 9.2933 | -74.4% |
| 10 | 134.6590 | 18.9646 | -85.9% |

Positive changes mean the table took longer. Negative changes mean shorter time. Close estimates must be read with their CI; this is not a universal selection threshold.

## Frame sizes, B

| N | default | opt_table |
|---:|---:|---:|
| 1 | 5–13 | 4–12 |
| 2 | 5–21 | 5–21 |
| 3 | 5–29 | 5–29 |
| 5 | 5–47 | 5–45 |
| 10 | 5–90 | 6–86 |

These are different wire codecs for identical logical fields. Metadata can vary with state in default; opt_table can use a fixed presence span. An empty 10-optional frame is larger with opt_table even though its maximum frame is smaller. Savings and timing are therefore not inferred from one maximum size.

## Conditions

For every size, 1024 cyclic frames use masks `(i*405+17)&1023`, projected to N bits. All 2^N combinations occur equally often; ID and values vary too. Both wire formats are validated and logical outputs compared through both checked and prepared decoding before timing. With equal-width optionals, payload length gives the count but usually cannot identify a sparse subset; this is deliberately not an always-deterministic best case.

Each case: 15 s warm-up, 10 s measurement, 200 samples, 99% CI. Prepared serialize_into / deserialize_into; allocation, validation and setup excluded. Frame selection and black_box included. Wire capacity 128 B and final destinations are aligned to 64 B; native source objects have natural Vec alignment. Compilation precedes the measured-process 8 GiB Job Object limit.

Peak job commit: 7.89 MiB. Windows 11 x86-64; i5-7400; RAM 24 GiB / 2133 MHz; Rust 1.98 / LLVM 22.1.8; Criterion 0.8.2. Affinity and OS isolation were not configured. Source hashes were fixed before timing and checked after it.

## CI 99%

| N | Operation | default | opt_table |
|---:|---|---:|---:|
| 1 | deserialize_into | 3.4243–3.7207 | 3.8739–3.9704 |
| 1 | serialize_into | 5.0185–5.2088 | 3.4140–3.4806 |
| 2 | deserialize_into | 8.4237–9.1893 | 6.2977–6.4450 |
| 2 | serialize_into | 11.1175–11.3655 | 4.9026–5.3359 |
| 3 | deserialize_into | 12.4884–13.1205 | 8.3262–9.3746 |
| 3 | serialize_into | 20.2144–20.7674 | 6.3041–6.4949 |
| 5 | deserialize_into | 14.5651–15.0854 | 18.6471–20.0981 |
| 5 | serialize_into | 35.9205–36.7678 | 9.1527–9.5258 |
| 10 | deserialize_into | 44.9150–47.0633 | 34.5444–35.4054 |
| 10 | serialize_into | 130.5244–141.3193 | 18.5154–19.5237 |

```powershell
.\qsl-bench\tools\run-payload.ps1 -Benchmark current_qsl -ManifestPath .\qsl-bench\opt-table-control\Cargo.toml -ResultDirectory .\qsl-bench\target\criterion-opt-table-repeat
cargo test --manifest-path qsl-bench/opt-table-control/Cargo.toml --bench current_qsl --locked -- --test
```

[JSON](data/opt-table-control.json) · [CSV](data/opt-table-control.csv) · [Raw](data/opt-table-control-raw/) · [Log](data/opt-table-control-run.log) · [Harness](../reference/opt-table-control/) · [Hashes](../reference/opt-table-source-manifest.json)
