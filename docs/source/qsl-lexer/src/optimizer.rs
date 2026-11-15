//! Deterministic physical-layout planning. The logical and wire field lists stay intact.
//! Cost is a layout proxy, not an assertion about CPU latency. ISA candidates stay disabled
//! until measured; no runtime dispatch or heuristic is introduced by this planner.
pub use qsl_core::optimizer::kernels;
mod memory;
pub mod registers;
use crate::{
    layout::{native_type_layout, StorageKind},
    QslError, QslField, QslResult, QslSchema, QslType,
};
pub use memory::{CopySegment, FieldLocation, MemoryKind, MemoryPlan, MemorySpan, PointingPlan};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectLayout {
    pub bytes: usize,
    pub alignment: usize,
}
fn align(n: usize, a: usize) -> Option<usize> {
    n.checked_add(a - 1).map(|n| n & !(a - 1))
}
fn inline(n: usize, e: ObjectLayout, word: usize) -> Option<ObjectLayout> {
    let a = e.alignment.max(word);
    Some(ObjectLayout {
        bytes: align(
            align(word, e.alignment)?.checked_add(n.checked_mul(e.bytes)?)?,
            a,
        )?,
        alignment: a,
    })
}
/// x86-64 codegen target facts; never interpreted as initialized payload spans.
pub fn object_layout(f: &QslField, s: &QslSchema) -> Option<ObjectLayout> {
    if f.optional {
        return None;
    }
    let scalar = |t: &QslType| {
        native_type_layout(t, &s.options.native_types).map(|l| ObjectLayout {
            bytes: l.native_bytes,
            alignment: l.alignment,
        })
    };
    match f.storage {
        StorageKind::StackString(n) => {
            let w = crate::layout::length_storage_bytes(f);
            Some(ObjectLayout {
                bytes: align(w.checked_add(n)?, w)?,
                alignment: w,
            })
        }
        StorageKind::StackArray(n) => {
            let QslType::RangedArray { element, .. } = &f.data else {
                return None;
            };
            inline(n, scalar(element)?, crate::layout::length_storage_bytes(f))
        }
        StorageKind::StrView => Some(ObjectLayout {
            bytes: 16,
            alignment: 8,
        }),
        StorageKind::HeapInlineString(_) | StorageKind::NativeView => Some(ObjectLayout {
            bytes: 8,
            alignment: 8,
        }),
        StorageKind::HeapString => Some(ObjectLayout {
            bytes: 24,
            alignment: 8,
        }),
        StorageKind::FixedArray(_) => scalar(&f.data),
        StorageKind::Default => match &f.data {
            QslType::String(_)
            | QslType::RangedString { .. }
            | QslType::Array(..)
            | QslType::RangedArray { .. }
            | QslType::FixedArray(..)
                if s.borrow_strings() =>
            {
                Some(ObjectLayout {
                    bytes: 16,
                    alignment: 8,
                })
            }
            QslType::String(_)
            | QslType::RangedString { .. }
            | QslType::Array(..)
            | QslType::RangedArray { .. } => Some(ObjectLayout {
                bytes: 24,
                alignment: 8,
            }),
            QslType::BorrowedString { .. } => Some(ObjectLayout {
                bytes: 16,
                alignment: 8,
            }),
            QslType::CompressedString { max, .. } | QslType::InlineRawString { max, .. } => {
                let w = crate::layout::length_storage_bytes(f);
                Some(ObjectLayout {
                    bytes: align(w.checked_add(*max)?, w)?,
                    alignment: w,
                })
            }
            // Default FixedArray historically lowers to Vec; only FixedArray storage is native.
            QslType::FixedArray(..) => Some(ObjectLayout {
                bytes: 24,
                alignment: 8,
            }),
            QslType::Padding(_) | QslType::BitPadding(_) | QslType::None => Some(ObjectLayout {
                bytes: 0,
                alignment: 1,
            }),
            _ => scalar(&f.data),
        },
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalPlan {
    pub order: Vec<usize>,
    pub offsets: Vec<usize>,
    pub bytes: Option<usize>,
    pub alignment: usize,
    pub padding: usize,
    pub exact_search: bool,
    pub reason: &'static str,
}
fn evaluate(order: &[usize], fields: &[ObjectLayout]) -> Option<(usize, usize, Vec<usize>)> {
    let mut pos = 0usize;
    let mut a = 1;
    let mut offsets = vec![0; fields.len()];
    let mut used = 0usize;
    for &i in order {
        let f = fields[i];
        a = a.max(f.alignment);
        pos = align(pos, f.alignment)?;
        offsets[i] = pos;
        pos = pos.checked_add(f.bytes)?;
        used = used.checked_add(f.bytes)?;
    }
    let bytes = align(pos, a)?;
    Some((bytes, bytes - used, offsets))
}
impl PhysicalPlan {
    pub fn analyze(s: &QslSchema) -> QslResult<Self> {
        let original = (0..s.fields.len()).collect::<Vec<_>>();
        let pinned = |reason| Self {
            order: original.clone(),
            offsets: Vec::new(),
            bytes: None,
            alignment: 1,
            padding: 0,
            exact_search: false,
            reason,
        };
        if !s.options.auto_layout {
            return Ok(pinned("physical auto is disabled"));
        }
        if s.options.external_definition
            || s.options.generated_region
            || s.options.native_overlay()
            || s.options.transmutate
            || !s.options.regions.is_empty()
        {
            return Ok(pinned("ABI or native region pins physical layout"));
        }
        let Some(layouts) = s
            .fields
            .iter()
            .map(|f| object_layout(f, s))
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(pinned("opaque or optional representation pins layout"));
        };
        let mut canonical = original.clone();
        canonical.sort_by(|&a, &b| {
            if s.options.dynamic.is_some() {
                s.fields[a]
                    .tag
                    .cmp(&s.fields[b].tag)
                    .then_with(|| s.fields[a].name.cmp(&s.fields[b].name))
            } else {
                s.fields[a].name.cmp(&s.fields[b].name)
            }
        });
        let mut best = canonical.clone();
        let mut best_cost = evaluate(&best, &layouts).ok_or(QslError::LengthOverflow)?;
        let exact = best.len() <= 8;
        if exact {
            fn search(
                prefix: usize,
                order: &mut [usize],
                layouts: &[ObjectLayout],
                ranks: &[usize],
                best: &mut Vec<usize>,
                cost: &mut (usize, usize, Vec<usize>),
            ) {
                if prefix == order.len() {
                    if let Some(candidate) = evaluate(order, layouts) {
                        let keys = |o: &[usize]| o.iter().map(|&i| ranks[i]).collect::<Vec<_>>();
                        if (candidate.0, candidate.1, keys(order)) < (cost.0, cost.1, keys(best)) {
                            *cost = candidate;
                            *best = order.to_vec();
                        }
                    }
                    return;
                }
                for i in prefix..order.len() {
                    order.swap(prefix, i);
                    search(prefix + 1, order, layouts, ranks, best, cost);
                    order.swap(prefix, i);
                }
            }
            let mut ranks = vec![0; canonical.len()];
            for (rank, &i) in canonical.iter().enumerate() {
                ranks[i] = rank;
            }
            search(
                0,
                &mut canonical,
                &layouts,
                &ranks,
                &mut best,
                &mut best_cost,
            );
        } else {
            canonical.sort_by(|&a, &b| {
                layouts[b]
                    .alignment
                    .cmp(&layouts[a].alignment)
                    .then_with(|| layouts[b].bytes.cmp(&layouts[a].bytes))
                    .then_with(|| s.fields[a].name.cmp(&s.fields[b].name))
                    .then_with(|| s.fields[a].tag.cmp(&s.fields[b].tag))
            });
            let candidate = evaluate(&canonical, &layouts).ok_or(QslError::LengthOverflow)?;
            if (candidate.0, candidate.1) < (best_cost.0, best_cost.1) {
                best = canonical;
                best_cost = candidate;
            }
        }
        Ok(Self {
            order: best,
            offsets: best_cost.2,
            bytes: Some(best_cost.0),
            alignment: layouts.iter().map(|l| l.alignment).max().unwrap_or(1),
            padding: best_cost.1,
            exact_search: exact,
            reason: if exact {
                "bounded exact layout search"
            } else {
                "stable alignment heuristic"
            },
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_reorder_preserves_logical_fields_and_hash() {
        let mut s = QslSchema {
            name: "P".into(),
            max_tag: 3,
            ..Default::default()
        };
        for (i, bits) in [8, 64, 8].into_iter().enumerate() {
            s.fields.push(QslField {
                name: format!("f{i}"),
                tag: i + 1,
                bits: bits as usize,
                data: QslType::Integer {
                    bits,
                    signed: false,
                    min: 0,
                },
                ..Default::default()
            });
        }
        let hash = s.hashes(Default::default()).unwrap();
        s.options.auto_layout = true;
        let plan = PhysicalPlan::analyze(&s).unwrap();
        assert_eq!(plan.bytes, Some(16));
        assert_eq!(s.fields[1].name, "f1");
        assert_eq!(s.hashes(Default::default()).unwrap(), hash);
        assert_eq!(plan, PhysicalPlan::analyze(&s).unwrap());
        s.options.transmutate = true;
        assert!(PhysicalPlan::analyze(&s).unwrap().bytes.is_none());
    }
}
