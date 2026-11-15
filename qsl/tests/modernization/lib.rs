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
//! let _ = <Region as qsl::Serializable>::deserialize(&[0;16]);
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
    fn stack_limit_from_builder_and_deq_controls_the_output_type() {
        // StackOut's length word exists in memory, but is outside the wire limit.
        assert_eq!(core::mem::size_of::<qsl::StackOut<64>>(), 68);
        let exact = OutputLimit64 { bytes: [7; 64] };
        let beyond = OutputLimit65 {
            bytes: [7; 64],
            tail: 9,
        };
        let mut stack = None;
        let mut heap = None;
        assert_eq!(
            allocation_probe::count(|| stack = Some(exact.serialize().unwrap())),
            0
        );
        assert_eq!(
            allocation_probe::count(|| heap = Some(beyond.serialize().unwrap())),
            1
        );
        let stack: qsl::StackOut<64> = stack.unwrap();
        let heap: Vec<u8> = heap.unwrap();
        assert_eq!(stack.len(), 64);
        assert_eq!(heap.len(), 65);
        assert_eq!(stack.as_slice(), &heap[..64]);
        assert_eq!(
            OutputLimit64::deserialize_checked(&stack).unwrap().bytes,
            exact.bytes
        );
        let restored = OutputLimit65::deserialize_checked(&heap).unwrap();
        assert_eq!((restored.bytes, restored.tail), (beyond.bytes, 9));
        let empty = OutputZero { bytes: [] };
        let out: qsl::StackOut<0> = empty.serialize().unwrap();
        assert!(out.is_empty());
        assert!(OutputZero::deserialize_checked(&[0]).is_err());
    }
    #[test]
    fn compact_wide_domains_select_the_packed_output_bound() {
        let value = OutputDomain {
            f0: 0,
            f1: 1,
            f2: 2,
            f3: 3,
            f4: 4,
            f5: 5,
            f6: 6,
            f7: 7,
            f8: 8,
            f9: 9,
            f10: 10,
            f11: 11,
            f12: 12,
            f13: 13,
            f14: 14,
            f15: 15,
        };
        let output: qsl::StackOut<8> = value.serialize().unwrap();
        assert_eq!(
            output.as_slice(),
            [0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe]
        );
        assert_eq!(OutputDomain::deserialize_checked(&output).unwrap().f15, 15);
    }
    #[test]
    fn owned_output_uses_schema_bound_without_allocating_small_frames() {
        let value = Output64 { bytes: [7; 64] };
        let mut encoded = None;
        assert_eq!(
            allocation_probe::count(|| encoded = Some(value.serialize().unwrap())),
            0
        );
        let encoded: qsl::StackOut<64> = encoded.unwrap();
        assert_eq!(encoded.len(), 64);
        assert_eq!(&*encoded, &[7; 64]);
        let trusted: qsl::StackOut<64> =
            unsafe { <Output64 as qsl::Serializable>::serialize(&value) };
        assert_eq!(encoded, trusted);
        let edge = Output252 { text: [b'a'; 252] };
        let _: qsl::StackOut<252> = edge.serialize().unwrap();
        let within = Output253 { text: [b'a'; 253] };
        let _: qsl::StackOut<253> = within.serialize().unwrap();
        let exact = Output256 { text: [b'a'; 256] };
        let _: qsl::StackOut<256> = exact.serialize().unwrap();
        let beyond = Output257 {
            text: "a".repeat(257),
        };
        let _: Vec<u8> = beyond.serialize().unwrap();
        let input = OutputBorrowed { text: "Яa" };
        let frame = input.serialize().unwrap();
        assert_eq!(
            OutputBorrowed::deserialize_checked(&frame).unwrap().text,
            "Яa"
        );
        let mut too_small = [0xA5; 63];
        assert!(value.serialize_checked_to(&mut too_small).is_err());
        assert_eq!(too_small, [0xA5; 63]);
    }
    #[test]
    fn large_and_unknown_frames_choose_heap_even_for_short_values() {
        for len in [0, 1, 4096] {
            let value = OutputLarge {
                first: "a".repeat(len),
                second: "b".repeat(len),
            };
            let mut frame = None;
            assert_eq!(
                allocation_probe::count(|| frame = Some(value.serialize().unwrap())),
                1
            );
            let frame: Vec<u8> = frame.unwrap();
            assert_eq!(frame.len(), 8 + 2 * len);
            let decoded = OutputLarge::deserialize_checked(&frame).unwrap();
            assert_eq!((decoded.first, decoded.second), (value.first, value.second));
        }
        let value = OutputUnknown { text: "Яa".into() };
        let _: Vec<u8> = value.serialize().unwrap();
    }
    #[test]
    fn compressed_and_dyn_output_publish_only_written_bytes() {
        let value = OutputLz {
            text: qsl::InlineString::try_from_str(&"a".repeat(100)).unwrap(),
        };
        let mut output = None;
        assert_eq!(
            allocation_probe::count(|| output = Some(value.serialize().unwrap())),
            0
        );
        let frame = output.unwrap();
        assert!(frame.len() < 100);
        assert_eq!(
            OutputLz::deserialize_checked(&frame).unwrap().text.as_str(),
            value.text.as_str()
        );
        let value = OutputDyn { id: 7, text: None };
        let mut output = None;
        assert_eq!(
            allocation_probe::count(|| output = Some(value.serialize().unwrap())),
            0
        );
        let frame = output.unwrap();
        let decoded = OutputDyn::deserialize_checked(&frame, &mut [0u64; 2]).unwrap();
        assert_eq!(decoded.id, 7);
        assert!(decoded.text.is_none());
    }
    #[test]
    fn small_qps_output_uses_stack_storage_and_keeps_typed_decode() {
        let value = OutputQuant { a: 3, b: 15 };
        let mut output = None;
        assert_eq!(
            allocation_probe::count(|| output = Some(value.serialize().unwrap())),
            0
        );
        let output = output.unwrap();
        assert_eq!(output.len(), 1);
        let mut destination = core::mem::MaybeUninit::uninit();
        let decoded = output.frame().decode_into(&mut destination);
        assert_eq!((decoded.a, decoded.b), (3, 15));
    }
    #[test]
    fn native_view_padding_cannot_cover_a_neighbor() {
        let value = PaddedNeighbor {
            meta: PaddedMeta { id: 7, value: 3 },
            neighbor: 99,
        };
        let frame = value.serialize_checked().unwrap();
        assert_eq!(frame.len(), 10); // Nine wire bytes for meta; native padding is omitted.
        assert!(PaddedMeta::__qsl_region_view(&frame[..9]).is_err());
        let copied = PaddedMeta::deserialize_transmutate_copy(&frame[..9]).unwrap();
        assert_eq!((copied.id, copied.value), (7, 3));
        let decoded = PaddedNeighbor::deserialize_checked(&frame).unwrap();
        assert_eq!(
            (decoded.meta.id, decoded.meta.value, decoded.neighbor),
            (7, 3, 99)
        );
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
        let mut cache = qsl::storage::CachedFrame::<65539>::new();
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
    fn lifetime_native_array_and_raw_text_borrow_the_aligned_frame() {
        let native = ArrayMeta {
            values: [1, 256, u16::MAX],
        };
        let value = BorrowNative {
            ArrayMeta: &native,
            text: "abc",
        };
        let bytes = value.serialize_checked().unwrap();
        #[repr(align(8))]
        struct Aligned([u8; 32]);
        let mut aligned = Aligned([0; 32]);
        aligned.0[..bytes.len()].copy_from_slice(&bytes);
        let decoded = BorrowNative::deserialize_checked(&aligned.0[..bytes.len()]).unwrap();
        assert_eq!(decoded.ArrayMeta.values, native.values);
        assert_eq!(
            decoded.ArrayMeta as *const ArrayMeta as *const u8,
            aligned.0.as_ptr()
        );
        assert_eq!(decoded.text.as_ptr(), unsafe { aligned.0.as_ptr().add(10) });
        let owned = ArrayMeta::deserialize_transmutate_copy(&aligned.0[..6]).unwrap();
        assert_eq!(owned.values, native.values);
    }
    #[test]
    fn prepared_required_heap_buffers_perform_no_allocation() {
        const {
            assert!(<HeapFixed as qsl::Serializable>::REUSES_STORAGE);
            assert!(<HeapLz as qsl::Serializable>::REUSES_STORAGE);
        }
        let source = HeapFixed {
            text: "Я".repeat(512),
            values: vec![65535; 200],
        };
        let frame = source.serialize_checked().unwrap();
        let mut output = HeapFixed {
            text: String::with_capacity(1024),
            values: Vec::with_capacity(200),
        };
        let pointers = (output.text.as_ptr(), output.values.as_ptr());
        assert_eq!(
            allocation_probe::count(|| unsafe {
                HeapFixed::deserialize_into(frame.as_ptr(), frame.len(), &mut output);
            }),
            0
        );
        assert_eq!(output.text, source.text);
        assert_eq!(output.values, source.values);
        assert_eq!((output.text.as_ptr(), output.values.as_ptr()), pointers);
        let source = HeapLz {
            text: qsl::InlineString::try_boxed_from_str(&"abc".repeat(300)).unwrap(),
        };
        let frame = source.serialize_checked().unwrap();
        let mut output = HeapLz {
            text: qsl::InlineString::try_boxed_new().unwrap(),
        };
        let pointer = output.text.as_bytes().as_ptr();
        assert_eq!(
            allocation_probe::count(|| unsafe {
                HeapLz::deserialize_into(frame.as_ptr(), frame.len(), &mut output);
            }),
            0
        );
        assert_eq!(output.text.as_str(), source.text.as_str());
        assert_eq!(output.text.as_bytes().as_ptr(), pointer);
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
            nested: Nested {
                lead: 1,
                meta: Meta {
                    id: 42,
                    value: 7,
                    tag: *b"abcd",
                },
                end: 2,
            },
            name: "ok".into(),
        };
        let bytes = x.serialize_checked().unwrap();
        let native = Nested::deserialize_transmutate_copy(&bytes[..18]).unwrap();
        assert_eq!(
            (native.lead, native.meta.id, native.meta.tag, native.end),
            (1, 42, *b"abcd", 2)
        );
        assert!(!std::hint::black_box(Nested::NATIVE_WIRE_COMPATIBLE));
        assert!(Nested::__qsl_region_view(&bytes[..18]).is_err());
        let decoded = NestedRegion::deserialize_checked(&bytes).unwrap();
        assert_eq!((decoded.nested.meta.id, decoded.name.as_str()), (42, "ok"));
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
    fn typed_exact_loads_at_protected_page_boundaries() {
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
            let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
            macro_rules! check {
                ($load:ident,$n:literal) => {{
                    let source = mem.add(4096 - $n);
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), source, $n);
                    let mut expected = [0u8; 16];
                    expected[..$n].copy_from_slice(&bytes[..$n]);
                    assert_eq!(
                        qsl::storage::$load::<$n>(source) as u128,
                        u128::from_le_bytes(expected)
                    );
                }};
            }
            check!(load_u8, 1);
            check!(load_u16, 2);
            check!(load_u32, 3);
            check!(load_u32, 4);
            check!(load_u64, 5);
            check!(load_u64, 6);
            check!(load_u64, 7);
            check!(load_u64, 8);
            check!(load_u128, 9);
            check!(load_u128, 10);
            check!(load_u128, 11);
            check!(load_u128, 12);
            check!(load_u128, 13);
            check!(load_u128, 14);
            check!(load_u128, 15);
            check!(load_u128, 16);
            assert_ne!(VirtualFree(mem.cast(), 0, 0x8000), 0);
        }
    }
    #[test]
    fn typed_groups_restore_crossing_signed_fields_and_length_headers() {
        for signed in [i64::MIN, -1, 0, i64::MAX] {
            let x = RegisterCross {
                head: u64::MAX,
                middle: 0xFEDCBA98,
                short: 0x8765,
                byte: 0xF1,
                crossing: signed,
                tail: 0xE2,
            };
            let bytes = x.serialize_checked().unwrap();
            let checked = RegisterCross::deserialize_checked(&bytes).unwrap();
            let trusted = unsafe { RegisterCross::deserialize(&bytes) };
            let mut out = std::mem::MaybeUninit::uninit();
            unsafe {
                RegisterCross::deserialize_into(bytes.as_ptr(), bytes.len(), out.as_mut_ptr());
            }
            let into = unsafe { out.assume_init() };
            for decoded in [checked, trusted, into] {
                assert_eq!(
                    (
                        decoded.head,
                        decoded.middle,
                        decoded.short,
                        decoded.byte,
                        decoded.crossing,
                        decoded.tail
                    ),
                    (x.head, x.middle, x.short, x.byte, x.crossing, x.tail)
                );
            }
        }
        for len in [1, 255, 256, 1024] {
            let x = HeaderCross {
                head: u64::MAX,
                middle: 0xFEDCBA98,
                short: 0x8765,
                byte: 0xF1,
                text: "x".repeat(len),
            };
            let bytes = x.serialize_checked().unwrap();
            let checked = HeaderCross::deserialize_checked(&bytes).unwrap();
            let trusted = unsafe { HeaderCross::deserialize(&bytes) };
            assert_eq!(checked.text, x.text);
            assert_eq!(trusted.text, x.text);
            assert_eq!(
                (trusted.head, trusted.middle, trusted.short, trusted.byte),
                (x.head, x.middle, x.short, x.byte)
            );
            assert!(HeaderCross::deserialize_checked(&bytes[..bytes.len() - 1]).is_err());
        }
    }
    #[test]
    fn typed_qps_windows_restore_full_u64_at_a_bit_offset() {
        for wide in [None, Some(0), Some(1), Some(u64::MAX)] {
            for small in [None, Some(0), Some(15)] {
                let x = QpsCross::try_new(1, wide, small).unwrap();
                let len = x.serialized_size();
                let mut bytes = vec![0xA5; len + 16];
                let frame = x.serialize_quantized_to(&mut bytes[..len]).unwrap();
                let mut out = std::mem::MaybeUninit::uninit();
                let decoded = frame.decode_into(&mut out);
                assert_eq!(
                    (decoded.bit, decoded.wide(), decoded.small()),
                    (1, wide, small)
                );
                let checked = QpsCross::deserialize_checked(&bytes[..len]).unwrap();
                assert_eq!((checked.wide(), checked.small()), (wide, small));
                assert!(bytes[len..].iter().all(|&byte| byte == 0xA5));
            }
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
        let x = HeapLz {
            text: qsl::InlineString::try_boxed_from_str(&text).unwrap(),
        };
        let bytes = x.serialize_checked().unwrap();
        assert!(bytes.len() < text.len());
        assert_eq!(
            HeapLz::deserialize_checked(&bytes).unwrap().text.as_str(),
            text
        );
        let x = HeapInline {
            text: qsl::InlineString::try_boxed_from_str(&text).unwrap(),
        };
        let bytes = x.serialize_checked().unwrap();
        assert_eq!(
            HeapInline::deserialize_checked(&bytes)
                .unwrap()
                .text
                .as_str(),
            text
        );
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
        assert_eq!(y.text.as_ptr(), unsafe {
            bytes.as_ptr().add(3 + BorrowInline::TEXT_HEADER_BYTES)
        });
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
            CountedMeta: CountedMeta {
                id: 5,
                value: 3,
                tag: *b"abcd",
            },
        };
        let bytes = value.serialize_checked().unwrap();
        native_validation_probe::reset();
        let a = unsafe { CountedRegion::deserialize(&bytes) };
        let b = unsafe { CountedMeta::deserialize_transmutate_copy_unchecked(&bytes) };
        assert_eq!(
            (a.CountedMeta.id, a.CountedMeta.value, a.CountedMeta.tag),
            (5, 3, *b"abcd")
        );
        assert_eq!((b.id, b.value, b.tag), (5, 3, *b"abcd"));
        assert_eq!(native_validation_probe::calls(), 0);
        // Control: instrumentation is active; checked schema decoding validates wire leaves separately.
        unsafe {
            <CountedMeta as qsl::storage::NativeField>::validate_native_fields(
                std::ptr::addr_of!(value.CountedMeta).cast(),
            )
            .unwrap();
        }
        assert_eq!(native_validation_probe::calls(), 1);
        CountedRegion::deserialize_checked(&bytes).unwrap();
        let mut invalid = bytes;
        invalid[12] = 255;
        assert!(CountedRegion::deserialize_checked(&invalid).is_err());
        assert!(CountedMeta::deserialize_transmutate_copy(&invalid).is_err());
    }
    #[test]
    fn unchecked_regions_are_available_without_safety() {
        let value = RegionTrusted {
            Meta: Meta {
                id: 7,
                value: 3,
                tag: *b"abcd",
            },
        };
        let mut bytes = [0; 16];
        unsafe {
            value.serialize_into_unsafe(bytes.as_mut_ptr());
        }
        let decoded = unsafe { RegionTrusted::deserialize(&bytes) };
        let native = unsafe { Meta::deserialize_transmutate_copy_unchecked(&bytes) };
        assert_eq!(
            (decoded.Meta.id, decoded.Meta.value, decoded.Meta.tag),
            (7, 3, *b"abcd")
        );
        assert_eq!((native.id, native.value, native.tag), (7, 3, *b"abcd"));
        let padded = PaddedNeighbor {
            meta: PaddedMeta { id: 7, value: 3 },
            neighbor: 99,
        };
        let frame = padded.serialize_checked().unwrap();
        assert!(!std::hint::black_box(PaddedMeta::NATIVE_WIRE_COMPATIBLE));
        let copy = unsafe { PaddedMeta::deserialize_transmutate_copy_unchecked(&frame[..9]) };
        assert_eq!((copy.id, copy.value), (7, 3));
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
            Meta: Meta {
                id: 5,
                value: 3,
                tag: *b"abcd",
            },
            name: "ok".into(),
        };
        let frame = value.serialize_checked().unwrap();
        let native = Meta::deserialize_transmutate_copy(&frame[..16]).unwrap();
        assert_eq!((native.id, native.value, native.tag), (5, 3, *b"abcd"));
        let decoded = Region::deserialize_checked(&frame).unwrap();
        assert_eq!((decoded.Meta.id, decoded.name.as_str()), (5, "ok"));
        let value = Cache {
            id: 2,
            text: qsl::StackString::try_from_str("hi").unwrap(),
        };
        let mut cached = qsl::storage::CachedFrame::<28>::new();
        assert_eq!(
            value.serialize_cached(&mut cached).unwrap(),
            value.serialize_checked().unwrap()
        );
    }
    // Independent adversarial frame builder: one literal followed by legal
    // distance-one tokens. decoded may deliberately exceed the schema maximum.
    fn repeated_lz_frame(maximum: usize, decoded: usize, fixed: bool) -> Vec<u8> {
        assert!(decoded > 0);
        let width = if maximum <= 255 { 1 } else { 2 };
        let mut packed = vec![0, b'A'];
        let mut remaining = decoded - 1;
        while remaining >= 4 {
            let count = remaining.min(131);
            packed.push(128 + (count - 4) as u8);
            packed.push(1);
            if width == 2 {
                packed.push(0);
            }
            remaining -= count;
        }
        if remaining != 0 {
            packed.push((remaining - 1) as u8);
            packed.extend(std::iter::repeat_n(b'A', remaining));
        }
        let header = if maximum <= 127 {
            1
        } else if maximum <= 32767 {
            2
        } else {
            4
        };
        let mut frame = if fixed {
            let mut frame = vec![255];
            frame.extend_from_slice(&(packed.len() as u16).to_le_bytes()[..width]);
            frame
        } else {
            let word = (packed.len() as u32) | (1 << (header * 8 - 1));
            word.to_le_bytes()[..header].to_vec()
        };
        frame.extend_from_slice(&packed);
        frame
    }

    fn checked_error_preserves_destination<T>(
        frame: &[u8],
        decode: for<'input, 'out> fn(
            &'input [u8],
            &'out mut std::mem::MaybeUninit<T>,
        ) -> qsl::QslResult<&'out mut T>,
    ) {
        #[repr(C)]
        struct Guarded<T> {
            before: [u8; 32],
            output: std::mem::MaybeUninit<T>,
            after: [u8; 32],
        }
        let mut target = Guarded {
            before: [0xA5; 32],
            output: std::mem::MaybeUninit::<T>::uninit(),
            after: [0xA5; 32],
        };
        // Initialize bytes, not a T value. A failed checked decode must not
        // publish a T or alter this destination; MaybeUninit does not drop T.
        unsafe {
            std::ptr::write_bytes(
                target.output.as_mut_ptr().cast::<u8>(),
                0xA5,
                std::mem::size_of::<T>(),
            );
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            decode(frame, &mut target.output).map(|_| ())
        }));
        assert!(matches!(result, Ok(Err(_))));
        let bytes = unsafe {
            std::slice::from_raw_parts(
                target.output.as_ptr().cast::<u8>(),
                std::mem::size_of::<T>(),
            )
        };
        assert!(bytes.iter().all(|&byte| byte == 0xA5));
        assert_eq!(target.before, [0xA5; 32]);
        assert_eq!(target.after, [0xA5; 32]);
    }

    #[test]
    fn forged_lz_lengths_cannot_expand_beyond_inline_or_heap_limits() {
        macro_rules! check {
            ($ty:ty,$maximum:literal,$fixed:literal) => {{
                let valid = repeated_lz_frame($maximum, $maximum, $fixed);
                assert_eq!(
                    <$ty>::deserialize_checked(&valid).unwrap().text.len(),
                    $maximum
                );
                <$ty>::validate(&valid).unwrap();
                let overflow = repeated_lz_frame($maximum, $maximum + 1, $fixed);
                assert!(matches!(
                    <$ty>::validate(&overflow),
                    Err(qsl::QslError::InvalidValue)
                ));
                assert!(matches!(
                    <$ty>::deserialize_checked(&overflow),
                    Err(qsl::QslError::InvalidValue)
                ));
                checked_error_preserves_destination(&overflow, <$ty>::deserialize_checked_into);
                for end in 0..overflow.len() {
                    assert!(<$ty>::deserialize_checked(&overflow[..end]).is_err());
                }
            }};
        }
        check!(TinyLz, 100, false);
        check!(Compressed, 255, false);
        check!(HeapLz, 1024, false);
        check!(FixedLz, 100, true);
        check!(HeapFixedLz, 1024, true);
        // A tiny forged compressed span cannot authorize a larger literal run.
        for frame in [&[0x81, 0x7F][..], &[0x84, 0, b'A', 0xFF, 1][..]] {
            assert!(TinyLz::deserialize_checked(frame).is_err());
            checked_error_preserves_destination(frame, TinyLz::deserialize_checked_into);
        }
        // The declared packed length exceeds the available input or schema.
        for frame in [&[0x85, 0, b'A', 0xDF, 1][..], &[0xFF][..]] {
            assert!(TinyLz::validate(frame).is_err());
            checked_error_preserves_destination(frame, TinyLz::deserialize_checked_into);
        }
    }

    #[test]
    fn invalid_first_lz_token_is_rejected_before_heap_reservation() {
        // Truncated literal, truncated distance, zero distance, missing history.
        for frame in [
            &[1, 0x80, 0x7F][..],
            &[1, 0x80, 0xFF][..],
            &[3, 0x80, 0x80, 0, 0][..],
            &[3, 0x80, 0xFF, 1, 0][..],
        ] {
            let allocations = allocation_probe::count(|| {
                assert!(HeapLz::deserialize_checked(frame).is_err());
                assert!(HeapLz::validate(frame).is_err());
            });
            assert_eq!(allocations, 0);
            checked_error_preserves_destination(frame, HeapLz::deserialize_checked_into);
        }
    }

    #[test]
    fn compressed_fields_share_the_actual_restoration_budget() {
        let first = repeated_lz_frame(65535, 40000, false);
        let second = repeated_lz_frame(65535, 40000, false);
        let mut frame = first;
        frame.extend_from_slice(&second);
        assert!(frame.len() < TwoLz::MAX_DATA_SIZE);
        assert!(matches!(
            TwoLz::validate(&frame),
            Err(qsl::QslError::SizeLimitExceeded)
        ));
        assert!(matches!(
            TwoLz::deserialize_checked(&frame),
            Err(qsl::QslError::SizeLimitExceeded)
        ));
        checked_error_preserves_destination(&frame, TwoLz::deserialize_checked_into);
    }

    #[test]
    fn lz_remains_lossless() {
        let text = "abc".repeat(64);
        let x = Compressed {
            text: qsl::InlineString::try_boxed_from_str(&text).unwrap(),
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
    fn region_slots_return_exhaustion_and_resume_after_drop() {
        let a = RT { id: 1 }.serialize().unwrap();
        let b = RT { id: 2 }.serialize().unwrap();
        assert!(matches!(
            RT { id: 3 }.serialize(),
            Err(qsl::QslError::PoolExhausted)
        ));
        assert_eq!(a.as_slice(), 1u32.to_le_bytes());
        assert_eq!(b.as_slice(), 2u32.to_le_bytes());
        drop(a);
        let c = RT { id: 3 }.serialize().unwrap();
        assert_eq!(c.as_slice(), 3u32.to_le_bytes());
        assert_eq!(b.as_slice(), 2u32.to_le_bytes());
    }
}
