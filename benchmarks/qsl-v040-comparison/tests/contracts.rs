use qsl_v040_comparison::*;
fn verify_one<C: Codec>(_: &mut (), length: usize) {
    qsl_v040_comparison::cases::verify::<C>(length)
}
fn frame<C: Codec>(f: &Fixture) -> Vec<u8> {
    let mut s = vec![0; CAPACITY];
    let mut b = Buffers::new(&mut s);
    let n = C::encode(&C::prepare(f), &mut b).unwrap();
    b.out[..n].to_vec()
}
#[test]
fn all_codecs_restore_every_field() {
    visit_cases!(verify_one, &mut ());
}
#[test]
fn equivalent_qsl_modes_preserve_exact_bytes() {
    for i in 0..FIXTURES {
        let f = Fixture::new(i, 256);
        assert_eq!(frame::<QNumDefault>(&f), frame::<QNumPointing>(&f));
        assert_eq!(frame::<QNumCompact>(&f), frame::<QNumBmi2>(&f));
        assert_eq!(frame::<QOptDefault>(&f), frame::<QOptTable>(&f));
        assert_eq!(frame::<QOptPredict>(&f), frame::<QOptPredictTable>(&f));
        assert_eq!(frame::<QOptBitmap>(&f), frame::<QOptBitmapTable>(&f));
        assert_eq!(frame::<QTextHeap>(&f), frame::<QTextInline>(&f));
        assert_eq!(frame::<QTextHeap>(&f), frame::<QTextBorrow>(&f));
        assert_eq!(frame::<QNumDefault>(&f).len(), 16);
        assert_eq!(frame::<QNumCompact>(&f).len(), 6);
        assert_eq!(frame::<QNumAligned>(&f).len(), 8);
    }
}
fn truncated_text<C: Codec>(_: &mut (), length: usize) {
    if C::KIND != Kind::Text {
        return;
    }
    let f = Fixture::new(11, length);
    let bytes = frame::<C>(&f);
    assert!(
        C::read(&bytes[..bytes.len() - 1], |v| v.digest()).is_err(),
        "{}",
        C::NAME
    );
}
#[test]
fn truncated_text_is_rejected() {
    visit_cases!(truncated_text, &mut ());
}
#[test]
fn empty_utf8_is_not_absence_and_is_supported() {
    for length in [0, 1, 31, 32, 1023, 1024] {
        let f = Fixture::new(23, length);
        let data = frame::<QTextHeap>(&f);
        QTextInline::read(&data, |v| assert_eq!(v, f.view(Kind::Text))).unwrap();
        QTextBorrow::read(&data, |v| assert_eq!(v, f.view(Kind::Text))).unwrap();
    }
}
#[test]
fn rejects_out_of_domain_wire() {
    use prost::Message;
    let wire = ProtoNumeric {
        f0: 4096,
        f1: 0,
        f2: 0,
        f3: 0,
    }
    .encode_to_vec();
    assert!(PNumVar::read(&wire, |v| v.digest()).is_err());
    let mut scratch = vec![0; CAPACITY];
    let mut out = Buffers::new(&mut scratch);
    let mut f = Fixture::new(0, 0);
    f.numbers[2] = 4096;
    assert!(QNumDefault::encode(&QNumDefault::prepare(&f), &mut out).is_err());
    assert!(QNumCompact::encode(&QNumCompact::prepare(&f), &mut out).is_err());
    assert!(BNumVar::encode(&BNumVar::prepare(&f), &mut out).is_err());
    assert!(CNum::encode(&CNum::prepare(&f), &mut out).is_err());
}
