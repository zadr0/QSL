//! Compatibility path; portable implementation is owned by qsl-core.
pub use qsl_core::qps::*;

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
