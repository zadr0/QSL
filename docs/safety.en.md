# Checked decoding and data limits

Checked deserialization is generated **only for [safety]**. Ordinary COPY materializes and validates values through one cursor before publishing a result. Native COPY/overlay validates the prepared representation before assume_init or a typed reference. Ordinary checked decoding does not perform a full pre-scan followed by another decode pass.

MAX_DATA_SIZE defaults to 65535 B and bounds input frames and aggregate useful values before allocation. Arrays charge element widths; LZ checks restored sizes. STACK_LIMIT defaults to 256 data bytes for every inline buffer. Above it String/Vec retain the wire codec.

```qsl
&Packet[safety] { u32 id; str(1..100) text; }
```

```rust
let bytes = value.serialize_checked()?;
let decoded = Packet::deserialize_checked(&bytes)?;
let mut out = std::mem::MaybeUninit::uninit();
let decoded = Packet::deserialize_checked_into(&bytes, &mut out)?;
```

No separate validate call is required before deserialize_checked. Checked_into never publishes a partial result on error. Successfully read owned temporaries use normal Drop. A standalone validator may allocate a bounded restoration buffer for large LZ strings; allocation-free validation for every schema is not promised.

serialize_checked_to validates values and destination capacity. serialize_checked uses fallible reserve: compressed strings reserve their raw upper bound while MAX_DATA_SIZE checks the actual frame. A final size error may leave caller output modified.

Trusted pointer APIs retain their unsafe requirements for complete valid frames, initialized memory and disjoint destinations. QslCustom must validate its own representations and values; arbitrary external code is not automatically proven safe.

QPS packs scalar numeric fields only. Generated prepared reads require no private tail: QUANT_PADDING_BYTES = 0. Strings, arrays and lengths retain independent codecs. ByteQps remains a legacy runtime API, not the new schema string policy.

[Full contract, errors, native COPY/LIFETIME and migration](modernization-2026-11-06/MODERNIZATION.en.md).

Trusted Serializable::serialize / BorrowSerializable::serialize trait methods are now unsafe. Callers must establish their preconditions. Generated RT/QPS serialize() methods remain safe Result APIs with value validation; use serialize_checked for general safe output.


The safety attribute does not disable unsafe reads. Native COPY/LIFETIME and region APIs always expose explicit _unchecked entry points without input validation, alongside checked methods. See the [unchecked API contract](modernization-2026-11-06/MODERNIZATION.en.md) for names and caller obligations.


An input LZ block length does not authorize a decoded size. Each run is checked against actual restored bytes, destination capacity and shared MAX_DATA_SIZE before reserve/writes. Forged lengths, MAX+1 expansion, truncated literal/offset data and invalid backreferences are rejected; unchecked APIs remain separate.
