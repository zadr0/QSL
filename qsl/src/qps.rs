//! Lossless QPS operand codecs, independent of field-presence prediction.
//!
//! ByteQps groups any serialized payload into exact radix operands and restores
//! its original bytes. The original byte length is stored without quantization.
//! A separate scalar specialization packs statically bounded numeric ranks;
//! quotient and remainder are retained. Q2/Q4/Q8/Q16 choose generated kernels.
//! `opt_table` owns length-based prediction. `quant=Qn` aliases the scalar
//! schema selection; the outer byte-envelope API is always explicit.
pub use crate::payload_quant::{
    PayloadQuantPlan as QpsPlan, QuantField as QpsField, QuantizedFrame, QuantizedLease,
    QuantizedOut, QuantizedRecord, QuantizedRegions,
};
/// Revision of the specialized scalar representation; the byte envelope has its own revision.
/// Peers must agree on schema/configuration; no runtime format guessing occurs.
pub const QPS_CODEC_REVISION: u32 = 2;
/// General whole-payload operands, independent of the scalar specialization.
pub use crate::qps_bytes::{
    ByteFrame, ByteMode, ByteQps, CheckedBorrowedRecord, CheckedRecord, BYTE_FRAME_HEADER,
    BYTE_QPS_CODEC_REVISION,
};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn required_only_qps_is_not_a_predictor() {
        for q in [2, 4, 8, 16] {
            let s = crate::parse_qsl(&format!(
                "&P[qps=Q{q},safety]{{u64(0..15) a;u64(0..15) b;}}"
            ))
            .unwrap()
            .remove(0);
            assert!(s.qps());
            assert!(!s.state_bits());
            assert!(!s.predict_seq());
            assert!(!s.sequential());
            assert_eq!(s.quantum(), q);
            let p = QpsPlan::analyze(&s).unwrap();
            assert!(p.active);
            assert_eq!(p.max_bytes, 1);
        }
    }
    #[test]
    fn quant_alias_and_raw_fallback_are_explicit() {
        let a = crate::parse_qsl("&P[qps=Q4]{u64(0..15) a;}")
            .unwrap()
            .remove(0);
        let b = crate::parse_qsl("&P[quant=Q4]{u64(0..15) a;}")
            .unwrap()
            .remove(0);
        assert_eq!(a.flags, b.flags);
        let s = crate::parse_qsl("&P[qps=Q16]{u64 a;}").unwrap().remove(0);
        assert!(!QpsPlan::analyze(&s).unwrap().active);
        assert!(crate::parse_qsl("&P[qps=Q2,quant=Q4]{u64 a;}").is_err());
    }
    #[test]
    fn default_quantum_is_sixteen() {
        let s = crate::parse_qsl("&P[qps]{u64(0..15) value;}")
            .unwrap()
            .remove(0);
        assert_eq!(s.quantum(), 16);
    }
    #[test]
    fn prediction_is_separately_selected() {
        let s = crate::parse_qsl("&P[qps=Q16,opt_table]{u64(0..15) a;u64?(0..15) b;}")
            .unwrap()
            .remove(0);
        assert!(s.qps());
        assert!(s.opt_table());
        assert!(!s.state_bits());
        assert!(QpsPlan::analyze(&s).unwrap().active);
        let legacy = crate::parse_qsl("&P[state_bits=Q16]{u8? a;u16? b;}")
            .unwrap()
            .remove(0);
        assert!(!legacy.qps());
        assert!(legacy.state_bits());
        assert_eq!(legacy.state_bits_quantum(), 16);
    }
}
