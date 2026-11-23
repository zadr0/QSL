use qsl_layout_bench::{generated::*, *};

#[test]
fn measured_layouts_round_trip_the_same_fields() {
    validate::<Declared>();
    validate::<Auto>();
    validate::<Canonical>();
    validate::<Manual>();
}

#[test]
fn wire_order_is_a_separate_choice_from_memory_layout() {
    let record = Fields::new(13);
    assert_eq!(
        example_frame::<Declared>(record),
        example_frame::<Auto>(record)
    );
    assert_ne!(
        example_frame::<Declared>(record),
        example_frame::<Manual>(record)
    );
}
