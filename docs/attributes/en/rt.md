# rt · QSL 0.2.0

DEQ: !config QSL_RT=true; !config QSL_RT_DEFAULT=16;. Only schemas with a finite proven output bound receive static slots; total capacity per schema is capped at 64 MiB. serialize() validates values and waits for a free lease without allocating output. Do not hold every slot while waiting for another from the same pool. StaticRegions::try_acquire is the nonblocking pool API and returns PoolExhausted. Input String/Vec construction may still allocate.

[Full guide](../../guide.en.md) · [Safety](../../safety.en.md) · [Migration](../../migration-0.2.0.en.md) · [All attributes](index.md).
