# lifetime · QSL 0.2.0

Borrow supported raw strings/bytes from immutable input. The input must outlive the result. Fixed native regions can use validated aligned views; whole-record transmutate+lifetime has stricter fixed-layout constraints. Compressed strings need owned restoration and cannot directly borrow compressed bytes.

[Full guide](../../guide.en.md) · [Safety](../../safety.en.md) · [Migration](../../migration-0.2.0.en.md) · [All attributes](index.md).
