//! QPS compresses a deterministic required scalar prefix only. Strings, array
//! elements and all length descriptors keep their ordinary independent codec.
use crate::{QslResult, QslSchema, QslType};
pub struct HybridPlan {
    pub prefix: QslSchema,
    pub tail: QslSchema,
    pub prefix_bytes: usize,
    pub raw_prefix_bytes: usize,
    pub prefix_minimum: usize,
    pub length_header: usize,
}
impl HybridPlan {
    pub fn analyze(s: &QslSchema) -> QslResult<Option<Self>> {
        if !s.qps()
            || s.fields.iter().all(|f| {
                matches!(
                    f.data,
                    QslType::Byte
                        | QslType::Integer { .. }
                        | QslType::RangedInteger { .. }
                        | QslType::Flags(_)
                )
            })
        {
            return Ok(None);
        }
        let count = s
            .fields
            .iter()
            .take_while(|f| {
                matches!(
                    f.data,
                    QslType::Byte
                        | QslType::Integer { .. }
                        | QslType::RangedInteger { .. }
                        | QslType::Flags(_)
                )
            })
            .count();
        if count == 0 || count == s.fields.len() {
            return Ok(None);
        }
        let mut prefix = s.clone();
        prefix.name = format!("__QslPrefix_{}", s.name);
        prefix.fields.truncate(count);
        prefix.max_tag = count;
        prefix.flags = crate::FLAG_QPS
            | (s.flags
                & (crate::PAYLOAD_QUANT_MASK
                    | crate::FLAG_SAFETY
                    | crate::FLAG_INLINE
                    | crate::FLAG_INLINE_ALWAYS
                    | crate::FLAG_INLINE_NEVER));
        prefix.options = Default::default();
        let quant = crate::payload_quant::PayloadQuantPlan::analyze(&prefix)?;
        if !quant.active {
            return Ok(None);
        }
        let raw_prefix_bytes = prefix
            .fields
            .iter()
            .try_fold(0usize, |n, f| {
                n.checked_add(f.data.fixed_wire_size().ok().flatten()?)
            })
            .ok_or(crate::QslError::LengthOverflow)?;
        let mut tail = s.clone();
        tail.name = format!("__QslTail_{}", s.name);
        tail.fields.drain(..count);
        tail.flags &= !(crate::FLAG_QPS | crate::PAYLOAD_QUANT_MASK);
        tail.options.loyal_cache = false;
        tail.options.cache_limit = None;
        tail.options.regions.clear();
        let length_header = if quant.min_bytes == quant.max_bytes {
            0
        } else if quant.max_bytes <= 255 {
            1
        } else {
            2
        };
        Ok(Some(Self {
            prefix,
            tail,
            prefix_bytes: quant.max_bytes,
            raw_prefix_bytes,
            prefix_minimum: quant.min_bytes,
            length_header,
        }))
    }
}
