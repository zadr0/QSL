//! Compatibility path; implementation is owned by qsl-core.
pub use qsl_core::bitmap::*;

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn boundaries_and_spare_bits() {
        for n in 0..=128 {
            let p = BitmapPlan::new(n).unwrap();
            assert_eq!(
                p.width,
                match n {
                    0 => 0,
                    1..=8 => 8,
                    9..=16 => 16,
                    17..=32 => 32,
                    33..=64 => 64,
                    _ => 128,
                }
            );
            assert!(p.validate(0).is_ok());
            for marker in 1..=p.width {
                let bit = 1u128 << (p.width - marker);
                assert_eq!(p.validate(bit).is_ok(), marker <= n);
            }
        }
        assert!(BitmapPlan::new(129).is_err());
    }
    #[test]
    fn traversal_matches_reverse_markers() {
        for n in [1, 8, 9, 16, 17, 32, 33, 64, 65, 128] {
            let p = BitmapPlan::new(n).unwrap();
            let mut mask = (1..=n)
                .step_by(3)
                .fold(0, |m, i| m | p.marker_bit(i).unwrap());
            let mut markers = Vec::new();
            while mask != 0 {
                markers.push(p.width - mask.trailing_zeros() as usize);
                mask &= mask - 1;
            }
            assert_eq!(
                markers,
                (1..=n)
                    .step_by(3)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
            );
        }
    }
}
