//! Build-time static-index analysis and legacy state_bits compatibility.
//! Q_table compresses only the index
//! table; no payload length or numerical value is rounded.
use crate::{QslError, QslResult, QslSchema, QslType};

pub const ANALYSIS_MAX_STATES: usize = 4096;
pub const ANALYSIS_MAX_BYTES: usize = 65535;

#[derive(Debug, Clone)]
pub struct StateCode {
    pub min: usize,
    pub max: usize,
    pub shift: u32,
    pub bits: u32,
    pub fixed: bool,
}
#[derive(Debug, Clone)]
pub struct OffsetRow {
    pub len: u32,
    pub mask: u32,
    pub offsets: Vec<u32>,
}
#[derive(Debug, Clone)]
pub struct PredictPlan {
    pub fields: Vec<StateCode>,
    pub used_bits: u32,
    pub rows: Vec<OffsetRow>,
    pub index: Vec<u32>,
    pub quantum: usize,
    pub reason: Option<&'static str>,
}

impl PredictPlan {
    pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
        if schema.optional_count() > 64 {
            return Err(QslError::InvalidSchema);
        }
        let mut used_bits = 0;
        let mut fields = Vec::new();
        for f in schema.fields.iter().filter(|f| f.optional) {
            let (min, max, fixed) = match &f.data {
                QslType::String(_) => return Err(QslError::InvalidSchema),
                QslType::RangedString { min, max }
                | QslType::InlineRawString { min, max }
                | QslType::BinaryString {
                    min,
                    max: Some(max),
                }
                | QslType::GuardString {
                    min,
                    max: Some(max),
                }
                | QslType::CopyGuardString {
                    min,
                    max: Some(max),
                } => (*min, *max, false),
                // Compression has a data-dependent encoded size. It is not a
                // raw UTF-8 field and must retain the legacy explicit codec.
                QslType::CompressedString { .. } | QslType::FixedCompressedString(_) => {
                    return Err(QslError::InvalidSchema)
                }
                other => {
                    let n = other.fixed_wire_size()?.ok_or(QslError::InvalidSchema)?;
                    (n, n, true)
                }
            };
            let delta = if fixed {
                1
            } else {
                max.checked_sub(min)
                    .and_then(|n| n.checked_add(1))
                    .ok_or(QslError::LengthOverflow)?
            };
            let bits = usize::BITS - delta.leading_zeros();
            fields.push(StateCode {
                min,
                max,
                shift: used_bits,
                bits,
                fixed,
            });
            used_bits += bits;
        }
        if used_bits > 32 && schema.state_bits() {
            return Err(QslError::InvalidWidth);
        }
        let mut result = Self {
            fields,
            used_bits,
            rows: Vec::new(),
            index: Vec::new(),
            quantum: schema.state_bits_quantum(),
            reason: None,
        };
        let total = result
            .fields
            .iter()
            .try_fold(0usize, |s, f| s.checked_add(f.max))
            .ok_or(QslError::LengthOverflow)?;
        if result.fields.iter().any(|f| !f.fixed) {
            result.reason = Some("variable lengths: direct D extraction; no length-only lookup");
        } else if result.fields.len() >= usize::BITS as usize
            || result.fields.len() > 12
            || total > ANALYSIS_MAX_BYTES
        {
            result.reason = Some("static table analysis budget exceeded; direct D extraction");
        } else {
            let mut by_len = vec![None; total + 1];
            for mask in 0..(1usize << result.fields.len()) {
                let mut len = 0u32;
                let mut offsets = Vec::new();
                for (i, f) in result.fields.iter().enumerate() {
                    offsets.push(len);
                    if mask & (1 << i) != 0 {
                        len += f.min as u32;
                    }
                }
                if by_len[len as usize].is_some() {
                    result.reason = Some("ambiguous subset sums: direct D is required");
                    break;
                }
                by_len[len as usize] = Some(OffsetRow {
                    len,
                    mask: mask as u32,
                    offsets,
                });
            }
            if result.reason.is_none() {
                let mut rank = 0;
                for row in by_len {
                    if let Some(row) = row {
                        rank = result.rows.len() as u32;
                        result.rows.push(row);
                    }
                    result.index.push(rank);
                }
            }
        }
        if schema.predict_table() && result.reason.is_some() {
            return Err(QslError::InvalidSchema);
        }
        // A selected Q is never silently ignored. Only unit-transition length
        // indices admit this exact base+popcount representation.
        if result.quantum > 1 && result.reason.is_some() {
            return Err(QslError::InvalidSchema);
        }
        Ok(result)
    }
    pub fn packed_index(&self) -> QslResult<Vec<u32>> {
        quantize_index(&self.index, self.quantum)
    }
    /// Build/test oracle only. Generated runtime I/O uses unrolled expressions.
    pub fn pack_lengths(&self, lengths: &[Option<usize>]) -> QslResult<u32> {
        if lengths.len() != self.fields.len() || self.used_bits > 32 {
            return Err(QslError::InvalidWidth);
        }
        let mut d = 0;
        for (length, f) in lengths.iter().zip(&self.fields) {
            if let Some(n) = length {
                if *n < f.min || *n > f.max {
                    return Err(QslError::InvalidValue);
                }
                let code = if f.fixed { 1 } else { *n - f.min + 1 };
                d |= (code as u32) << f.shift;
            }
        }
        Ok(d)
    }
    pub fn unpack_lengths(&self, d: u32) -> QslResult<Vec<Option<usize>>> {
        if self.used_bits > 32 || (self.used_bits < 32 && d >> self.used_bits != 0) {
            return Err(QslError::InvalidValue);
        }
        self.fields
            .iter()
            .map(|f| {
                let mask = if f.bits == 32 {
                    u32::MAX
                } else {
                    (1u32 << f.bits) - 1
                };
                let code = (d >> f.shift) & mask;
                if code == 0 {
                    return Ok(None);
                }
                let n = if f.fixed {
                    f.min
                } else {
                    f.min + code as usize - 1
                };
                if n > f.max || f.fixed && code != 1 {
                    return Err(QslError::InvalidValue);
                }
                Ok(Some(n))
            })
            .collect()
    }
}

