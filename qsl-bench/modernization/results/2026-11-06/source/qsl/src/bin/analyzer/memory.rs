//! Native repr(C) object sizes for this analyzer's target; allocator capacity is separate.
use super::json;
use qsl::{
    modernization::{field_rust_type, StorageKind},
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
        _ => return None,
    })
}
fn field(f: &qsl::QslField, borrowed: bool, aggressive: bool) -> Option<M> {
    if let Some(r) = qsl::modernization::optional_representation(f, aggressive) {
        if let Some(niche) = r.niche {
            let bits = niche
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .ok()?;
            let t = QslType::Integer {
                bits,
                signed: false,
                min: 0,
            };
            return scalar(&t, false);
        }
        return scalar(&f.data, false);
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
            QslType::GuardString { min, max } => (min, max),
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
            base(round(4usize.checked_add(*max)?, 4)?, 4)
        }
        QslType::FixedArray(e, n)
        | QslType::Array(e, n)
        | QslType::RangedArray {
            element: e, min: n, ..
        } => {
            let ef = qsl::QslField {
                data: (**e).clone(),
                ..Default::default()
            };
            let element = field(&ef, false, aggressive)?;
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
                        let a = element.align.max(4);
                        base(
                            round(
                                round(4, element.align)?
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
        | QslType::GuardString { min, .. }
        | QslType::CopyGuardString { min, .. }
        | QslType::BinaryString { min, .. } => {
            let max = match &f.data {
                QslType::RangedString { max, .. } => Some(*max),
                QslType::GuardString { max, .. }
                | QslType::CopyGuardString { max, .. }
                | QslType::BinaryString { max, .. } => *max,
                _ => None,
            };
            if let StorageKind::StackString(n) = f.storage {
                base(round(4usize.checked_add(n)?, 4)?, 4)
            } else if borrowed
                || f.storage == StorageKind::StrView
                || matches!(f.data, QslType::BinaryString { .. })
            {
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
        QslType::Custom { .. } => return None,
        _ => return None,
    };
    if f.optional {
        m.heap_min = 0;
        m.borrow_min = 0;
        // String/Vec and slices use the guaranteed null-pointer niche of their pointer.
        let niche = matches!(f.storage, StorageKind::StrView)
            || ((borrowed || matches!(f.data, QslType::BinaryString { .. }))
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
    for f in &s.fields {
        let m = field(f, s.borrow_strings(), s.options.aggressive);
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
        let storage_type =
            if let Some(r) = qsl::modernization::optional_representation(f, s.options.aggressive) {
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
    let bitmap = s
        .fields
        .iter()
        .filter(|f| {
            qsl::modernization::optional_representation(f, s.options.aggressive)
                .is_some_and(|r| r.niche.is_none())
        })
        .count();
    if bitmap != 0 {
        let bytes = if bitmap <= 8 {
            1
        } else if bitmap <= 16 {
            2
        } else if bitmap <= 32 {
            4
        } else {
            8
        };
        align = align.max(bytes);
        let at = size.and_then(|n| round(n, bytes));
        size = at.and_then(|n| n.checked_add(bytes));
        rows.push(format!("{{\"name\":\"__qsl_presence\",\"rustType\":\"u{}\",\"offset\":{},\"size\":{bytes},\"alignment\":{bytes},\"heapMinimum\":0,\"heapMaximum\":0,\"borrowedMinimum\":0,\"borrowedMaximum\":0}}",bytes*8,n(at)));
    }
    let size = size.and_then(|n| round(n, align));
    format!("{{\"layout\":\"repr(C)\",\"target\":{},\"pointerBytes\":{},\"objectBytes\":{},\"alignment\":{align},\"ownedDataMinimum\":{heap_min},\"ownedDataMaximum\":{},\"borrowedDataMinimum\":{borrow_min},\"borrowedDataMaximum\":{},\"allocatorCapacityIncluded\":false,\"fields\":[{}]}}",json(std::env::consts::ARCH),size_of::<usize>(),n(size),n(heap_max),n(borrow_max),rows.join(","))
}
