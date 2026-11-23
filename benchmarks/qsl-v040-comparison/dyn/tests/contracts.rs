use qsl_dyn_bench::*;

fn verify<C: Codec>(_: &mut (), length: usize) {
    let mut scratch = vec![0; CAPACITY];
    let mut buffers = Buffers::new(&mut scratch);
    for i in 0..FIXTURES {
        let fixture = Fixture::new(i, length);
        let n = C::encode(&C::prepare(&fixture), &mut buffers).unwrap();
        let wire = &buffers.out[..n];
        C::read(wire, |v| {
            assert_eq!(v, fixture.view(C::KIND), "{} fixture {i}", C::NAME)
        })
        .unwrap();
        assert!(
            C::read(&wire[..n - 1], |v| v.digest()).is_err(),
            "{} truncation",
            C::NAME
        );
        let mut altered = wire.to_vec();
        altered[8] ^= 1; // revision differs
        assert!(
            C::read(&altered, |v| v.digest()).is_err(),
            "{} revision",
            C::NAME
        );
        altered.copy_from_slice(wire);
        altered[6] = 1; // reserved header byte
        assert!(
            C::read(&altered, |v| v.digest()).is_err(),
            "{} reserved",
            C::NAME
        );
    }
}
#[test]
fn every_dyn_mode_restores_values_and_checks_framing() {
    visit_dyn_cases!(verify, &mut ());
}
#[test]
fn insufficient_caller_scratch_is_rejected() {
    let fixture = Fixture::new(7, 1024);
    let mut storage = vec![0; CAPACITY];
    let mut buffers = Buffers::new(&mut storage);
    let n = NumLine16::encode(&NumLine16::prepare(&fixture), &mut buffers).unwrap();
    assert!(generated::NumLine16::deserialize_checked(&buffers.out[..n], &mut []).is_err());
    let n = NumDynamic32::encode(&NumDynamic32::prepare(&fixture), &mut buffers).unwrap();
    assert!(generated::NumDynamic32::deserialize_checked(&buffers.out[..n], &mut []).is_err());
    let n = NumStrict64::encode(&NumStrict64::prepare(&fixture), &mut buffers).unwrap();
    assert!(generated::NumStrict64::deserialize_checked(&buffers.out[..n], &mut []).is_err());
}
#[test]
fn dyn_modes_reject_another_mode_or_width() {
    let fixture = Fixture::new(7, 0);
    let mut storage = vec![0; CAPACITY];
    let mut buffers = Buffers::new(&mut storage);
    let n = NumLine16::encode(&NumLine16::prepare(&fixture), &mut buffers).unwrap();
    assert!(NumLine32::read(&buffers.out[..n], |v| v.digest()).is_err());
    assert!(NumDynamic32::read(&buffers.out[..n], |v| v.digest()).is_err());
    let n = NumDynamic32::encode(&NumDynamic32::prepare(&fixture), &mut buffers).unwrap();
    assert!(NumStrict32::read(&buffers.out[..n], |v| v.digest()).is_err());
}
