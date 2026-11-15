//! Shared immutable build-time plan. No IR objects are carried into message I/O.
use super::*;
use crate::{domain::NumericDomain, layout::StorageKind, optimizer::MemoryPlan};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitExtent {
    Fixed(usize),
    RuntimeLength,
    Presence(usize),
    Opaque,
}
#[derive(Clone, Debug)]
pub struct FieldPlan {
    pub logical: usize,
    pub wire: usize,
    pub name: String,
    pub physical_offset: Option<usize>,
    pub physical_bits: Option<usize>,
    pub encoded_bits: Option<usize>,
    pub domain: Option<NumericDomain>,
    pub signed: bool,
    pub storage: StorageKind,
    pub capacity: Option<usize>,
    pub initialized: InitExtent,
    pub optional_bit: Option<usize>,
    pub wire_version: Option<u16>,
    pub qsl_lz: bool,
    pub shifted_length: bool,
    pub direct_copy: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Load {
        field: usize,
        register: BitRegister,
    },
    Normalize {
        field: usize,
    },
    Pack {
        field: usize,
        bits: usize,
    },
    Store {
        field: usize,
    },
    Codec {
        field: usize,
    },
    Barrier {
        field: usize,
        kind: RegionKind,
    },
    Copy {
        native: usize,
        wire: usize,
        bytes: usize,
    },
}
#[derive(Clone, Debug)]
pub struct CompilationPlan {
    pub logical: QslSchema,
    pub schema: QslSchema,
    pub wire: WirePlan,
    pub memory: MemoryPlan,
    pub io: QslIoPlan,
    pub fields: Vec<FieldPlan>,
    pub operations: Vec<Operation>,
    pub registers: crate::optimizer::registers::RegisterPlan,
}
impl CompilationPlan {
    pub fn analyze(logical: &QslSchema) -> QslResult<Self> {
        Self::analyze_with_profile(logical, crate::optimizer::registers::CpuProfile::Portable)
    }
    pub fn analyze_with_profile(
        logical: &QslSchema,
        profile: crate::optimizer::registers::CpuProfile,
    ) -> QslResult<Self> {
        let wire = WirePlan::analyze(logical)?;
        let schema = wire.lower(logical)?;
        let memory = MemoryPlan::analyze(&schema)?;
        let mut io = QslIoPlan::with_memory(&schema, &memory)?;
        let registers =
            crate::optimizer::registers::RegisterPlan::analyze(&schema, &memory, profile)?;
        io.fixed_bmi2_write = registers.groups.len() == 1
            && schema.fields.len() == 4
            && registers.groups[0].selected_write
                == crate::optimizer::registers::PackingStrategy::PextPdep;
        let mut fields = Vec::with_capacity(schema.fields.len());
        let mut operations = Vec::new();
        for (i, f) in schema.fields.iter().enumerate() {
            let domain = match &f.data {
                QslType::DomainInteger(d) => Some(d.clone()),
                _ => None,
            };
            let k = wire
                .regions
                .iter()
                .find(|r| r.fields.contains(&wire.wire_to_logical[i]))
                .map(|r| r.kind)
                .ok_or(QslError::InvalidSchema)?;
            let loc = &memory.fields[i];
            let capacity = match f.storage {
                StorageKind::StackString(n)
                | StorageKind::StackArray(n)
                | StorageKind::FixedArray(n) => Some(n),
                _ => match f.data {
                    QslType::InlineRawString { max, .. }
                    | QslType::CompressedString { max, .. } => Some(max),
                    _ => None,
                },
            };
            let signed = domain.as_ref().is_some_and(NumericDomain::signedness)
                || matches!(
                    f.data,
                    QslType::Integer { signed: true, .. }
                        | QslType::RangedInteger { signed: true, .. }
                );
            let bits = if k == RegionKind::Fixed {
                Some(if schema.compact() {
                    f.data.rank_width(f.bits)?
                } else {
                    f.data
                        .fixed_wire_size()?
                        .ok_or(QslError::InvalidSchema)?
                        .checked_mul(8)
                        .ok_or(QslError::InvalidWidth)?
                })
            } else {
                None
            };
            let initialized = if f.optional {
                InitExtent::Presence(f.tag - 1)
            } else if k == RegionKind::Dynamic {
                InitExtent::RuntimeLength
            } else if let Some(bytes) = loc
                .bytes
                .filter(|&n| memory.can_load_data(loc.offset.unwrap_or(usize::MAX), n))
            {
                InitExtent::Fixed(bytes)
            } else {
                InitExtent::Opaque
            };
            let direct_copy = io
                .pointing
                .as_ref()
                .is_some_and(|p| p.segments.iter().any(|s| s.fields.contains(&i)));
            fields.push(FieldPlan {
                logical: wire.wire_to_logical[i],
                wire: i,
                name: f.name.clone(),
                physical_offset: loc.offset,
                physical_bits: loc.bytes.and_then(|n| n.checked_mul(8)),
                encoded_bits: bits,
                domain,
                signed,
                storage: f.storage,
                capacity,
                initialized,
                optional_bit: f.optional.then_some(f.tag.saturating_sub(1)),
                wire_version: wire.version,
                shifted_length: f.shifted_length,
                qsl_lz: matches!(
                    f.data,
                    QslType::CompressedString { .. } | QslType::FixedCompressedString(_)
                ),
                direct_copy,
            });
            if k != RegionKind::Fixed {
                operations.push(Operation::Barrier { field: i, kind: k });
            }
            if direct_copy {
                continue;
            }
            if let Some(bits) = bits.filter(|_| {
                matches!(
                    f.data,
                    QslType::Byte
                        | QslType::Integer { .. }
                        | QslType::RangedInteger { .. }
                        | QslType::DomainInteger(_)
                        | QslType::Flags(_)
                )
            }) {
                let register = if schema.compact() {
                    io.blocks[io.field_blocks[i]].reader
                } else {
                    BitRegister::for_bits(bits)?
                };
                operations.push(Operation::Load { field: i, register });
                if schema.compact() {
                    operations.push(Operation::Normalize { field: i });
                    operations.push(Operation::Pack { field: i, bits });
                }
                operations.push(Operation::Store { field: i });
            } else {
                operations.push(Operation::Codec { field: i });
            }
        }
        if let Some(p) = &io.pointing {
            for c in &p.segments {
                operations.push(Operation::Copy {
                    native: c.native_offset,
                    wire: c.wire_offset,
                    bytes: c.bytes,
                });
            }
        }
        Ok(Self {
            logical: logical.clone(),
            schema,
            wire,
            memory,
            io,
            fields,
            operations,
            registers,
        })
    }
}
