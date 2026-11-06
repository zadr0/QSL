//! Generated memory contracts.
//! Safety decoding is opt-in:
//! ```compile_fail
//! use qsl_modernization_contracts::Trusted;
//! let _ = Trusted::deserialize_checked(&[0;4]);
//! ```
//! Native Rust macros reject incompatible packed layouts:
//! ```compile_fail
//! #[qsl::schem_macro(aggressive_transmutate)]
//! #[repr(C,packed)]
//! struct Bad { field: u32 }
//! ```
//! Even with safety enabled, native unchecked APIs require an unsafe call:
//! ```compile_fail
//! use qsl_modernization_contracts::CopyNative;
//! let _ = CopyNative::deserialize_transmutate_copy_unchecked(&[0;16]);
//! ```
//! ```compile_fail
//! use qsl_modernization_contracts::Region;
//! let _ = Region::deserialize_region_0_view_unchecked(&[0;16]);
//! ```
include!("models.rs");
include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
/// The raw convenience writer has caller-enforced field/length invariants.
/// Safe applications use serialize_checked (or the checked RT/QPS inherent API).
/// ```compile_fail
/// use qsl_modernization_contracts::Trusted;
/// let value=Trusted{id:1};
/// let _=<Trusted as qsl::Serializable>::serialize(&value);
/// ```
pub struct TrustedWriterContract;
#[cfg(test)]
mod allocation_probe {
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        cell::Cell,
    };
    std::thread_local! {static ENABLED:Cell<bool>=const{Cell::new(false)};static COUNT:Cell<usize>=const{Cell::new(0)};}
    pub struct Counting;
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            let _ = ENABLED.try_with(|v| {
                if v.get() {
                    let _ = COUNT.try_with(|n| n.set(n.get() + 1));
                }
            });
            System.alloc(l)
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            System.dealloc(p, l)
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            let _ = ENABLED.try_with(|v| {
                if v.get() {
                    let _ = COUNT.try_with(|c| c.set(c.get() + 1));
                }
            });
            System.realloc(p, l, n)
        }
    }
    #[global_allocator]
    static ALLOCATOR: Counting = Counting;
    pub fn count(f: impl FnOnce()) -> usize {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                ENABLED.with(|v| v.set(false));
            }
        }
        COUNT.with(|n| n.set(0));
        ENABLED.with(|v| v.set(true));
        let reset = Reset;
        f();
        drop(reset);
        COUNT.with(Cell::get)
    }
}
#[cfg(test)]
mod native_validation_probe {
    use std::cell::Cell;
    std::thread_local! {static CALLS:Cell<usize>=const{Cell::new(0)};}
    pub fn record() {
        CALLS.with(|n| n.set(n.get() + 1));
    }
    pub fn reset() {
        CALLS.with(|n| n.set(0));
    }
    pub fn calls() -> usize {
        CALLS.with(Cell::get)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use qsl::Serializable;

    #[test]
    fn native_view_padding_cannot_cover_a_neighbor() {
        let value = PaddedNeighbor {
            meta_id: 7,
            meta_value: 3,
            neighbor: 99,
        };
        assert_eq!(value.region_0_copy().unwrap().id, 7);
        assert!(value.region_0_view().is_err());
        let malformed = HeapFixed {
            text: "short".into(),
            values: vec![1],
        };
        assert!(malformed.serialize_checked().is_err());
        assert!(malformed.serialize().is_err());
    }
    #[test]
    fn proven_cache_capacity_does_not_disable_frame_limit() {
        let value = LargeCache {
            text: "a".repeat(65534),
        };
        let mut cache = qsl::modernization::CachedFrame::<65539>::new();
        value.validate_values().unwrap();
        assert!(matches!(
            value.serialize_cached(&mut cache),
            Err(qsl::QslError::SizeLimitExceeded)
        ));
        assert!(cache.as_slice().is_empty());
    }
    #[test]
    fn checked_stack_read_allocates_nothing() {
        let value = Plain::try_new(7, "Привет".into(), vec![1, 2, 3], Some(0), None).unwrap();
        let bytes = value.serialize_checked().unwrap();
        let baseline = allocation_probe::count(|| {
            std::hint::black_box(Plain::deserialize_checked(&bytes).unwrap());
        });
        let inline = allocation_probe::count(|| {
            std::hint::black_box(Stack::deserialize_checked(&bytes).unwrap());
        });
        assert!(baseline >= 2);
        assert_eq!(inline, 0);
    }
    #[test]
    fn lifetime_native_array_is_owned_but_raw_text_is_borrowed() {
        let value = BorrowNative {
            values: [1, 256, u16::MAX],
            text: "abc",
        };
        let bytes = value.serialize_checked().unwrap();
        let decoded = BorrowNative::deserialize_checked(&bytes).unwrap();
        assert_eq!(decoded.values, value.values);
        assert_eq!(decoded.text.as_ptr(), unsafe { bytes.as_ptr().add(10) });
        assert_eq!(decoded.region_0_copy().unwrap().values, value.values);
    }
    #[test]
    fn native_object_sizes_match_the_editor_target() {
        assert_eq!(std::mem::size_of::<Plain>(), 64);
        assert_eq!(std::mem::size_of::<Stack>(), 136);
        assert_eq!(std::mem::size_of::<HeapFixed>(), 48);
        assert_eq!(std::mem::size_of::<Overlay>(), 16);
        assert_eq!(std::mem::size_of::<Niches>(), 8);
    }
    #[test]
    fn corrupted_frames_never_panic() {
        for len in 0..96 {
            for seed in 0..32u8 {
                let bytes = (0..len)
                    .map(|i| (i as u8).wrapping_mul(37).wrapping_add(seed))
                    .collect::<Vec<_>>();
                assert!(std::panic::catch_unwind(|| {
                    let _ = HybridOptional::deserialize_checked(&bytes);
                    let _ = Table::deserialize_checked(&bytes);
                    let _ = Graph::deserialize_checked(&bytes);
                    let _ = HeapLz::deserialize_checked(&bytes);
                    let _ = CompactArray::deserialize_checked(&bytes);
                    let _ = CopyNative::deserialize_checked(&bytes);
                })
                .is_ok());
            }
        }
    }
    #[test]
    fn ordinary_table_and_full_bitmap_keep_all_states() {
        for mask in 0..8 {
            let x = Table::try_new(
                9,
                (mask & 1 != 0).then_some(0),
                (mask & 2 != 0).then_some(65535),
                (mask & 4 != 0).then_some(u32::MAX),
            )
            .unwrap();
            let bytes = x.serialize_checked().unwrap();
            let y = Table::deserialize_checked(&bytes).unwrap();
            assert_eq!((y.a(), y.b(), y.c()), (x.a(), x.b(), x.c()));
        }
        let mut x = Many::try_new(
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
        )
        .unwrap();
        x.set_f0(Some(0)).unwrap();
        x.set_f63(Some(255)).unwrap();
        let bytes = x.serialize_checked().unwrap();
        let y = Many::deserialize_checked(&bytes).unwrap();
        assert_eq!((y.f0(), y.f63()), (Some(0), Some(255)));
        assert_eq!(y.f32(), None);
        for end in 0..bytes.len() {
            assert!(Many::deserialize_checked(&bytes[..end]).is_err());
        }
    }
    #[test]
    fn niches_restore_signed_and_zero_without_collision() {
        for shifted in [None, Some(-10), Some(0), Some(10)] {
            let x = Niches::try_new(Some(255), Some(-9), shifted, Some(0)).unwrap();
            let bytes = x.serialize_checked().unwrap();
            let y = Niches::deserialize_checked(&bytes).unwrap();
            assert_eq!(
                (y.positive(), y.negative(), y.shifted(), y.full()),
                (Some(255), Some(-9), shifted, Some(0))
            );
        }
        assert!(Niches::try_new(Some(0), None, None, None).is_err());
    }
    #[test]
    fn nested_native_regions_reconstruct_padding_safely() {
        let x = NestedRegion {
            nested_lead: 1,
            nested_meta_id: 42,
            nested_meta_value: 7,
            nested_meta_tag: *b"abcd",
            nested_end: 2,
            name: "ok".into(),
        };
        let native = x.region_0_copy().unwrap();
        assert_eq!((native.lead, native.meta.id, native.end), (1, 42, 2));
        assert_eq!(x.region_0_view().unwrap().meta.id, 42);
        let bytes = x.serialize_checked().unwrap();
        let native = NestedRegion::deserialize_region_0_copy(&bytes[..18]).unwrap();
        assert_eq!(native.meta.tag, *b"abcd");
        assert!(NestedRegion::deserialize_region_0_view(&bytes[..18]).is_err());
    }
    #[test]
    fn wide_quantization_all_quanta_are_exact_and_unpadded() {
        macro_rules! check {
            ($t:ty) => {
                for a in [None, Some(0), Some(15)] {
                    for b in [None, Some(1), Some(65535)] {
                        let x = <$t>::try_new(1000, -100, a, b).unwrap();
                        let len = x.serialized_size();
                        let mut storage = vec![0xA5; len + 16];
                        let frame = x.serialize_quantized_to(&mut storage[..len]).unwrap();
                        let mut dst = std::mem::MaybeUninit::uninit();
                        let y = frame.decode_into(&mut dst);
                        assert_eq!((y.id, y.signed, y.a(), y.b()), (1000, -100, a, b));
                        assert_eq!(<$t>::deserialize_checked(&storage[..len]).unwrap().b(), b);
                        assert!(storage[len..].iter().all(|&b| b == 0xA5));
                    }
                }
            };
        }
        check!(WideQ2);
        check!(WideQ4);
        check!(WideQ8);
        check!(WideQ16);
    }
    #[test]
    fn exact_nine_byte_load_at_protected_page_boundary() {
        #[cfg(windows)]
        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn VirtualAlloc(a: *mut core::ffi::c_void, n: usize, t: u32, p: u32) -> *mut u8;
                fn VirtualProtect(
                    a: *mut core::ffi::c_void,
                    n: usize,
                    p: u32,
                    old: *mut u32,
                ) -> i32;
                fn VirtualFree(a: *mut core::ffi::c_void, n: usize, t: u32) -> i32;
            }
            let mem = VirtualAlloc(std::ptr::null_mut(), 8192, 0x3000, 4);
            assert!(!mem.is_null());
            let mut old = 0;
            assert_ne!(VirtualProtect(mem.add(4096).cast(), 4096, 1, &mut old), 0);
            let source = mem.add(4096 - 9);
            std::ptr::copy_nonoverlapping([1u8, 2, 3, 4, 5, 6, 7, 8, 9].as_ptr(), source, 9);
            assert_eq!(
                qsl::modernization::load_exact::<9>(source),
                0x090807060504030201
            );
            assert_ne!(VirtualFree(mem.cast(), 0, 0x8000), 0);
        }
    }
    #[test]
    fn optional_scalar_prefix_preserves_exact_states() {
        for count in [None, Some(0), Some(15)] {
            for text in [None, Some("abc")] {
                let x = HybridOptional::try_new(
                    2,
                    count,
                    text.map(|s| qsl::StackString::try_from_str(s).unwrap()),
                )
                .unwrap();
                let bytes = x.serialize_checked().unwrap();
                let y = HybridOptional::deserialize_checked(&bytes).unwrap();
                assert_eq!(y.count(), count);
                assert_eq!(y.text.as_ref().map(|s| s.as_str()), text);
                assert!(HybridOptional::deserialize_checked(&bytes[..bytes.len() - 1]).is_err());
            }
        }
    }
    #[test]
    fn strict_stack_limit_preserves_large_string_codecs() {
        let text = "abc".repeat(300);
        let x = HeapLz { text: text.clone() };
        let bytes = x.serialize_checked().unwrap();
        assert!(bytes.len() < text.len());
        assert_eq!(HeapLz::deserialize_checked(&bytes).unwrap().text, text);
        let x = HeapInline { text: text.clone() };
        let bytes = x.serialize_checked().unwrap();
        assert_eq!(HeapInline::deserialize_checked(&bytes).unwrap().text, text);
        let x = HeapFixed {
            text: "a".repeat(1024),
            values: vec![513; 200],
        };
        let bytes = x.serialize_checked().unwrap();
        let y = HeapFixed::deserialize_checked(&bytes).unwrap();
        assert_eq!(y.text, x.text);
        assert_eq!(y.values, x.values);
    }
    #[test]
    fn fixed_and_inline_lifetime_strings_borrow_input() {
        let x = BorrowInline {
            tag: "abc",
            text: "hello",
        };
        let bytes = x.serialize_checked().unwrap();
        let y = BorrowInline::deserialize_checked(&bytes).unwrap();
        assert_eq!(y.tag.as_ptr(), bytes.as_ptr());
        assert_eq!(y.text.as_ptr(), unsafe { bytes.as_ptr().add(5) });
        assert_eq!(y.text, "hello");
    }
    #[test]
    fn compact_numeric_arrays_respect_bit_alignment() {
        let x = CompactArray {
            id: 7,
            values: qsl::InlineArray::try_from_iter([u32::MAX, 65536]).unwrap(),
            fixed: [u16::MAX, 256, 1],
        };
        let bytes = x.serialize_checked().unwrap();
        CompactArray::validate(&bytes).unwrap();
        let y = CompactArray::deserialize_checked(&bytes).unwrap();
        assert_eq!(y.id, 7);
        assert_eq!(y.values.as_slice(), x.values.as_slice());
        assert_eq!(y.fixed, x.fixed);
    }
    #[test]
    fn mixed_qps_only_packs_scalars() {
        let x = Hybrid {
            a: 3,
            b: 15,
            text: qsl::StackString::try_from_str("hello").unwrap(),
            data: qsl::InlineArray::try_from_iter([1, 256, 65536]).unwrap(),
        };
        let bytes = x.serialize_checked().unwrap();
        assert_eq!(Hybrid::QPS_SCALAR_PREFIX_BYTES, 1);
        assert_eq!(&bytes[1..5], &5u32.to_le_bytes());
        let decoded = Hybrid::deserialize_checked(&bytes).unwrap();
        assert_eq!((decoded.a, decoded.b), (3, 15));
        assert_eq!(decoded.text.as_str(), "hello");
        assert_eq!(decoded.data.as_slice(), &[1, 256, 65536]);
    }
    #[test]
    fn numeric_arrays_use_element_widths() {
        let x = ArrayStack {
            values: qsl::InlineArray::try_from_iter([0u32, 65536, u32::MAX]).unwrap(),
            fixed: [1, 256, u16::MAX],
        };
        let bytes = x.serialize_checked().unwrap();
        assert_eq!(bytes.len(), 4 + 12 + 6);
        let x = ArrayStack::deserialize_checked(&bytes).unwrap();
        assert_eq!(x.values.as_slice(), &[0, 65536, u32::MAX]);
        assert_eq!(x.fixed, [1, 256, u16::MAX]);
    }
    #[test]
    fn stack_owned_and_plain_have_identical_wire_and_checked_values() {
        for mask in 0..4 {
            let text = "Привет";
            let raw = Plain::try_new(
                7,
                text.into(),
                vec![1, 2, 3],
                (mask & 1 != 0).then_some(0),
                (mask & 2 != 0).then_some(65535),
            )
            .unwrap();
            let fast = Stack::try_new(
                7,
                qsl::StackString::try_from_str(text).unwrap(),
                qsl::InlineArray::try_from_iter([1, 2, 3]).unwrap(),
                raw.a(),
                raw.b(),
            )
            .unwrap();
            let wire = raw.serialize_checked().unwrap();
            assert_eq!(wire, fast.serialize_checked().unwrap());
            let x = Stack::deserialize_checked(&wire).unwrap();
            assert_eq!(x.text.as_str(), text);
            assert_eq!(x.data.as_slice(), &[1, 2, 3]);
            assert_eq!(x.a(), raw.a());
            assert_eq!(x.b(), raw.b());
            for end in 0..wire.len() {
                assert!(Stack::deserialize_checked(&wire[..end]).is_err())
            }
        }
    }
    #[test]
    fn malformed_length_does_not_publish_partial_output() {
        let value = Stack::try_new(
            9,
            qsl::StackString::try_from_str("abc").unwrap(),
            qsl::InlineArray::new(),
            None,
            None,
        )
        .unwrap();
        let mut bytes = value.serialize_checked().unwrap();
        bytes[5..9].copy_from_slice(&u32::MAX.to_le_bytes());
        let mut output = std::mem::MaybeUninit::new(value);
        assert!(Stack::deserialize_checked_into(&bytes, &mut output).is_err());
        assert_eq!(unsafe { output.assume_init_ref() }.id, 9);
        unsafe { output.assume_init_drop() };
    }
    #[test]
    fn length_graph_materializes_stack_strings() {
        for mask in 0..4 {
            let input = Graph {
                id: 3,
                a: if mask & 1 != 0 {
                    Some(qsl::StackString::try_from_str("abc").unwrap())
                } else {
                    None
                },
                b: if mask & 2 != 0 {
                    Some(qsl::StackString::try_from_str("xy").unwrap())
                } else {
                    None
                },
            };
            let bytes = input.serialize_checked().unwrap();
            let x = Graph::deserialize_checked(&bytes).unwrap();
            assert_eq!(
                x.a.as_ref().map(|s| s.as_str()),
                input.a.as_ref().map(|s| s.as_str())
            );
            assert_eq!(
                x.b.as_ref().map(|s| s.as_str()),
                input.b.as_ref().map(|s| s.as_str())
            );
        }
    }
    #[test]
    fn quantized_frames_require_no_private_padding() {
        for c in [None, Some(1), Some(15)] {
            let value = Numeric::try_new(0, 15, c).unwrap();
            let len = value.serialized_size();
            assert_eq!(Numeric::QUANT_PADDING_BYTES, 0);
            let mut bytes = vec![0xA5; len + 16];
            let frame = value.serialize_quantized_to(&mut bytes[..len]).unwrap();
            assert_eq!(frame.as_slice().len(), len);
            let mut out = std::mem::MaybeUninit::uninit();
            let x = frame.decode_into(&mut out);
            assert_eq!((x.a, x.b, x.c()), (0, 15, c));
            assert!(bytes[len..].iter().all(|&b| b == 0xA5));
            let checked = qsl::QuantizedFrame::<Numeric>::try_new(&bytes[..len], len).unwrap();
            let mut out = std::mem::MaybeUninit::uninit();
            assert_eq!(checked.decode_into(&mut out).c(), c);
        }
    }
    #[test]
    fn unchecked_native_methods_coexist_with_safety_and_legacy_names() {
        let value = CopyNative {
            id: 42,
            value: 11,
            tag: *b"utf8",
        };
        let bytes = value.serialize_checked().unwrap();
        // Safety adds checked entry points without removing the trusted ones.
        let x = unsafe { CopyNative::deserialize_transmutate_copy_unchecked(&bytes) };
        assert_eq!((x.id, x.value, x.tag), (42, 11, *b"utf8"));
        let x = unsafe { CopyTrusted::deserialize_transmutate_copy_unchecked(&bytes) };
        assert_eq!((x.id, x.value, x.tag), (42, 11, *b"utf8"));
        let x = unsafe { CopyTrusted::deserialize_transmutate_copy(&bytes) };
        assert_eq!((x.id, x.value, x.tag), (42, 11, *b"utf8"));
        let fallback = CopyFallback { tag: 1, id: 42 };
        assert!(!std::hint::black_box(CopyFallback::NATIVE_WIRE_COMPATIBLE));
        let packed = fallback.serialize_checked().unwrap();
        assert_eq!(packed.len(), 9);
        let x = unsafe { CopyFallback::deserialize_transmutate_copy_unchecked(&packed) };
        assert_eq!((x.tag, x.id), (1, 42));
        #[repr(align(8))]
        struct Aligned([u8; 16]);
        let mut input = Aligned([0; 16]);
        input.0.copy_from_slice(&bytes);
        let x = unsafe { Overlay::deserialize_overlay_unchecked(&input.0) };
        assert_eq!(x as *const _ as *const u8, input.0.as_ptr());
        assert_eq!((x.id, x.value, x.tag), (42, 11, *b"utf8"));
        let x = unsafe { OverlayTrusted::deserialize_overlay_unchecked(&input.0) };
        assert_eq!(x as *const _ as *const u8, input.0.as_ptr());
        let x = unsafe { OverlayTrusted::deserialize_overlay(&input.0) };
        assert_eq!(x as *const _ as *const u8, input.0.as_ptr());
        // Existing general unsafe APIs are still independent of safety too.
        let mut out = std::mem::MaybeUninit::<CopyNative>::uninit();
        unsafe {
            CopyNative::deserialize_into(bytes.as_ptr(), bytes.len(), out.as_mut_ptr());
        }
        assert_eq!(unsafe { out.assume_init() }.id, 42);
    }

    #[test]
    fn unchecked_regions_do_not_invoke_native_validation() {
        let value = CountedRegion {
            id: 5,
            value: 3,
            tag: *b"abcd",
        };
        let bytes = value.serialize_checked().unwrap();
        #[repr(align(8))]
        struct Aligned([u8; 16]);
        let mut input = Aligned([0; 16]);
        input.0.copy_from_slice(&bytes);
        assert!(std::hint::black_box(
            CountedRegion::REGION_0_COPY_COMPATIBLE
        ));
        assert!(std::hint::black_box(
            CountedRegion::REGION_0_VIEW_COMPATIBLE
        ));
        assert!(std::hint::black_box(
            CountedRegion::REGION_0_WIRE_COPY_COMPATIBLE
        ));
        assert!(std::hint::black_box(
            CountedRegion::REGION_0_WIRE_VIEW_COMPATIBLE
        ));
        native_validation_probe::reset();
        let a = unsafe { value.region_0_copy_unchecked() };
        let b = unsafe { value.region_0_view_unchecked() };
        let c = unsafe { CountedRegion::deserialize_region_0_copy_unchecked(&input.0) };
        let d = unsafe { CountedRegion::deserialize_region_0_view_unchecked(&input.0) };
        for x in [&a, b, &c, d] {
            assert_eq!((x.id, x.value, x.tag), (5, 3, *b"abcd"));
        }
        assert_eq!(
            b as *const _ as *const u8,
            std::ptr::addr_of!(value.id).cast()
        );
        assert_eq!(d as *const _ as *const u8, input.0.as_ptr());
        assert_eq!(native_validation_probe::calls(), 0);
        value.region_0_copy().unwrap();
        value.region_0_view().unwrap();
        CountedRegion::deserialize_region_0_copy(&input.0).unwrap();
        CountedRegion::deserialize_region_0_view(&input.0).unwrap();
        assert_eq!(native_validation_probe::calls(), 4);
        input.0[12] = 255;
        assert!(CountedRegion::deserialize_region_0_copy(&input.0).is_err());
        assert!(CountedRegion::deserialize_region_0_view(&input.0).is_err());
    }

    #[test]
    fn unchecked_regions_are_available_without_safety() {
        let value = RegionTrusted {
            id: 7,
            value: 3,
            tag: *b"abcd",
        };
        #[repr(align(8))]
        struct Aligned([u8; 16]);
        let mut input = Aligned([0; 16]);
        unsafe {
            value.serialize_into_unsafe(input.0.as_mut_ptr());
        }
        let a = unsafe { value.region_0_copy_unchecked() };
        let b = unsafe { value.region_0_view_unchecked() };
        let c = unsafe { RegionTrusted::deserialize_region_0_copy_unchecked(&input.0) };
        let d = unsafe { RegionTrusted::deserialize_region_0_view_unchecked(&input.0) };
        for x in [&a, b, &c, d] {
            assert_eq!((x.id, x.value, x.tag), (7, 3, *b"abcd"));
        }
        // An incompatible VIEW does not disable the leaf-wise COPY path.
        let padded = PaddedNeighbor {
            meta_id: 7,
            meta_value: 3,
            neighbor: 99,
        };
        assert!(!std::hint::black_box(
            PaddedNeighbor::REGION_0_VIEW_COMPATIBLE
        ));
        assert!(std::hint::black_box(
            PaddedNeighbor::REGION_0_COPY_COMPATIBLE
        ));
        let copied = unsafe { padded.region_0_copy_unchecked() };
        assert_eq!((copied.id, copied.value), (7, 3));
    }

    #[test]
    fn native_copy_and_overlay_validate_before_exposing_values() {
        let value = CopyNative {
            id: 42,
            value: 11,
            tag: *b"utf8",
        };
        let bytes = value.serialize_checked().unwrap();
        assert_eq!(bytes.len(), 16);
        let x = CopyNative::deserialize_transmutate_copy(&bytes).unwrap();
        assert_eq!((x.id, x.value, x.tag), (42, 11, *b"utf8"));
        #[repr(align(8))]
        struct A([u8; 24]);
        let mut aligned = A([0; 24]);
        aligned.0[..16].copy_from_slice(&bytes);
        let x = Overlay::deserialize_overlay(&aligned.0[..16]).unwrap();
        assert_eq!(x as *const _ as *const u8, aligned.0.as_ptr());
        aligned.0[1..17].copy_from_slice(&bytes);
        assert!(Overlay::deserialize_overlay(&aligned.0[1..17]).is_err());
        aligned.0[..16].copy_from_slice(&bytes);
        aligned.0[12] = 255;
        assert!(Overlay::deserialize_overlay(&aligned.0[..16]).is_err());
    }
    #[test]
    fn native_regions_and_proven_cache() {
        let value = Region {
            id: 5,
            value: 3,
            tag: *b"abcd",
            name: "ok".into(),
        };
        let m = value.region_0_copy().unwrap();
        assert_eq!((m.id, m.value, m.tag), (5, 3, *b"abcd"));
        assert_eq!(value.region_0_view().unwrap().id, 5);
        let value = Cache {
            id: 2,
            text: "hi".into(),
        };
        let mut cached = qsl::modernization::CachedFrame::<28>::new();
        assert_eq!(
            value.serialize_cached(&mut cached).unwrap(),
            value.serialize_checked().unwrap()
        );
    }
    #[test]
    fn lz_remains_lossless() {
        let text = "abc".repeat(64);
        let x = Compressed {
            text: unsafe { qsl::InlineString::from_str_unchecked(&text) },
        };
        let bytes = x.serialize_checked().unwrap();
        assert!(bytes.len() < text.len());
        assert_eq!(
            Compressed::deserialize_checked(&bytes)
                .unwrap()
                .text
                .as_str(),
            text
        );
    }
    #[test]
    fn region_slots_wait_and_resume_without_lost_wakeup() {
        use std::sync::mpsc;
        let a = RT { id: 1 }.serialize().unwrap();
        let b = RT { id: 2 }.serialize().unwrap();
        let (ready, r) = mpsc::channel();
        let (done, d) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            ready.send(()).unwrap();
            let lease = RT { id: 3 }.serialize().unwrap();
            done.send(lease.as_slice().to_vec()).unwrap();
        });
        r.recv().unwrap();
        assert!(d
            .recv_timeout(std::time::Duration::from_millis(20))
            .is_err());
        drop(a);
        assert_eq!(
            d.recv_timeout(std::time::Duration::from_secs(2)).unwrap(),
            3u32.to_le_bytes()
        );
        drop(b);
        worker.join().unwrap();
    }
}