/// Exact base plus unit finite-difference bitmap, as in spec §7.3.
/// Non-unit and negative differences are rejected, never counted as one.
pub fn quantize_index(index: &[u32], quantum: usize) -> QslResult<Vec<u32>> {
    if !matches!(quantum, 1 | 2 | 4 | 8 | 16) {
        return Err(QslError::InvalidValue);
    }
    if quantum == 1 {
        return Ok(index.to_vec());
    }
    let bits = quantum - 1;
    index
        .chunks(quantum)
        .map(|chunk| {
            let base = chunk[0];
            if base > (u32::MAX >> bits) {
                return Err(QslError::InvalidWidth);
            }
            let mut mask = 0u32;
            for i in 1..chunk.len() {
                match chunk[i].checked_sub(chunk[i - 1]) {
                    Some(0) => {}
                    Some(1) => mask |= 1 << (i - 1),
                    _ => return Err(QslError::InvalidValue),
                }
            }
            Ok((base << bits) | mask)
        })
        .collect()
}
#[inline]
pub fn lookup_index(table: &[u32], quantum: usize, index: usize) -> Option<u32> {
    if !matches!(quantum, 1 | 2 | 4 | 8 | 16) {
        return None;
    }
    let word = *table.get(index / quantum)?;
    if quantum == 1 {
        return Some(word);
    }
    let r = index & (quantum - 1);
    Some((word >> (quantum - 1)) + (word & ((1u32 << r) - 1)).count_ones())
}

