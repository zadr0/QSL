//! Compatibility path; implementation is owned by qsl-core.
pub use qsl_core::analysis::*;

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn bit_windows_and_byte_barriers_select_minimal_registers() {
        for (source, read, write) in [
            (
                "&A[compact]{u8(0..7) p;str(1..100) t|binary_opt=view|;u8(0..7) q;}",
                vec![16, 8],
                vec![16, 8],
            ),
            (
                "&A[compact]{u8(0..7) p;str t|binary_opt=view|;u16 q;}",
                vec![64, 0],
                vec![64, 0],
            ),
            (
                "&A[compact]{u8(0..7) p;u64 n;str(0..100) t|binary_opt=view|;u8(0..7) q;}",
                vec![128, 8],
                vec![128, 8],
            ),
            ("&A{str t;}", vec![0], vec![0]),
        ] {
            let schema = crate::parse_qsl(source).unwrap();
            let plan = QslIoPlan::analyze(&schema[0]).unwrap();
            assert_eq!(
                plan.blocks
                    .iter()
                    .map(|b| b.reader.bits())
                    .collect::<Vec<_>>(),
                read
            );
            assert_eq!(
                plan.blocks
                    .iter()
                    .map(|b| b.writer.bits())
                    .collect::<Vec<_>>(),
                write
            );
        }
    }
}
