# Focused UTF-8 decoding benchmark

From the repository root on the Windows stand:

```powershell
./qsl-bench/strings/run.ps1 -Output ./qsl-bench/strings/results/local
```

The runner builds only the strings target, waits for three consecutive 2-second windows at <=5% mean background CPU, then pins the process to the least-busy nonzero logical core selected before timing and applies an 8 GiB job limit. Criterion performs 15 s warm-up and 10 s measurement per case, 200 samples, 99% CI. Metrics are collected outside the hot path. No k6 or broad suite is started. A failed idle gate does not start measurement; end-of-run OS noise is reported separately.

Nine cases cover String new/reuse, StackString new/reuse, InlineString new/reuse and &str for 1024 Cyrillic scalars (2048 useful bytes), plus String new/reuse for 255 ASCII scalars. STACK_LIMIT=2048 is explicitly configured for this experiment; the normal default is 256 B. Validation and buffer preparation precede timing. String new includes allocation and Drop; &str does not copy the payload, so its P=M/T is a logical index rather than memory bandwidth.

Results retain raw Criterion samples, 99% intervals, the executable SHA-256 and the measured-source archive. Read each run's OS-noise status before comparing it. A passed pre-start gate alone does not certify the complete run. All three recorded runs remain available; the final run is used by the website. The 25 ns target is not reached by the copying cases.