/// Exact suffix counts for structural states, including present zero length.
/// Saturation at 2^32+1 is enough to reject an overflowing conditional rank.
pub fn structural_counts(domains: &[Vec<usize>], max_len: usize) -> QslResult<Vec<u64>> {
    if max_len > ANALYSIS_MAX_BYTES || domains.len() > 64 {
        return Err(QslError::InvalidSchema);
    }
    let cap = (1u64 << 32) + 1;
    let mut counts = vec![0u64; max_len + 1];
    counts[0] = 1;
    for domain in domains {
        if domain.len() > ANALYSIS_MAX_BYTES || domain.windows(2).any(|w| w[0] >= w[1]) {
            return Err(QslError::InvalidSchema);
        }
        let mut next = counts.clone();
        if let (Some(&min), Some(&max)) = (domain.first(), domain.last()) {
            if max - min + 1 == domain.len() {
                let mut window = 0u64;
                for s in 0..=max_len {
                    if s >= min {
                        window += counts[s - min];
                    }
                    if s > max {
                        window -= counts[s - max - 1];
                    }
                    next[s] = (counts[s] + window).min(cap);
                }
                counts = next;
                continue;
            }
        }
        if domain.len().saturating_mul(max_len + 1) > 4_000_000 {
            return Err(QslError::InvalidSchema);
        }
        for &len in domain {
            if len > max_len {
                continue;
            }
            for s in len..=max_len {
                next[s] = next[s].saturating_add(counts[s - len]).min(cap);
            }
        }
        counts = next;
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_quantums_and_rejected_derivatives() {
        let index: Vec<u32> = (0..257).map(|i| i / 3).collect();
        for q in [1, 2, 4, 8, 16] {
            let t = quantize_index(&index, q).unwrap();
            for (i, &r) in index.iter().enumerate() {
                assert_eq!(lookup_index(&t, q, i), Some(r));
            }
        }
        assert!(quantize_index(&[0, 2], 2).is_err());
        assert!(quantize_index(&[1, 0], 2).is_err());
        for q in [2, 4, 8, 16] {
            for mask in 0u32..(1u32 << (q - 1)) {
                let index: Vec<_> = (0..q)
                    .map(|r| 123 + (mask & ((1u32 << r) - 1)).count_ones())
                    .collect();
                let packed = quantize_index(&index, q).unwrap();
                for (r, &expected) in index.iter().enumerate() {
                    assert_eq!(lookup_index(&packed, q, r), Some(expected));
                }
            }
        }
    }
    #[test]
    fn collisions_and_zero_presence_are_counted() {
        assert_eq!(
            structural_counts(&[vec![1], vec![2], vec![3]], 6).unwrap()[3],
            2
        );
        assert_eq!(structural_counts(&[vec![0], vec![0]], 0).unwrap()[0], 4);
        for n in [31, 32, 33] {
            assert_eq!(
                structural_counts(&vec![vec![0]; n], 0).unwrap()[0],
                (1u64 << n).min((1u64 << 32) + 1)
            );
        }
    }
    #[test]
    fn exact_states_including_empty_are_exhaustive() {
        let s = crate::parse_qsl("&A[state_bits]{str?(0..3) a;str?(1..3) b;u16? c;}")
            .unwrap()
            .remove(0);
        let p = PredictPlan::analyze(&s).unwrap();
        for a in [None, Some(0), Some(1), Some(2), Some(3)] {
            for b in [None, Some(1), Some(2), Some(3)] {
                for c in [None, Some(2)] {
                    let values = [a, b, c];
                    let d = p.pack_lengths(&values).unwrap();
                    assert_eq!(p.unpack_lengths(d).unwrap(), values);
                }
            }
        }
        assert!(p.unpack_lengths(u32::MAX).is_err());
    }
    #[test]
    fn bit_capacity_and_unsupported_layouts_are_explicit() {
        for bits in [31, 32] {
            let max = (1u64 << bits) - 2;
            let s = crate::parse_qsl(&format!("&A[state_bits]{{str?(0..{max}) value;}}"))
                .unwrap()
                .remove(0);
            let p = PredictPlan::analyze(&s).unwrap();
            assert_eq!(p.used_bits, bits);
            let d = p.pack_lengths(&[Some(max as usize)]).unwrap();
            assert_eq!(p.unpack_lengths(d).unwrap(), [Some(max as usize)]);
        }
        for source in [
            "&A[state_bits]{str?(0..4294967295) value;}",
            "&A[predict_seq]{u8? a;u8? b;}",
            "&A[state_bits=Q16]{str?(0..100) a;}",
            "&A[state_bits,compact]{u8? a;}",
            "&A[state_bits=Q3]{u8? a;}",
            "&A[state_bits,state_bits=Q16]{u8? a;}",
            "&A{u8? a;u16 required;}",
        ] {
            assert!(crate::parse_qsl(source).is_err(), "{source}");
        }
        for count in [1, 2, 32] {
            let fields = (0..count).map(|i| format!("u8? f{i};")).collect::<String>();
            let s = crate::parse_qsl(&format!("&A[state_bits]{{{fields}}}"))
                .unwrap()
                .remove(0);
            assert_eq!(PredictPlan::analyze(&s).unwrap().used_bits, count);
        }
    }
}
