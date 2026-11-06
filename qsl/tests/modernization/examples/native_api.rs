//! Run from dev-qsl:
//! cargo run --manifest-path qsl/tests/modernization/Cargo.toml --example native_api --locked
//! Schemas: ../schemas.qsl. Rust native template: ../models.rs.
use qsl::Serializable;
use qsl_modernization_contracts::{CopyNative, Meta, Overlay, Region};
use std::mem::MaybeUninit;

// The example's native layouts are established at compile time.
const _: () = {
    assert!(Overlay::NATIVE_ALIGNMENT <= 8);
    assert!(Region::REGION_0_COPY_COMPATIBLE);
    assert!(Region::REGION_0_VIEW_COMPATIBLE);
    assert!(Region::REGION_0_WIRE_COPY_COMPATIBLE);
    assert!(Region::REGION_0_WIRE_VIEW_COMPATIBLE);
};

#[repr(align(8))]
struct AlignedFrame([u8; 16]);

fn main() -> qsl::QslResult<()> {
    let value = CopyNative {
        id: 42,
        value: 11,
        tag: *b"utf8",
    };
    let mut frame = AlignedFrame([0; 16]);
    // Returns the written length, not a slice.
    let len = value.serialize_checked_to(&mut frame.0)?;
    let wire = &frame.0[..len];

    // 1. COPY: checked and unchecked entry points coexist with [safety].
    let checked = CopyNative::deserialize_transmutate_copy(wire)?;
    // SAFETY: wire was produced by this schema's checked encoder.
    let direct = unsafe { CopyNative::deserialize_transmutate_copy_unchecked(wire) };
    assert_eq!((checked.id, direct.id), (42, 42));

    // The ordinary caller-owned destination API is retained too.
    let mut out = MaybeUninit::<CopyNative>::uninit();
    // SAFETY: valid complete wire; aligned, uninitialized and disjoint output.
    let decoded = unsafe {
        <CopyNative as Serializable>::deserialize_into(wire.as_ptr(), wire.len(), out.as_mut_ptr());
        out.assume_init()
    };
    assert_eq!(decoded.id, 42);
    println!(
        "COPY: checked={}, unchecked={}, into={}",
        checked.id, direct.id, decoded.id
    );

    // 2. OVERLAY: a reference into aligned input, without a payload copy.
    // CopyNative and Overlay use identical 16-byte wire/native field layouts.
    let checked_view = Overlay::deserialize_overlay(wire)?;
    // SAFETY: the checked overlay established validity; input remains immutable,
    // aligned and alive until both references are no longer used.
    let direct_view = unsafe { Overlay::deserialize_overlay_unchecked(wire) };
    assert_eq!(direct_view as *const _ as *const u8, wire.as_ptr());
    println!(
        "OVERLAY: checked={}, unchecked={}, same_address=true",
        checked_view.id, direct_view.id
    );

    // 3. REGIONS: native leaves can be copied or viewed independently of the tail.
    let record = Region {
        id: 42,
        value: 11,
        tag: *b"utf8",
        name: "tail".into(),
    };
    // SAFETY: compile-time layout assertions above; fields are valid and the
    // record stays immutably borrowed. Only initialized leaves are copied.
    let copied: Meta = unsafe { record.region_0_copy_unchecked() };
    let viewed: &Meta = unsafe { record.region_0_view_unchecked() };
    // SAFETY: wire contains exactly the compatible 16-byte region, not the
    // surrounding Region frame. The input is aligned and remains alive.
    let wire_copy: Meta = unsafe { Region::deserialize_region_0_copy_unchecked(wire) };
    let wire_view: &Meta = unsafe { Region::deserialize_region_0_view_unchecked(wire) };
    assert_eq!(
        (copied.id, viewed.id, wire_copy.id, wire_view.id),
        (42, 42, 42, 42)
    );
    println!(
        "REGION: self_copy={}, self_view={}, wire_copy={}, wire_view={}",
        copied.id, viewed.id, wire_copy.id, wire_view.id
    );
    Ok(())
}
