use qsl_optimized_bench::{
    cases::{registry, COUNT, VALIDATION_OFFSET},
    MAX_FRAME,
};
#[test]
fn candidate_codecs_and_views_on_both_corpora() {
    let mut buffer = Box::new([0; MAX_FRAME]);
    for case in registry() {
        // The constructors check deep equality for API records and verify
        // that the complete text view aliases the actual encoded input.
        for offset in [0, VALIDATION_OFFSET] {
            for index in (0..COUNT).step_by(11) {
                let record = case.new_input(offset, index, &mut buffer).unwrap();
                case.run(&record, &mut buffer).unwrap();
            }
        }
    }
}
