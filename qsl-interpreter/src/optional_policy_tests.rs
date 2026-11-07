use crate::*;
#[test]
fn bitmap_prediction_and_table_do_not_change_the_new_wire_identity() {
    let a = parse_qsl("&P[optionals=bitmap,safety]{u32 id;u16? a;u8? b;}").unwrap();
    let b = parse_qsl("&P[optionals=bitmap,predict_seq,opt_table,safety]{u32 id;u16? a;u8? b;}")
        .unwrap();
    assert!(!b[0].predict_seq());
    assert!(!b[0].sequential());
    assert_eq!(
        a[0].hashes(Default::default()).unwrap(),
        b[0].hashes(Default::default()).unwrap()
    );
}
#[test]
fn prediction_reuses_one_optional_reader_definition() {
    let builder =
        QslSchemaBuilder::from_source("&P[predict_seq,opt_table,safety]{u8 id;u32? a;u16? b;}")
            .unwrap();
    let source = String::from_utf8(builder.use_interpreter().unwrap()).unwrap();
    assert_eq!(source.matches("unsafe fn __qsl_read_a(").count(), 1);
    assert_eq!(source.matches("unsafe fn __qsl_read_b(").count(), 1);
    assert!(source.contains("_OPT_TABLE"));
}
