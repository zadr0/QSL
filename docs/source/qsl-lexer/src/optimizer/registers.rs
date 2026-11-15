//! Deterministic build-time register candidate diagnostics.
//! Costs are relative scheduling estimates, not measurements or clock cycles.
//! An unmeasured ISA candidate is never promoted over the existing scalar codec.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuProfile {
    Portable,
    KabyLake,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackingStrategy {
    Scalar,
    GprPair,
    PextPdep,
    ByteShuffle,
    Packus,
    Madd,
    Avx2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cost {
    pub uops: u16,
    pub dependency_depth: u16,
    pub gpr: u8,
    pub vector: u8,
    pub transfers: u8,
    pub loads: u8,
    pub stores: u8,
    pub code_bytes: u16,
}
impl Cost {
    /// Fixed integer weights keep the report reproducible across machines.
    pub fn score(self, profile: CpuProfile) -> u32 {
        let transfer = match profile {
            CpuProfile::Portable => 4,
            CpuProfile::KabyLake => 3,
        };
        8 * u32::from(self.dependency_depth)
            + 2 * u32::from(self.uops)
            + transfer * u32::from(self.transfers)
            + u32::from(self.gpr.saturating_sub(8)) * 4
            + u32::from(self.vector.saturating_sub(8)) * 4
            + u32::from(self.code_bytes) / 16
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub strategy: PackingStrategy,
    pub feature: Option<&'static str>,
    pub feasible: bool,
    pub reason: &'static str,
    pub cost: Cost,
    pub measured: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterGroupPlan {
    pub fields: Vec<usize>,
    pub native_offset: Option<usize>,
    pub physical_bits: usize,
    pub useful_bits: usize,
    pub widths: Vec<usize>,
    pub candidates: Vec<Candidate>,
    pub selected_write: PackingStrategy,
    pub selected_read: PackingStrategy,
    pub selection_reason: &'static str,
    pub cheapest_estimate: PackingStrategy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterPlan {
    pub profile: CpuProfile,
    pub groups: Vec<RegisterGroupPlan>,
}
fn numeric(t: &QslType) -> Option<(usize, bool)> {
    match t {
        QslType::RangedInteger {
            bits: 32,
            signed: false,
            min,
            max,
        } => Some((QslType::rank_bits(*min, *max).ok()?, *min == 0)),
        QslType::Integer {
            bits: 32,
            signed: false,
            min,
        } => Some((32, *min == 0)),
        QslType::DomainInteger(crate::domain::NumericDomain::Interval(d))
            if d.physical_bits() == 32 && !d.signedness() =>
        {
            Some((
                d.encoded_bits() as usize,
                d.minimum_bits() == 0 && d.step() == 1,
            ))
        }
        _ => None,
    }
}
impl RegisterPlan {
    pub fn analyze(s: &QslSchema, memory: &MemoryPlan, profile: CpuProfile) -> QslResult<Self> {
        let mut groups = Vec::new();
        if !s.compact()
            || s.flags & crate::FLAG_ALIGNMENT != 0
            || s.optional_count() != 0
            || s.options.dynamic.is_some()
        {
            return Ok(Self { profile, groups });
        }
        let mut i = 0;
        while i < s.fields.len() {
            let mut end = i;
            while end < s.fields.len() && numeric(&s.fields[end].data).is_some() {
                end += 1;
            }
            while end - i >= 4 {
                let count = if end - i >= 8 { 8 } else { 4 };
                let ids = (i..i + count).collect::<Vec<_>>();
                let facts = ids
                    .iter()
                    .map(|&j| numeric(&s.fields[j].data).unwrap())
                    .collect::<Vec<_>>();
                let widths = facts.iter().map(|v| v.0).collect::<Vec<_>>();
                let useful_bits: usize = widths.iter().sum();
                let at = memory.fields[i].offset;
                let contiguous = at.is_some_and(|at| {
                    ids.iter().enumerate().all(|(n, &j)| {
                        memory.fields[j].offset == Some(at + n * 4)
                            && memory.fields[j].bytes == Some(4)
                    }) && memory.can_load_data(at, count * 4)
                });
                let normalized = facts.iter().all(|v| v.1);
                let narrow = widths.iter().all(|&w| w == 16);
                let twelve = widths.iter().all(|&w| w == 12);
                let scalar = Cost {
                    uops: (count * 4) as u16,
                    dependency_depth: 4,
                    gpr: count as u8 + 2,
                    vector: 0,
                    transfers: 0,
                    loads: count as u8,
                    stores: useful_bits.div_ceil(64) as u8,
                    code_bytes: (count * 14) as u16,
                };
                let candidate = |strategy, feature, shape: bool, cost| Candidate {
                    strategy,
                    feature,
                    feasible: contiguous && normalized && shape,
                    reason: if !contiguous {
                        "physical group crosses padding, descriptors or reordered fields"
                    } else if !normalized {
                        "normalization is required before a grouped native load"
                    } else if !shape {
                        "rank widths are outside this kernel's exact shape"
                    } else {
                        "reference tests only; end-to-end benchmark required before selection"
                    },
                    cost,
                    measured: false,
                };
                let mut candidates = vec![Candidate {
                    strategy: PackingStrategy::Scalar,
                    feature: None,
                    feasible: true,
                    reason: "existing canonical scalar implementation",
                    cost: scalar,
                    measured: profile == CpuProfile::KabyLake && count == 4 && twelve,
                }];
                candidates.push(candidate(
                    PackingStrategy::GprPair,
                    None,
                    count == 4,
                    Cost {
                        uops: 14,
                        dependency_depth: 6,
                        gpr: 7,
                        vector: 0,
                        transfers: 0,
                        loads: 2,
                        stores: 2,
                        code_bytes: 66,
                    },
                ));
                candidates.push(candidate(
                    PackingStrategy::PextPdep,
                    Some("bmi2"),
                    count == 4,
                    Cost {
                        uops: 10,
                        dependency_depth: 5,
                        gpr: 7,
                        vector: 0,
                        transfers: 0,
                        loads: 2,
                        stores: 2,
                        code_bytes: 68,
                    },
                ));
                candidates.push(candidate(
                    PackingStrategy::ByteShuffle,
                    Some("ssse3"),
                    count == 4 && narrow,
                    Cost {
                        uops: 4,
                        dependency_depth: 3,
                        gpr: 2,
                        vector: 2,
                        transfers: 1,
                        loads: 2,
                        stores: 1,
                        code_bytes: 32,
                    },
                ));
                candidates.push(candidate(
                    PackingStrategy::Packus,
                    Some("sse4.1"),
                    count == 4 && narrow,
                    Cost {
                        uops: 4,
                        dependency_depth: 3,
                        gpr: 2,
                        vector: 2,
                        transfers: 1,
                        loads: 1,
                        stores: 1,
                        code_bytes: 24,
                    },
                ));
                candidates.push(candidate(
                    PackingStrategy::Madd,
                    Some("sse4.1"),
                    count == 4 && twelve,
                    Cost {
                        uops: 9,
                        dependency_depth: 6,
                        gpr: 3,
                        vector: 3,
                        transfers: 1,
                        loads: 2,
                        stores: 1,
                        code_bytes: 46,
                    },
                ));
                candidates.push(candidate(
                    PackingStrategy::Avx2,
                    Some("avx2"),
                    count == 8 && narrow,
                    Cost {
                        uops: 6,
                        dependency_depth: 5,
                        gpr: 2,
                        vector: 3,
                        transfers: 0,
                        loads: 1,
                        stores: 1,
                        code_bytes: 38,
                    },
                ));
                // Portable builds cannot authorize target-specific instructions. Kaby Lake
                // estimates expose them for experiments, while preserving the scalar default.
                let cheapest_estimate = candidates
                    .iter()
                    .filter(|c| {
                        c.feasible && (profile != CpuProfile::Portable || c.feature.is_none())
                    })
                    .min_by_key(|c| c.cost.score(profile))
                    .unwrap()
                    .strategy;
                let accepted = profile == CpuProfile::KabyLake
                    && s.options.aggressive_group
                    && count == 4
                    && s.fields.len() == 4
                    && twelve
                    && contiguous
                    && normalized;
                if accepted {
                    let c = candidates
                        .iter_mut()
                        .find(|c| c.strategy == PackingStrategy::PextPdep)
                        .unwrap();
                    c.measured = true;
                    c.reason="generated 4x12-bit record, i5-7400: paired forward/reverse runs accept PEXT writes, reject PDEP reads";
                }
                let selected_write = if accepted {
                    PackingStrategy::PextPdep
                } else {
                    PackingStrategy::Scalar
                };
                groups.push(RegisterGroupPlan {
                    fields: ids,
                    native_offset: at,
                    physical_bits: count * 32,
                    useful_bits,
                    widths,
                    candidates,
                    selected_write,
                    // Generated-profile regression on 2026-11-14: PDEP read was
                    // slower in both orders. Selection is directional, not symmetric.
                    selected_read: PackingStrategy::Scalar,
                    selection_reason: if accepted {
                        "PEXT write accepted; PDEP read rejected by generated-profile Criterion regression"
                    } else {
                        "scalar retained: no accepted generated-profile measurement for this shape"
                    },
                    cheapest_estimate,
                });
                i += count;
            }
            i = end.max(i + 1);
        }
        Ok(Self { profile, groups })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn schema() -> QslSchema {
        QslSchema {
            name: "Pack".into(),
            max_tag: 4,
            fields: (0..4)
                .map(|i| QslField {
                    tag: i + 1,
                    name: format!("f{i}"),
                    data: QslType::RangedInteger {
                        bits: 32,
                        signed: false,
                        min: 0,
                        max: 65535,
                    },
                    bits: 16,
                    ..Default::default()
                })
                .collect(),
            flags: crate::FLAG_COMPACT,
            ..Default::default()
        }
    }
    #[test]
    fn measured_profile_selects_write_and_read_independently() {
        let mut s = schema();
        s.options.aggressive_group = true;
        for f in &mut s.fields {
            f.data = QslType::RangedInteger {
                bits: 32,
                signed: false,
                min: 0,
                max: 4095,
            };
            f.bits = 12;
        }
        let memory = MemoryPlan::analyze(&s).unwrap();
        let p = RegisterPlan::analyze(&s, &memory, CpuProfile::KabyLake).unwrap();
        assert_eq!(p.groups[0].selected_write, PackingStrategy::PextPdep);
        assert_eq!(p.groups[0].selected_read, PackingStrategy::Scalar);
        assert!(p.groups[0].selection_reason.contains("read rejected"));
        let p = RegisterPlan::analyze(&s, &memory, CpuProfile::Portable).unwrap();
        assert_eq!(p.groups[0].selected_write, PackingStrategy::Scalar);
        assert_eq!(p.groups[0].selected_read, PackingStrategy::Scalar);
    }
    #[test]
    fn candidates_require_initialized_contiguity_and_never_select_unmeasured_isa() {
        let s = schema();
        let m = MemoryPlan::analyze(&s).unwrap();
        let a = RegisterPlan::analyze(&s, &m, CpuProfile::KabyLake).unwrap();
        assert_eq!(a.groups.len(), 1);
        assert_eq!(a.groups[0].useful_bits, 64);
        assert_eq!(a.groups[0].selected_write, PackingStrategy::Scalar);
        assert!(a.groups[0]
            .candidates
            .iter()
            .any(|c| c.strategy == PackingStrategy::Packus && c.feasible));
        assert_eq!(
            a,
            RegisterPlan::analyze(&s, &m, CpuProfile::KabyLake).unwrap()
        );
        let mut broken = m;
        broken.fields[1].offset = Some(8);
        let p = RegisterPlan::analyze(&s, &broken, CpuProfile::KabyLake).unwrap();
        assert!(p.groups[0]
            .candidates
            .iter()
            .filter(|c| c.strategy != PackingStrategy::Scalar)
            .all(|c| !c.feasible));
    }
}
