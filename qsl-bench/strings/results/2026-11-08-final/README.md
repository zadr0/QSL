# String decoding after core extraction

Measured under background load; not accepted as strict speed evidence. Estimated average background during the run: 8.2%.

Windows 11 x86-64 · i5-7400 · 24 GiB, 2133 MHz · warm-up 15 s · measurement 10 s · 200 samples · 99% CI · one pinned core · 8 GiB limit. Unchecked decoding of prevalidated frames; validation/preparation precede timing. STACK_LIMIT=2048 is explicit. P for &str is a logical index; text bytes are not copied.

Each timed iteration decodes one string. A ring of 1024 distinct prepared frames prevents repeatedly decoding a single constant. String new includes allocation and destruction; reuse has sufficient capacity. Borrowing and copying perform different work. The 25 ns target is not met by the copying cases in this pass.

| Case | Useful B | T ns | 99% CI ns | P B/ns | Wire B |
|---|---:|---:|---:|---:|---:|
| String_new | 2048 | 100.175 | 99.324–101.335 | 20.444 | 2052 |
| String_reuse | 2048 | 52.811 | 52.523–53.176 | 38.780 | 2052 |
| StackString_new | 2048 | 52.689 | 52.368–53.110 | 38.870 | 2052 |
| StackString_reuse | 2048 | 104.654 | 103.432–106.004 | 19.569 | 2052 |
| InlineString_new | 2048 | 52.794 | 52.166–53.627 | 38.793 | 2050 |
| InlineString_reuse | 2048 | 52.676 | 52.381–53.045 | 38.880 | 2050 |
| str_view | 2048 | 6.523 | 6.465–6.590 | 313.956 | 2052 |
| String_new | 255 | 53.836 | 53.479–54.266 | 4.737 | 259 |
| String_reuse | 255 | 12.636 | 12.370–12.973 | 20.180 | 259 |

The initial process-window metric included the benchmark PID in its initial snapshot. The classification above instead subtracts benchmark user+kernel CPU from whole-window OS busy time. The rerun harness takes its other-process snapshot before starting the benchmark. No metrics sampler runs in the hot path.

Binary SHA-256: `AB3745819F9DDF58A64BC02772A55664E41686C3CA9F145E0CB10EC21649E547`.
Peak job committed memory: 15.79 MiB; peak working set: 20.00 MiB.
