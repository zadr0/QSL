//! Conservative byte initialization map for generated repr(C) objects.
//! Unknown/conditional storage never authorizes a grouped load.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryKind {
    Data(usize),
    Metadata(usize),
    Padding,
    Descriptor(usize),
    Conditional(usize),
    Unknown(usize),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemorySpan {
    pub offset: Option<usize>,
    pub bytes: Option<usize>,
    pub kind: MemoryKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldLocation {
    pub offset: Option<usize>,
    pub bytes: Option<usize>,
    pub alignment: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryPlan {
    pub spans: Vec<MemorySpan>,
    pub fields: Vec<FieldLocation>,
    pub bytes: Option<usize>,
    pub alignment: usize,
}
fn data_type(t: &QslType) -> bool {
    match t {
        QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::DomainInteger(_)
        | QslType::Flags(_)
        | QslType::FixedString(_)
        | QslType::FixedCompressedString(_) => true,
        QslType::FixedArray(e, _) => data_type(e),
        _ => false,
    }
}
impl MemoryPlan {
    pub fn analyze(s: &QslSchema) -> QslResult<Self> {
        s.validate()?;
        let physical = PhysicalPlan::analyze(s)?;
        let mut out = Self {
            spans: Vec::new(),
            fields: vec![
                FieldLocation {
                    offset: None,
                    bytes: None,
                    alignment: None
                };
                s.fields.len()
            ],
            bytes: Some(0),
            alignment: 1,
        };
        let mut position = Some(0usize);
        for &i in &physical.order {
            let f = &s.fields[i];
            let layout = object_layout(f, s);
            let offset = layout.and_then(|l| align(position?, l.alignment));
            if let (Some(start), Some(at)) = (position, offset) {
                if at > start {
                    out.spans.push(MemorySpan {
                        offset: Some(start),
                        bytes: Some(at - start),
                        kind: MemoryKind::Padding,
                    });
                }
            }
            if let Some(l) = layout {
                out.alignment = out.alignment.max(l.alignment);
            }
            out.fields[i] = FieldLocation {
                offset,
                bytes: layout.map(|l| l.bytes),
                alignment: layout.map(|l| l.alignment),
            };
            let inline = matches!(
                f.storage,
                StorageKind::StackString(_) | StorageKind::StackArray(_)
            ) || matches!(
                f.data,
                QslType::InlineRawString { .. } | QslType::CompressedString { .. }
            ) && !matches!(
                f.storage,
                StorageKind::HeapString | StorageKind::HeapInlineString(_)
            );
            if f.optional {
                out.spans.push(MemorySpan {
                    offset,
                    bytes: layout.map(|l| l.bytes),
                    kind: MemoryKind::Conditional(i),
                });
            } else if inline {
                // Only the length word is unconditionally initialized; the tail is conditional.
                let word = crate::layout::length_storage_bytes(f);
                out.spans.push(MemorySpan {
                    offset,
                    bytes: Some(word),
                    kind: MemoryKind::Metadata(i),
                });
                out.spans.push(MemorySpan {
                    offset: offset.and_then(|p| p.checked_add(word)),
                    bytes: layout.and_then(|l| l.bytes.checked_sub(word)),
                    kind: MemoryKind::Conditional(i),
                });
            } else {
                let kind = if matches!(
                    f.storage,
                    StorageKind::StrView
                        | StorageKind::NativeView
                        | StorageKind::HeapString
                        | StorageKind::HeapInlineString(_)
                ) || matches!(
                    f.data,
                    QslType::String(_)
                        | QslType::RangedString { .. }
                        | QslType::BorrowedString { .. }
                        | QslType::Array(..)
                        | QslType::RangedArray { .. }
                ) || matches!(f.data, QslType::FixedArray(..))
                    && !matches!(f.storage, StorageKind::FixedArray(_))
                {
                    MemoryKind::Descriptor(i)
                } else if data_type(&f.data)
                    && (f.storage == StorageKind::Default
                        || matches!(f.storage, StorageKind::FixedArray(_)))
                {
                    MemoryKind::Data(i)
                } else {
                    MemoryKind::Unknown(i)
                };
                out.spans.push(MemorySpan {
                    offset,
                    bytes: layout.map(|l| l.bytes),
                    kind,
                });
            }
            position = offset
                .zip(layout)
                .and_then(|(at, l)| at.checked_add(l.bytes));
        }
        out.alignment = out.alignment.max(s.options.native_alignment.unwrap_or(1));
        out.bytes = position.and_then(|p| align(p, out.alignment));
        if let (Some(end), Some(total)) = (position, out.bytes) {
            if total > end {
                out.spans.push(MemorySpan {
                    offset: Some(end),
                    bytes: Some(total - end),
                    kind: MemoryKind::Padding,
                });
            }
        }
        Ok(out)
    }
    /// True only when the entire nonempty window consists of initialized plain data.
    pub fn can_load_data(&self, offset: usize, bytes: usize) -> bool {
        let Some(end) = offset.checked_add(bytes) else {
            return false;
        };
        if bytes == 0 {
            return true;
        }
        let mut at = offset;
        for span in &self.spans {
            let (Some(start), Some(n)) = (span.offset, span.bytes) else {
                continue;
            };
            let Some(stop) = start.checked_add(n) else {
                return false;
            };
            if stop <= at {
                continue;
            }
            if start > at {
                return false;
            }
            if !matches!(span.kind, MemoryKind::Data(_)) {
                return false;
            }
            at = end.min(stop);
            if at == end {
                return true;
            }
        }
        false
    }
    /// Contiguous initialized raw-wire fields. A native-copy caller must additionally
    /// enforce little-endian target layout and compile-time size/offset assertions.
    pub fn direct_group(&self, s: &QslSchema, fields: &[usize]) -> Option<(usize, usize)> {
        if s.compact() || s.qps() || s.options.dynamic.is_some() || fields.is_empty() {
            return None;
        }
        let first = self.fields.get(fields[0])?.offset?;
        let mut bytes = 0usize;
        for &i in fields {
            let f = s.fields.get(i)?;
            if f.optional {
                return None;
            }
            let loc = self.fields.get(i)?;
            let n = f.data.fixed_wire_size().ok()??;
            if loc.offset? != first.checked_add(bytes)? || loc.bytes? != n {
                return None;
            }
            bytes = bytes.checked_add(n)?;
        }
        (bytes != 0 && self.can_load_data(first, bytes)).then_some((first, bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn schema(types: Vec<QslType>) -> QslSchema {
        QslSchema {
            name: "P".into(),
            max_tag: types.len(),
            fields: types
                .into_iter()
                .enumerate()
                .map(|(i, data)| crate::QslField {
                    name: format!("f{i}"),
                    tag: i + 1,
                    bits: data.default_compact_bits().unwrap(),
                    data,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }
    #[test]
    fn exact_repr_c_map_never_authorizes_padding_or_descriptors() {
        #[repr(C)]
        struct Native {
            a: u8,
            b: u64,
            c: u8,
        }
        let s = schema(vec![
            QslType::Byte,
            QslType::Integer {
                bits: 64,
                signed: false,
                min: 0,
            },
            QslType::Byte,
        ]);
        let p = MemoryPlan::analyze(&s).unwrap();
        assert_eq!(p.bytes, Some(std::mem::size_of::<Native>()));
        assert_eq!(p.fields[1].offset, Some(std::mem::offset_of!(Native, b)));
        for start in 0..32usize {
            for n in 1..=17 {
                let expected = (start..start + n).all(|i| i == 0 || (8..=16).contains(&i));
                assert_eq!(p.can_load_data(start, n), expected, "{start}+{n}");
            }
        }
        assert!(p.direct_group(&s, &[0, 1]).is_none());
        assert_eq!(p.direct_group(&s, &[1, 2]), Some((8, 9)));
        assert!(!p.can_load_data(usize::MAX, 2));
        let s = schema(vec![QslType::String(0), QslType::Byte]);
        let p = MemoryPlan::analyze(&s).unwrap();
        assert!(!p.can_load_data(0, std::mem::size_of::<String>()));
        assert!(p.can_load_data(std::mem::size_of::<String>(), 1));
    }
    #[test]
    fn inline_capacity_and_absent_slots_are_not_unconditional_data() {
        let mut s = schema(vec![QslType::RangedString { min: 0, max: 9 }]);
        s.fields[0].storage = StorageKind::StackString(9);
        let p = MemoryPlan::analyze(&s).unwrap();
        assert_eq!(p.bytes, Some(16));
        for start in 0..16 {
            assert!(!p.can_load_data(start, 1));
        }
        assert_eq!(p.spans[0].kind, MemoryKind::Metadata(0));
        s.fields[0].optional = true;
        let p = MemoryPlan::analyze(&s).unwrap();
        assert_eq!(p.bytes, None);
        assert!(!p.can_load_data(0, 1));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopySegment {
    pub fields: Vec<usize>,
    pub native_offset: usize,
    pub wire_offset: usize,
    pub bytes: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointingPlan {
    pub memory: MemoryPlan,
    pub segments: Vec<CopySegment>,
    pub wire_bytes: usize,
    pub all_bits_valid: bool,
    pub scalar_copies: usize,
}
fn unrestricted(t: &QslType) -> bool {
    match t {
        QslType::Byte => true,
        QslType::Integer { min, .. } => *min == 0,
        QslType::RangedInteger {
            bits,
            signed,
            min,
            max,
        } => {
            let low = if *signed { -(1i128 << (*bits - 1)) } else { 0 };
            let high = if *signed {
                (1i128 << (*bits - 1)) - 1
            } else {
                (1i128 << *bits) - 1
            };
            *min == low && *max == high
        }
        QslType::DomainInteger(crate::domain::NumericDomain::Interval(d)) => {
            d.step() == 1 && d.maximum_rank() == crate::domain::low_mask(d.physical_bits())
        }
        QslType::Flags(n) => matches!(*n, 8 | 16 | 32 | 64),
        QslType::FixedArray(e, _) => unrestricted(e),
        _ => false,
    }
}
fn scalar_copies(t: &QslType) -> usize {
    match t {
        QslType::FixedArray(e, n) => scalar_copies(e).saturating_mul(*n),
        _ => 1,
    }
}
impl PointingPlan {
    pub fn analyze(s: &QslSchema) -> QslResult<Option<Self>> {
        let memory = MemoryPlan::analyze(s)?;
        Self::with_memory(s, &memory)
    }
    pub fn with_memory(s: &QslSchema, memory: &MemoryPlan) -> QslResult<Option<Self>> {
        if !s.options.aggressive_pointing
            || s.compact()
            || s.qps()
            || s.sequential()
            || s.state_bits()
            || s.optional_count() != 0
            || s.lifetime()
            || s.options.dynamic.is_some()
            || s.options.transmutate
            || s.options.native_overlay()
            || s.options.external_definition
        {
            return Ok(None);
        }
        if memory.bytes.is_none() {
            return Ok(None);
        }
        let mut segments = Vec::new();
        let mut at = 0;
        let mut wire = 0usize;
        while at < s.fields.len() {
            let mut end = at + 1;
            let Some(mut span) = memory.direct_group(s, &[at]) else {
                return Ok(None);
            };
            while end < s.fields.len() {
                let fields = (at..=end).collect::<Vec<_>>();
                let Some(next) = memory.direct_group(s, &fields) else {
                    break;
                };
                span = next;
                end += 1;
            }
            segments.push(CopySegment {
                fields: (at..end).collect(),
                native_offset: span.0,
                wire_offset: wire,
                bytes: span.1,
            });
            wire = wire.checked_add(span.1).ok_or(QslError::LengthOverflow)?;
            at = end;
        }
        let copies = s
            .fields
            .iter()
            .map(|f| scalar_copies(&f.data))
            .sum::<usize>();
        // The flag only admits the candidate: require an actual reduction in copy groups.
        if wire == 0 || segments.len() >= copies {
            return Ok(None);
        }
        Ok(Some(Self {
            memory: memory.clone(),
            segments,
            wire_bytes: wire,
            all_bits_valid: s.fields.iter().all(|f| unrestricted(&f.data)),
            scalar_copies: copies,
        }))
    }
}
