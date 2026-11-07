//! Compatibility path; implementation is owned by qsl-core.
pub use qsl_core::types::*;

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn widths() {
        let s = QslSchema {
            max_tag: 8,
            ..QslSchema::default()
        };
        assert_eq!(s.tag_bits(), 3);
        let s = QslSchema {
            max_tag: 1,
            ..QslSchema::default()
        };
        assert_eq!(s.tag_bits(), 0);
    }
}
