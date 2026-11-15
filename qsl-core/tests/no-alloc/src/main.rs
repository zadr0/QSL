#![no_std]
#![no_main]
use qsl_core::{InlineArray, QslError, StackString};
#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn ExitProcess(code: u32) -> !;
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    exit(101)
}
fn exit(code: u32) -> ! {
    #[cfg(target_os = "windows")]
    unsafe {
        ExitProcess(code)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = code;
        loop {
            core::hint::spin_loop()
        }
    }
}
include!(concat!(env!("OUT_DIR"), "/schemas.rs"));
fn check() -> Result<(), QslError> {
    let text = StackString::<32>::try_from_str("Яa")?;
    if text.len() != 3 {
        return Err(QslError::InvalidValue);
    }
    let mut values = InlineArray::<u32, 4>::new();
    values.push(7)?;
    if values.as_slice() != [7] {
        return Err(QslError::InvalidValue);
    }
    let mut encoded = [0u8; 272];
    let mut restored = [0u8; 256];
    let frame = qsl_core::qps_bytes::ByteQps::<16>::encode_to(&[b'a'; 256], &mut encoded)?;
    if frame.decode_to(&mut restored)? != [b'a'; 256] {
        return Err(QslError::InvalidValue);
    }
    let mut c = qsl_core::validation::ValidationCursor::with_limit(&[0x84, 0, b'A', 0xDF, 1], 100)?;
    if c.decode_compressed::<100>(0, false, false)?.len() != 100 {
        return Err(QslError::InvalidValue);
    }
    c.finish()?;
    let mut forged = qsl_core::validation::ValidationCursor::new(&[0x84, 0, b'A', 0xFF, 1]);
    if forged.decode_compressed::<100>(0, false, false).is_ok() {
        return Err(QslError::InvalidValue);
    }
    let mut storage = [0u8; 8];
    let mut out = qsl_core::SliceOut::new(&mut storage);
    out[0] = 42;
    out.commit(1)?;
    if out.frame() != [42] {
        return Err(QslError::InvalidValue);
    }
    let native = NativeRecord {
        NativeFields: NativeFields { id: 1, value: 2 },
        tail: 3,
    };
    let frame = native.serialize()?;
    let back = NativeRecord::deserialize_checked(&frame)?;
    if back.NativeFields.id != 1 || back.NativeFields.value != 2 || back.tail != 3 {
        return Err(QslError::InvalidValue);
    }
    let basic = Basic {
        kind: 5,
        value: 513,
    };
    let owned = basic.serialize_checked()?;
    if Basic::deserialize_checked(&owned)?.kind != basic.kind {
        return Err(QslError::InvalidValue);
    }
    let frame = basic.serialize()?;
    let back = Basic::deserialize_checked(&frame)?;
    if (back.kind, back.value) != (5, 513) {
        return Err(QslError::InvalidValue);
    }
    let record = StackRecord {
        name: qsl::StackString::try_from_str("Яa")?,
        values: qsl::InlineArray::try_from_slice(&[0, 1, 65535])?,
    };
    let frame = record.serialize()?;
    let back = StackRecord::deserialize_checked(&frame)?;
    if back.name.as_str() != "Яa" || back.values.as_slice() != [0, 1, 65535] {
        return Err(QslError::InvalidValue);
    }
    let mut reused = StackRecord {
        name: qsl::StackString::new(),
        values: qsl::InlineArray::new(),
    };
    unsafe {
        StackRecord::deserialize_into(frame.as_ptr(), frame.len(), &mut reused);
    }
    if reused.name.as_str() != record.name.as_str()
        || reused.values.as_slice() != record.values.as_slice()
    {
        return Err(QslError::InvalidValue);
    }
    let record = LzRecord {
        text: qsl::InlineString::try_from_str(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )?,
    };
    let frame = record.serialize()?;
    if LzRecord::deserialize_checked(&frame)?.text.as_str() != record.text.as_str() {
        return Err(QslError::InvalidValue);
    }
    let borrowed = BorrowedRecord { name: "Яa" };
    let frame = borrowed.serialize()?;
    if BorrowedRecord::deserialize_checked(&frame)?.name != "Яa" {
        return Err(QslError::InvalidValue);
    }
    // A heap-selected owned frame does not disable caller-buffer I/O without alloc.
    let large = LargeBorrowedRecord { name: "Яa" };
    let mut provided = [0u8; 64];
    let used = large.serialize_checked_to(&mut provided)?;
    if LargeBorrowedRecord::deserialize_checked(&provided[..used])?.name != large.name {
        return Err(QslError::InvalidValue);
    }
    for a in [None, Some(7)] {
        for b in [None, Some(512)] {
            let v = Predicted::try_new(42, a, b)?;
            let frame = v.serialize()?;
            let v = Predicted::deserialize_checked(&frame)?;
            if v.a() != a || v.b() != b {
                return Err(QslError::InvalidValue);
            }
        }
    }
    let q = QuantRecord {
        a: 0,
        b: 3,
        c: 7,
        d: 15,
    };
    let frame = q.serialize()?;
    let mut out = core::mem::MaybeUninit::uninit();
    let q = frame.frame().decode_into(&mut out);
    if (q.a, q.b, q.c, q.d) != (0, 3, 7, 15) {
        return Err(QslError::InvalidValue);
    }
    for values in [
        None,
        Some(qsl::InlineArray::new()),
        Some(qsl::InlineArray::try_from_slice(&[0, 1, 65535])?),
    ] {
        let v = OptionalArrays { id: 7, values };
        let frame = v.serialize()?;
        let back = OptionalArrays::deserialize_checked(&frame)?;
        if back.values.as_ref().map(|v| v.as_slice()) != v.values.as_ref().map(|v| v.as_slice()) {
            return Err(QslError::InvalidValue);
        }
    }
    let typed = TypedStorage {
        name: qsl::StackString::try_from_str("Яa")?,
        values: qsl::InlineArray::try_from_slice(&[0, 1, 65535])?,
        extra: Some(qsl::InlineArray::new()),
    };
    let frame = typed.serialize()?;
    let back = TypedStorage::deserialize_checked(&frame)?;
    if back.name.as_str() != "Яa"
        || back.values.as_slice() != [0, 1, 65535]
        || back.extra.as_ref().map(|a| a.len()) != Some(0)
    {
        return Err(QslError::InvalidValue);
    }
    let local = qsl_rt::LocalRegions::<16, 2>::new();
    let mut local_frame = local.try_acquire()?;
    qsl_core::buffer::write_checked(&basic, &mut local_frame)?;
    if Basic::deserialize_checked(&local_frame)?.value != 513 {
        return Err(QslError::InvalidValue);
    }
    let local_qps =
        qsl_core::payload_quant::QuantizedRegions::<QuantRecord, qsl_rt::LocalRegions<32, 2>>::new(
        );
    let local_frame = local_qps.encode(q)?;
    let mut destination = core::mem::MaybeUninit::uninit();
    let back = local_frame.frame().decode_into(&mut destination);
    if (back.a, back.b, back.c, back.d) != (0, 3, 7, 15) {
        return Err(QslError::InvalidValue);
    }
    let pool = qsl_rt::StaticRegions::<16, 1>::new();
    let mut slot = qsl_core::FramePool::try_acquire(&pool)?;
    unsafe {
        qsl_core::FrameStorage::writable_ptr(&mut slot).write(42);
        qsl_core::FrameStorage::set_initialized_len(&mut slot, 1);
    }
    if qsl_core::FrameStorage::as_bytes(&slot) != [42] {
        return Err(QslError::InvalidValue);
    }
    Ok(())
}
#[no_mangle]
pub extern "C" fn mainCRTStartup() -> ! {
    exit(if check().is_ok() { 0 } else { 1 })
}
