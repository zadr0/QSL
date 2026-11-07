//! Exact operand quantization plans.
use crate::{QslError, QslResult, QslSchema, QslType};

#[derive(Debug, Clone)]
pub struct QuantField {
    pub index: usize,
    pub min: i128,
    pub max: i128,
    pub bits: usize,
    pub native_bytes: usize,
    pub rust_type: String,
    pub optional: bool,
    pub word_bytes: usize,
    pub max_start: usize,
}
#[derive(Debug, Clone)]
pub struct PayloadQuantPlan {
    pub quantum: usize,
    pub padding_bytes: usize,
    pub fields: Vec<QuantField>,
    pub header_bytes: usize,
    pub max_bytes: usize,
    pub min_bytes: usize,
    pub raw_max_bytes: Option<usize>,
    pub active: bool,
    pub reason: Option<&'static str>,
}
impl PayloadQuantPlan {
    pub fn analyze(s: &QslSchema) -> QslResult<Self> {
        let q = s.payload_quantum();
        let n = s.optional_count();
        let mut p = Self {
            quantum: q,
            padding_bytes: 1,
            fields: Vec::new(),
            header_bytes: if s.state_bits() { 4 } else { (n + 7) >> 3 },
            max_bytes: 0,
            min_bytes: 0,
            raw_max_bytes: None,
            active: false,
            reason: Some("payload quantization disabled"),
        };
        if !s.quant_requested() {
            return Ok(p);
        }
        if n > 64 {
            return Err(QslError::InvalidSchema);
        }
        if s.compact()
            || s.bitmap()
            || s.flags & crate::FLAG_ALIGNMENT != 0
            || s.flags & crate::FLAG_OPT_TABLE_LEGACY != 0
        {
            return Err(QslError::InvalidSchema);
        }
        let mut full_bits = 0usize;
        let mut required_bits = 0usize;
        let mut native = 0usize;
        let mut residues = 1u8;
        for (index, f) in s.fields.iter().enumerate() {
            f.data.validate(s.borrow_strings())?;
            let (min, max) = match f.data {
                QslType::Byte => (0, 255),
                QslType::RangedInteger { min, max, .. } => (min, max),
                QslType::Integer {
                    bits,
                    signed,
                    min: 0,
                } => {
                    if signed {
                        (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
                    } else {
                        (0, (1i128 << bits) - 1)
                    }
                }
                QslType::Flags(bits) => (0, (1i128 << bits) - 1),
                _ => {
                    p.reason = Some("non-numeric payload: preserve the explicit original codec");
                    return Ok(p);
                }
            };
            let bits = QslType::rank_bits(min, max)?;
            let bytes = f.data.fixed_wire_size()?.ok_or(QslError::InvalidSchema)?;
            full_bits += bits;
            if !f.optional {
                required_bits += bits;
            }
            native += bytes;
            let max_start = 7 - residues.leading_zeros() as usize;
            let word_bytes = [1, 2, 4, 8, 16]
                .into_iter()
                .find(|&w| w <= q && w * 8 >= bits + max_start)
                .unwrap_or(q);
            p.padding_bytes = p.padding_bytes.max(word_bytes);
            p.fields.push(QuantField {
                index,
                min,
                max,
                bits,
                native_bytes: bytes,
                rust_type: f.data.primitive()?.ok_or(QslError::InvalidSchema)?.into(),
                optional: f.optional,
                word_bytes,
                max_start,
            });
            let next = residues.rotate_left((bits & 7) as u32);
            residues = if f.optional { residues | next } else { next };
        }
        p.padding_bytes = 0; // Exact-width dispatch never touches a private tail.
        let raw_header = if s.state_bits() {
            4
        } else if s.predicted_opt_table() {
            crate::static_optional::StaticOptionalPlan::analyze(s)?.header_bytes()
        } else if s.predict_seq() || n == 0 {
            0
        } else if n == 1 {
            1
        } else {
            (s.count_bits() + n * s.tag_bits() + 7) >> 3
        };
        p.raw_max_bytes = Some(raw_header + native);
        p.max_bytes = p.header_bytes + ((full_bits + 7) >> 3);
        p.min_bytes = p.header_bytes + ((required_bits + 7) >> 3);
        if p.max_bytes
            .checked_add(p.padding_bytes)
            .is_none_or(|size| size >= raw_header + native)
        {
            p.reason =
                Some("no physical buffer saving after exact operands and padding; raw fallback");
            return Ok(p);
        }
        if s.predict_seq() && !s.opt_table() || s.state_bits() && s.state_bits_quantum() != 1 {
            return Err(QslError::InvalidSchema);
        }
        p.active = true;
        p.reason = None;
        Ok(p)
    }
}
