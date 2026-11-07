//! Compatibility path; implementation is owned by qsl-core.
pub use qsl_core::static_index::*;

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
