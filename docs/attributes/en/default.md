# default · QSL 0.2.0

Field modifier |default=7| controls generated initialization within the declared numeric domain. It does not add an optional-presence bit, authorize invalid values or change explicitly supplied wire data. It is excluded from schema identity. Default generation is explicit, not an unconditional unwrap_or_default in the hot path.

[Full guide](../../guide.en.md) · [Safety](../../safety.en.md) · [Migration](../../migration-0.2.0.en.md) · [All attributes](index.md).
