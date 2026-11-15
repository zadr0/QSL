//! Native repr(C) object sizes for this analyzer's target; allocator capacity is separate.
use super::json;
use qsl::{
    layout::{field_rust_type, StorageKind},
    QslSchema, QslType,
};
use std::mem::{align_of, size_of};
#[derive(Clone, Copy)]
struct M {
    size: usize,
    align: usize,
    heap_min: usize,
    heap_max: Option<usize>,
    borrow_min: usize,
    borrow_max: Option<usize>,
}
fn round(n: usize, a: usize) -> Option<usize> {
    n.checked_add(a - 1).map(|n| n & !(a - 1))
}
fn add(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    a?.checked_add(b?)
}
fn base(size: usize, align: usize) -> M {
    M {
        size,
        align,
        heap_min: 0,
        heap_max: Some(0),
        borrow_min: 0,
        borrow_max: Some(0),
    }
}
fn scalar(t: &QslType, optional: bool) -> Option<M> {
    let name = t.primitive().ok()??;
    macro_rules! native {
        ($t:ty) => {
            if optional {
                base(size_of::<Option<$t>>(), align_of::<Option<$t>>())
            } else {
                base(size_of::<$t>(), align_of::<$t>())
            }
        };
    }
    Some(match name {
        "u8" => native!(u8),
        "i8" => native!(i8),
        "u16" => native!(u16),
        "i16" => native!(i16),
        "u32" => native!(u32),
        "i32" => native!(i32),
        "u64" => native!(u64),
        "i64" => native!(i64),
        "u128" => native!(u128),
        "i128" => native!(i128),
        _ => return None,
    })
}
fn field(
    f: &qsl::QslField,
    borrowed: bool,
    aggressive: bool,
    types: &std::collections::BTreeMap<String, qsl::layout::NativeTypeLayout>,
) -> Option<M> {
    if let Some(r) = qsl::layout::optional_representation(f, aggressive) {
        if let Some(niche) = r.niche {
            let bits = niche
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .ok()?;
            let t = QslType::DomainInteger(
                qsl::domain::IntegerDomain::unsigned(bits, 0, qsl::domain::low_mask(bits))
                    .ok()?
                    .into(),
            );
            return scalar(&t, false);
        }
        return scalar(&f.data, false);
    }
    if let StorageKind::HeapInlineString(capacity) = f.storage {
        // Box owns a full bounded object; useful UTF-8 length is not allocation size.
        let word = qsl::layout::length_storage_bytes(f);
        let owned = round(word.checked_add(capacity)?, word)?;
        let mut m = base(size_of::<Box<u8>>(), align_of::<Box<u8>>());
        m.heap_min = if f.optional { 0 } else { owned };
        m.heap_max = Some(owned);
        return Some(m);
    }
    if f.storage == StorageKind::HeapString {
        let (min, max) = match f.data {
            QslType::FixedString(n) | QslType::FixedCompressedString(n) => (n, Some(n)),
            QslType::InlineRawString { min, max }
            | QslType::CompressedString { min, max }
            | QslType::RangedString { min, max } => (min, Some(max)),
            QslType::String(min) => (min, None),
            _ => return None,
        };
        let mut m = base(size_of::<String>(), align_of::<String>());
        m.heap_min = if f.optional { 0 } else { min };
        m.heap_max = max;
        return Some(m);
    }
    if f.storage == StorageKind::StrView {
        let (min, max) = match f.data {
            QslType::FixedString(n) => (n, Some(n)),
            QslType::InlineRawString { min, max } => (min, Some(max)),
            QslType::BorrowedString { min, max } => (min, max),
            _ => return None,
        };
        let mut m = base(size_of::<&str>(), align_of::<&str>());
        m.borrow_min = if f.optional { 0 } else { min };
        m.borrow_max = max;
        return Some(m);
    }
    if let Some(m) = scalar(&f.data, f.optional) {
        return Some(m);
    }
    let mut m = match &f.data {
        QslType::None | QslType::Padding(_) => base(0, 1),
        QslType::FixedString(n) | QslType::FixedCompressedString(n) => base(*n, 1),
        QslType::InlineRawString { max, .. } | QslType::CompressedString { max, .. } => {
            let w = qsl::layout::length_storage_bytes(f);
            base(round(w.checked_add(*max)?, w)?, w)
        }
        QslType::FixedArray(e, n)
        | QslType::Array(e, n)
        | QslType::RangedArray {
            element: e, min: n, ..
        } => {
            let ef = qsl::QslField {
                storage: if matches!(f.storage, StorageKind::FixedArray(_)) {
                    if let QslType::FixedArray(_, n) = e.as_ref() {
                        StorageKind::FixedArray(*n)
                    } else {
                        StorageKind::Default
                    }
                } else {
                    StorageKind::Default
                },
                data: (**e).clone(),
                ..Default::default()
            };
            let element = field(&ef, false, aggressive, types)?;
            let max = match &f.data {
                QslType::FixedArray(_, n) | QslType::RangedArray { max: n, .. } => Some(*n),
                _ => None,
            };
            if borrowed && !matches!(f.storage, StorageKind::FixedArray(_)) {
                let mut m = base(size_of::<&[u8]>(), align_of::<&[u8]>());
                m.borrow_min = n.checked_mul(element.size)?;
                m.borrow_max = max.and_then(|n| n.checked_mul(element.size));
                m
            } else {
                match f.storage {
                    StorageKind::FixedArray(n) => base(n.checked_mul(element.size)?, element.align),
                    StorageKind::StackArray(n) => {
                        let word = qsl::layout::length_storage_bytes(f);
                        let a = element.align.max(word);
                        base(
                            round(
                                round(word, element.align)?
                                    .checked_add(n.checked_mul(element.size)?)?,
                                a,
                            )?,
                            a,
                        )
                    }
                    _ => {
                        let mut m = base(size_of::<Vec<u8>>(), align_of::<Vec<u8>>());
                        m.heap_min = n.checked_mul(element.size)?;
                        m.heap_max = max.and_then(|n| n.checked_mul(element.size));
                        m
                    }
                }
            }
        }
        QslType::String(min)
        | QslType::RangedString { min, .. }
        | QslType::BorrowedString { min, .. } => {
            let max = match &f.data {
                QslType::RangedString { max, .. } => Some(*max),
                QslType::BorrowedString { max, .. } => *max,
                _ => None,
            };
            if let StorageKind::StackString(n) = f.storage {
                {
                    let w = qsl::layout::length_storage_bytes(f);
                    base(round(w.checked_add(n)?, w)?, w)
                }
            } else if borrowed || f.storage == StorageKind::StrView || false {
                let mut m = base(size_of::<&str>(), align_of::<&str>());
                m.borrow_min = *min;
                m.borrow_max = max;
                m
            } else {
                let mut m = base(size_of::<String>(), align_of::<String>());
                m.heap_min = *min;
                m.heap_max = max;
                m
            }
        }
        QslType::Custom { rust_type, .. } => {
            // An arbitrary custom Option may have an unknown niche. Never guess it.
            if f.optional {
                return None;
            }
            let layout = types.get(rust_type)?;
            if f.storage == StorageKind::NativeView {
                let mut m = base(size_of::<&u8>(), align_of::<&u8>());
                m.borrow_min = layout.wire_bytes;
                m.borrow_max = Some(layout.wire_bytes);
                m
            } else {
                base(layout.native_bytes, layout.alignment)
            }
        }
        _ => return None,
    };
    if f.optional {
        m.heap_min = 0;
        m.borrow_min = 0;
        // String/Vec and slices use the guaranteed null-pointer niche of their pointer.
        let niche = f.shifted_length
            || matches!(f.storage, StorageKind::StrView)
            || ((borrowed || false)
                && !matches!(
                    f.data,
                    QslType::FixedString(_) | QslType::FixedCompressedString(_)
                ))
            || matches!(f.storage, StorageKind::Default)
                && matches!(
                    f.data,
                    QslType::String(_)
                        | QslType::RangedString { .. }
                        | QslType::Array(..)
                        | QslType::RangedArray { .. }
                );
        if !niche {
            m.size = round(m.size.checked_add(m.align)?, m.align)?;
        }
    }
    Some(m)
}
fn n(v: Option<usize>) -> String {
    v.map_or("null".into(), |n| n.to_string())
}
pub(super) fn describe(s: &QslSchema) -> String {
    let mut size = Some(0usize);
    let mut align = 1;
    let mut heap_min = 0usize;
    let mut heap_max = Some(0usize);
    let mut borrow_min = 0usize;
    let mut borrow_max = Some(0usize);
    let mut rows = Vec::new();
    let order = qsl::optimizer::PhysicalPlan::analyze(s)
        .map(|p| p.order)
        .unwrap_or_else(|_| (0..s.fields.len()).collect());
    for index in order {
        let f = &s.fields[index];
        let private = s.options.aggressive_bitmap && f.optional;
        let mut slot = f.clone();
        if private {
            slot.optional = false;
        }
        let m = field(
            &slot,
            s.borrow_strings(),
            s.options.aggressive_shift_enabled(),
            &s.options.native_types,
        )
        .map(|mut m| {
            if private {
                m.heap_min = 0;
                m.borrow_min = 0;
            }
            m
        });
        let offset = m.and_then(|m| round(size?, m.align));
        if let Some(m) = m {
            align = align.max(m.align);
            size = offset.and_then(|n| n.checked_add(m.size));
            heap_min = heap_min.saturating_add(m.heap_min);
            heap_max = add(heap_max, m.heap_max);
            borrow_min = borrow_min.saturating_add(m.borrow_min);
            borrow_max = add(borrow_max, m.borrow_max);
        } else {
            size = None;
            heap_max = None;
            borrow_max = None;
        }
        let storage_type = if private {
            format!(
                "MaybeUninit<{}>",
                field_rust_type(f, s.borrow_strings()).unwrap_or("?".into())
            )
        } else if let Some(r) =
            qsl::layout::optional_representation(f, s.options.aggressive_shift_enabled())
        {
            r.niche.map_or(r.rust_type, |n| format!("Option<{n}>"))
        } else {
            let ty = field_rust_type(f, s.borrow_strings()).unwrap_or("?".into());
            if f.optional {
                format!("Option<{ty}>")
            } else {
                ty
            }
        };
        rows.push(format!("{{\"name\":{},\"rustType\":{},\"offset\":{},\"size\":{},\"alignment\":{},\"heapMinimum\":{},\"heapMaximum\":{},\"borrowedMinimum\":{},\"borrowedMaximum\":{}}}",json(&f.name),json(&storage_type),n(offset),n(m.map(|m|m.size)),n(m.map(|m|m.align)),n(m.map(|m|m.heap_min)),n(m.and_then(|m|m.heap_max)),n(m.map(|m|m.borrow_min)),n(m.and_then(|m|m.borrow_max))));
    }
    let bitmap = if s.options.aggressive_bitmap {
        s.fields
            .iter()
            .filter(|f| f.optional)
            .map(|f| f.tag)
            .max()
            .unwrap_or(0)
    } else {
        s.fields
            .iter()
            .filter(|f| {
                qsl::layout::optional_representation(f, s.options.aggressive_shift_enabled())
                    .is_some_and(|r| r.niche.is_none())
            })
            .count()
    };
    if bitmap != 0 {
        let bytes = if bitmap <= 8 {
            1
        } else if bitmap <= 16 {
            2
        } else if bitmap <= 32 {
            4
        } else if bitmap <= 64 {
            8
        } else {
            16
        };
        let mask_alignment = if bytes == 16 {
            align_of::<u128>()
        } else {
            bytes
        };
        align = align.max(mask_alignment);
        let at = size.and_then(|n| round(n, mask_alignment));
        size = at.and_then(|n| n.checked_add(bytes));
        rows.push(format!("{{\"name\":\"__qsl_presence\",\"rustType\":\"u{}\",\"offset\":{},\"size\":{bytes},\"alignment\":{mask_alignment},\"heapMinimum\":0,\"heapMaximum\":0,\"borrowedMinimum\":0,\"borrowedMaximum\":0}}",bytes*8,n(at)));
    }
    align = align.max(s.options.native_alignment.unwrap_or(1));
    let size = size.and_then(|n| round(n, align));
    format!("{{\"layout\":\"repr(C)\",\"target\":{},\"pointerBytes\":{},\"objectBytes\":{},\"alignment\":{align},\"ownedDataMinimum\":{heap_min},\"ownedDataMaximum\":{},\"borrowedDataMinimum\":{borrow_min},\"borrowedDataMaximum\":{},\"allocatorCapacityIncluded\":false,\"fields\":[{}]}}",json(std::env::consts::ARCH),size_of::<usize>(),n(size),n(heap_max),n(borrow_max),rows.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heap_inline_reports_pointer_and_full_heap_object_separately() {
        for optional in [false, true] {
            let source = if optional {
                "&P[safety]{str?(0..512) text|decompress=true|;}"
            } else {
                "&P[safety]{str(0..512) text|decompress=true|;}"
            };
            let s = qsl::layout::lower_storage(&qsl::parse_qsl(source).unwrap().remove(0), 256)
                .unwrap();
            let actual = field(&s.fields[0], false, false, &Default::default()).unwrap();
            assert_eq!(actual.size, size_of::<Box<qsl::InlineString<512>>>());
            let heap = size_of::<qsl::InlineString<512>>();
            assert_eq!(actual.heap_min, if optional { 0 } else { heap });
            assert_eq!(actual.heap_max, Some(heap));
            let json = describe(&s);
            assert!(json.contains(&format!(
                "\"objectBytes\":{}",
                size_of::<Box<qsl::InlineString<512>>>()
            )));
            assert!(json.contains("Box<::qsl::InlineString<512>>"));
        }
    }
    #[test]
    fn shifted_array_layout_matches_target_nonzero_word_and_element_alignment() {
        use core::num::NonZeroU8;
        for (ty, size, align) in [
            (
                "u8",
                size_of::<qsl::InlineArray<u8, 8, NonZeroU8>>(),
                align_of::<qsl::InlineArray<u8, 8, NonZeroU8>>(),
            ),
            (
                "u16",
                size_of::<qsl::InlineArray<u16, 8, NonZeroU8>>(),
                align_of::<qsl::InlineArray<u16, 8, NonZeroU8>>(),
            ),
            (
                "u64",
                size_of::<qsl::InlineArray<u64, 8, NonZeroU8>>(),
                align_of::<qsl::InlineArray<u64, 8, NonZeroU8>>(),
            ),
        ] {
            for optional in [false, true] {
                let source = format!(
                    "&P[safety,aggressive[shift=true]]{{array<{ty}>{}(0..8) values;}}",
                    if optional { "?" } else { "" }
                );
                let s =
                    qsl::layout::lower_storage(&qsl::parse_qsl(&source).unwrap().remove(0), 256)
                        .unwrap();
                let actual = field(&s.fields[0], false, true, &Default::default()).unwrap();
                assert_eq!((actual.size, actual.align), (size, align), "{source}");
                assert_eq!(actual.heap_max, Some(0));
                assert!(describe(&s).contains(&format!("\"objectBytes\":{size}")));
            }
        }
    }
    #[test]
    fn private_wide_slots_and_physical_order_match_real_rust_layout() {
        #[repr(C)]
        struct Slots {
            id: u128,
            x: std::mem::MaybeUninit<u128>,
            y: std::mem::MaybeUninit<i128>,
            mask: u8,
        }
        let s = qsl::parse_qsl("&P[safety,aggressive[bitmap]]{u128 id;u128? x;i128? y;}")
            .unwrap()
            .remove(0);
        let json = describe(&s);
        assert!(json.contains(&format!("\"objectBytes\":{}", size_of::<Slots>())));
        assert!(json.contains("MaybeUninit<u128>"));
        assert!(json.contains(&format!("\"offset\":{}", std::mem::offset_of!(Slots, mask))));
        #[repr(C)]
        struct Physical {
            b: u64,
            a: u8,
            c: u8,
        }
        let s = qsl::parse_qsl("&P[auto]{u8 a;u64 b;u8 c;}")
            .unwrap()
            .remove(0);
        let json = describe(&s);
        assert!(json.contains(&format!("\"objectBytes\":{}", size_of::<Physical>())));
    }
}
