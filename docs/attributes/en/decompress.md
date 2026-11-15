# decompress · QSL 0.2.0

Field modifier |decompress=true| selects independent bounded QSL-LZ, not QPS. Store raw when compression is unhelpful. Checks bound each restored run by actual output size, capacity and MAX_DATA_SIZE; a forged encoded length cannot authorize expansion. The codec is lossless and linear with bounded match work, but no universal speed/compression guarantee is claimed. Above STACK_LIMIT, COPY restoration uses String.

[Full guide](../../guide.en.md) · [Safety](../../safety.en.md) · [Migration](../../migration-0.2.0.en.md) · [All attributes](index.md).
