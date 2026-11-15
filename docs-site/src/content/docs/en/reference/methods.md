---
title: "Serialization and decoding methods"
description: "QSL documentation: Serialization and decoding methods."
---

Generated methods vary by schema/mode; dyn/lifetime/QPS/native signatures differ. Generated Rust is authoritative.
Method | Contract
---|---
validate_values / validate(input) | Value budget/domain or available checked input validation
serialize_checked | Validated fallible output: `StackOut<N>` or `Vec<u8>`, selected by the schema bound.
serialize_checked_to | Caller slice, validation/capacity checks, exact written length
deserialize_checked | Checked materialization with safety
deserialize_checked_into | New MaybeUninit destination; dyn adds scratch
serialized_size | Required capacity; compressed strings may use a raw upper bound
serialize_into_unsafe | Caller guarantees values, capacity and nonoverlap
deserialize_from_raw_parts | Trusted ordinary length-aware record
deserialize_into/reuse/prepared_into | Unsafe destination/reuse contracts
comp_size/comp_write/comp_read | Compatibility COMPACT helpers, when emitted
Prepared unsafe reuse requires sufficient capacity before the call and preserves eligible String, Vec and boxed InlineString storage. It does not grow these buffers. An initialized mutable object and an uninitialized destination have separate contracts.

Serializable trusted methods remain unsafe. Callers establish valid schema data, complete input, sufficient storage, alignment and lifetimes.

Checked cursors validate extents, overflow, domains/spare codes, UTF-8, presence, padding, restoration and aggregate budgets before publication. Aggregate checks only cover proven accessible islands. Unsafe callers guarantee complete valid frames, schema match, capacity/initialization, alignment, lifetime and nonoverlap; prepared QPS adds private storage requirements.
Error | Meaning
---|---
UnexpectedEof / BufferTooSmall | Insufficient input / destination or scratch
LengthOverflow | Length/offset arithmetic overflow
InvalidWidth / InvalidSchema | Unsupported width/configuration
InvalidValue | Domain, spare code or structural value error
InvalidUtf8 / InvalidPadding | Text/padding error
SizeLimitExceeded | Frame/restoration/value budget exceeded
AllocationFailed | Fallible allocation failed
PoolExhausted | Nonblocking pool acquisition failed
Custom codec correctness also depends on its own validator/implementation. A writer error does not promise transactional restoration of already changed output; failed output must not be published as a frame.

The generator selects the output type from the proven maximum frame size. If it is at most STACK_LIMIT, including equality, ordinary serialization returns `StackOut<N>`; a larger or unknown bound selects alloc::`Vec<u8>`. Transmitted headers count toward this threshold; the output object length and alignment metadata do not.
