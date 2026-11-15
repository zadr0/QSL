# qsl-rt

Memory providers for Quick Schema Language. The crate is always `#![no_std]`, does not depend on alloc, and never declares a global allocator. Wire codecs, validation and I/O stay in qsl-core.

| Provider | Ownership and synchronization |
|---|---|
| `LocalRegions<CAP,N>` | Caller-owned pool with a Cell presence mask; no atomics. Not Sync; its leases cannot move between threads. |
| `StaticRegions<CAP,N>` | Caller-owned or static pool with an atomic presence mask; usable across threads. |

Both implement qsl-core `FramePool` and `PreparedFramePool`; their leases implement `FrameStorage`. CAP is the byte capacity per slot; N must be 1–64. Slots are cache-line aligned. The pool's native size includes padding and its occupancy mask, so CAP × N is not its complete native size.

`try_acquire()` returns `PoolExhausted` immediately when every slot is live. Drop releases exactly the leased slot. Each lease initially exposes an empty initialized prefix; publishing bytes requires initialization first. `new_zeroed()` and `PreparedFramePool::ZEROED` initialize the entire capacity for kernels that require a prepared tail. `new()` does not provide that guarantee.

`LocalRegions` and its leases can live inside a function; they do not require a static lifetime. This also applies to core QPS region encoding. Neither provider is an IPC/shared-memory implementation or an allocator for alloc::String/Vec.

```rust
use qsl_core::{FramePool, FrameStorage, QslResult};

fn store<P: FramePool>(pool: &P) -> QslResult<()> {
    let mut output = pool.try_acquire()?;
    if output.capacity() < 3 {
        return Err(qsl_core::QslError::BufferTooSmall);
    }
    // The live lease exclusively owns this slot; exactly three bytes are written.
    unsafe {
        output.writable_ptr().copy_from_nonoverlapping(b"QSL".as_ptr(), 3);
        output.set_initialized_len(3);
    }
    assert_eq!(output.as_bytes(), b"QSL");
    Ok(())
}

let pool = qsl_rt::LocalRegions::<64, 2>::new();
store(&pool)?;
store(&pool)?; // The previous lease was dropped; storage is reused.
# Ok::<(), qsl_core::QslError>(())
```

For generated checked records, `qsl_core::buffer::write_checked(&record, &mut lease)` validates and encodes without exposing a raw pointer to application code. The encoded prefix is published only on success. Keep leases alive while consumers read their bytes.

```text
cargo test --manifest-path qsl-rt/Cargo.toml --locked
cargo check --manifest-path qsl-rt/Cargo.toml --target wasm32-wasip2 --locked
```
