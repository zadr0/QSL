//! A graph of critical payload lengths. Nodes describe forced/possible field
//! sets, not a guess at an ambiguous boundary. Metadata remains outside I.
use crate::{QslError, QslResult, QslSchema, QslType};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct Domain {
    pub min: usize,
    pub max: usize,
    pub variable: bool,
    pub slot: Option<(usize, usize)>,
}
#[derive(Clone, Debug)]
pub struct LengthNode {
    pub start: usize,
    pub end: usize,
    pub forced: u64,
    pub possible: u64,
    pub valid: bool,
}
/// Static field-layout recipe; byte positions never live inside a record.
#[derive(Clone, Copy, Debug)]
pub struct OffsetRecipe {
    pub prior_fields: u64,
    pub min: usize,
    pub max: usize,
    pub length_slot: Option<(usize, usize)>,
}
#[derive(Clone, Debug)]
pub struct LengthGraph {
    pub fields: Vec<Domain>,
    pub nodes: Vec<LengthNode>,
    pub tag_bytes: usize,
    pub metadata_bytes: usize,
    pub max_payload: usize,
}
impl LengthGraph {
    pub fn analyze(s: &QslSchema) -> QslResult<Option<Self>> {
        let mut fields = Vec::new();
        for f in s.fields.iter().filter(|f| f.optional) {
            let data = match f.data {
                QslType::RangedString { min, max } | QslType::InlineRawString { min, max } => {
                    Some((min, max, true))
                }
                QslType::BorrowedString {
                    min,
                    max: Some(max),
                } => Some((min, max, true)),
                QslType::CompressedString { .. }
                | QslType::FixedCompressedString(_)
                | QslType::String(_) => None,
                _ => f.data.fixed_wire_size()?.map(|n| (n, n, false)),
            };
            let Some((min, max, variable)) = data else {
                return Ok(None);
            };
            if min > max {
                return Err(QslError::InvalidSchema);
            }
            fields.push(Domain {
                min,
                max,
                variable,
                slot: None,
            });
        }
        if fields.is_empty() || fields.len() > 64 {
            return Ok(None);
        }
        let max_payload = fields
            .iter()
            .try_fold(0usize, |n, f| n.checked_add(f.max))
            .ok_or(QslError::LengthOverflow)?;
        if max_payload > crate::static_index::ANALYSIS_MAX_BYTES {
            return Ok(None);
        }
        if fields.len() > 12 {
            let width = fields[0].min;
            if fields
                .iter()
                .any(|f| f.variable || f.min != width || f.max != width)
            {
                return Ok(None);
            }
            let all = if fields.len() == 64 {
                u64::MAX
            } else {
                (1u64 << fields.len()) - 1
            };
            let mut nodes = Vec::new();
            let mut next = 0;
            if width == 0 {
                nodes.push(LengthNode {
                    start: 0,
                    end: 0,
                    forced: 0,
                    possible: all,
                    valid: true,
                });
            } else {
                for count in 0..=fields.len() {
                    let at = count * width;
                    if next < at {
                        nodes.push(LengthNode {
                            start: next,
                            end: at - 1,
                            forced: 0,
                            possible: 0,
                            valid: false,
                        });
                    }
                    push_node(
                        &mut nodes,
                        LengthNode {
                            start: at,
                            end: at,
                            forced: if count == fields.len() { all } else { 0 },
                            possible: if count == 0 { 0 } else { all },
                            valid: true,
                        },
                    );
                    next = at + 1;
                }
            }
            return Ok(Some(Self::finish(fields, nodes, max_payload)));
        }
        let mut events = BTreeMap::<usize, Vec<(u64, bool)>>::new();
        for mask in 0..1u64 << fields.len() {
            let (mut min, mut max) = (0, 0);
            for (i, f) in fields.iter().enumerate() {
                if mask >> i & 1 != 0 {
                    min += f.min;
                    max += f.max;
                }
            }
            events.entry(min).or_default().push((mask, true));
            events.entry(max + 1).or_default().push((mask, false));
        }
        events.entry(0).or_default();
        events.entry(max_payload + 1).or_default();
        let events = events.into_iter().collect::<Vec<_>>();
        let mut active = 0usize;
        let mut counts = [0usize; 12];
        let mut nodes: Vec<LengthNode> = Vec::new();
        for pair in events.windows(2) {
            let (start, changes) = &pair[0];
            for &(mask, add) in changes {
                if add {
                    active += 1
                } else {
                    active -= 1
                };
                for (i, count) in counts.iter_mut().enumerate().take(fields.len()) {
                    if mask >> i & 1 != 0 {
                        if add {
                            *count += 1
                        } else {
                            *count -= 1
                        }
                    }
                }
            }
            let end = pair[1].0 - 1;
            let (mut forced, mut possible) = (0, 0);
            if active != 0 {
                for (i, &count) in counts.iter().enumerate().take(fields.len()) {
                    forced |= ((count == active) as u64) << i;
                    possible |= ((count != 0) as u64) << i;
                }
            }
            let next = LengthNode {
                start: *start,
                end,
                forced,
                possible,
                valid: active != 0,
            };
            push_node(&mut nodes, next);
        }
        Ok(Some(Self::finish(fields, nodes, max_payload)))
    }
    fn finish(mut fields: Vec<Domain>, nodes: Vec<LengthNode>, max_payload: usize) -> Self {
        let tag_bytes = if nodes.iter().any(|n| n.valid && n.forced != n.possible) {
            (fields.len() + 7) >> 3
        } else {
            0
        };
        let mut metadata_bytes = tag_bytes;
        let last = fields.iter().rposition(|f| f.variable);
        for (i, f) in fields.iter_mut().enumerate() {
            if f.variable && Some(i) != last {
                let width = if f.max <= u8::MAX as usize {
                    1
                } else if f.max <= u16::MAX as usize {
                    2
                } else {
                    4
                };
                f.slot = Some((metadata_bytes, width));
                metadata_bytes += width;
            }
        }
        Self {
            fields,
            nodes,
            tag_bytes,
            metadata_bytes,
            max_payload,
        }
    }
    /// Build/test oracle. Generated I/O emits numeric comparison operands.
    pub fn event_index(&self, len: usize) -> Option<usize> {
        (len <= self.max_payload).then(|| self.nodes.partition_point(|n| n.end < len))
    }
}
fn push_node(nodes: &mut Vec<LengthNode>, next: LengthNode) {
    if let Some(last) = nodes.last_mut().filter(|last| {
        last.end + 1 == next.start
            && last.forced == next.forced
            && last.possible == next.possible
            && last.valid == next.valid
    }) {
        last.end = next.end;
    } else {
        nodes.push(next);
    }
}
