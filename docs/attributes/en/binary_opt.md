# binary_opt · QSL 0.2.0

String/str field modifier: true selects UTF-8 bytes, view selects &str, inline selects a dedicated bounded string header, guard retains COPY compatibility syntax. Current COPY lowering uses String/StackString according to bounds and STACK_LIMIT; legacy guard wrappers remain runtime APIs. UTF-8 validation is retained. Fixed strings use [u8;N] within the strict limit, or a documented owned fallback above it.

[Full guide](../../guide.en.md) · [Safety](../../safety.en.md) · [Migration](../../migration-0.2.0.en.md) · [All attributes](index.md).
