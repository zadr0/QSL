//! Build-time logical/wire mapping and region barriers. No runtime schema data.
use crate::{QslError, QslResult, QslSchema, QslType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionKind {
    Fixed,
    Dynamic,
    Conditional,
    Opaque,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WireRegion {
    /// Indices into the logical schema, listed in wire order.
    pub fields: Vec<usize>,
    pub kind: RegionKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WirePlan {
    pub version: Option<u16>,
    pub wire_to_logical: Vec<usize>,
    pub logical_to_wire: Vec<usize>,
    pub regions: Vec<WireRegion>,
}
fn plain(t: &QslType) -> bool {
    match t {
        QslType::Byte
        | QslType::Integer { .. }
        | QslType::RangedInteger { .. }
        | QslType::DomainInteger(_)
        | QslType::Flags(_)
        | QslType::FixedString(_)
        | QslType::BitPadding(_) => true,
        QslType::FixedArray(e, _) => plain(e),
        _ => false,
    }
}
fn kind(s: &QslSchema, index: usize) -> RegionKind {
    let f = &s.fields[index];
    if f.optional {
        RegionKind::Conditional
    } else if plain(&f.data) {
        RegionKind::Fixed
    } else if matches!(
        f.data,
        QslType::String(_)
            | QslType::RangedString { .. }
            | QslType::BorrowedString { .. }
            | QslType::InlineRawString { .. }
            | QslType::CompressedString { .. }
            | QslType::FixedCompressedString(_)
            | QslType::Array(..)
            | QslType::RangedArray { .. }
    ) {
        RegionKind::Dynamic
    } else {
        RegionKind::Opaque
    }
}
impl WirePlan {
    pub fn validate_policy(s: &QslSchema) -> QslResult<()> {
        if let Some(v) = s.options.auto_wire {
            if v != 1
                || s.options.dynamic.is_some()
                || s.qps()
                || s.state_bits()
                || s.legacy_opt_table()
                || s.options.transmutate
                || s.options.native_overlay()
                || s.options.external_definition
                || s.options.generated_region
                || !s.options.regions.is_empty()
            {
                return Err(QslError::InvalidSchema);
            }
        }
        Ok(())
    }
    pub fn analyze(s: &QslSchema) -> QslResult<Self> {
        s.validate()?;
        let mut order = (0..s.fields.len()).collect::<Vec<_>>();
        if s.options.auto_wire.is_some() {
            let mut start = 0;
            while start < order.len() {
                if matches!(
                    kind(s, order[start]),
                    RegionKind::Conditional | RegionKind::Opaque
                ) {
                    start += 1;
                    continue;
                }
                let mut end = start + 1;
                while end < order.len()
                    && !matches!(
                        kind(s, order[end]),
                        RegionKind::Conditional | RegionKind::Opaque
                    )
                {
                    end += 1;
                }
                // v1: independent fixed fields first, then dynamic fields; names are unique.
                // Tags are implicit declaration ordinals on required fields, not stable keys.
                order[start..end].sort_by(|&a, &b| {
                    let key = |i| {
                        (
                            usize::from(kind(s, i) == RegionKind::Dynamic),
                            s.fields[i].name.as_str(),
                        )
                    };
                    key(a).cmp(&key(b))
                });
                start = end;
            }
        }
        let mut inverse = vec![0; order.len()];
        let mut regions: Vec<WireRegion> = Vec::new();
        for (wire, &logical) in order.iter().enumerate() {
            inverse[logical] = wire;
            let k = kind(s, logical);
            if k == RegionKind::Fixed && regions.last().is_some_and(|r| r.kind == k) {
                regions.last_mut().unwrap().fields.push(logical);
            } else {
                regions.push(WireRegion {
                    fields: vec![logical],
                    kind: k,
                });
            }
        }
        Ok(Self {
            version: s.options.auto_wire,
            wire_to_logical: order,
            logical_to_wire: inverse,
            regions,
        })
    }
    pub fn lower(&self, s: &QslSchema) -> QslResult<QslSchema> {
        let n = s.fields.len();
        if self.version != s.options.auto_wire
            || self.wire_to_logical.len() != n
            || self.logical_to_wire.len() != n
        {
            return Err(QslError::InvalidSchema);
        }
        let mut seen = vec![false; n];
        for (wire, &logical) in self.wire_to_logical.iter().enumerate() {
            if logical >= n || seen[logical] || self.logical_to_wire[logical] != wire {
                return Err(QslError::InvalidSchema);
            }
            seen[logical] = true;
        }
        let mut out = s.clone();
        if self.version.is_some() && out.options.declaration_order.is_empty() {
            out.options.declaration_order = s.fields.iter().map(|f| f.name.clone()).collect();
        }
        out.fields = self
            .wire_to_logical
            .iter()
            .map(|&i| s.fields[i].clone())
            .collect();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_plan_cannot_drop_duplicate_or_access_missing_fields() {
        let s = QslSchema {
            name: "P".into(),
            max_tag: 2,
            fields: (1..=2)
                .map(|tag| crate::QslField {
                    name: format!("f{tag}"),
                    tag,
                    bits: 8,
                    data: QslType::Byte,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let original = WirePlan::analyze(&s).unwrap();
        for order in [vec![], vec![0, 0], vec![1, 2], vec![1, 0]] {
            let mut p = original.clone();
            p.wire_to_logical = order;
            assert!(p.lower(&s).is_err());
        }
        assert!(original.lower(&s).is_ok());
    }
}
