use qsl::qps_bytes::{ByteMode, ByteQps};
#[test]
fn qps_byte_frame_has_unquantized_length_and_exact_group_words() {
    let input = (0u8..16).cycle().take(256).collect::<Vec<_>>();
    let mut storage = [0; 272];
    let frame = ByteQps::<16>::encode_to(&input, &mut storage).unwrap();
    assert_eq!(frame.mode(), ByteMode::Packed);
    assert_eq!(frame.as_slice().len(), 160);
    assert_eq!(
        &frame.as_slice()[..16],
        &[81, 80, 66, 51, 16, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
    );
    for group in frame.as_slice()[16..].as_chunks::<9>().0 {
        assert_eq!(group, &[4, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe]);
    }
    let mut restored = [0; 256];
    assert_eq!(frame.decode_to(&mut restored).unwrap(), input);
    let frame = ByteQps::<2>::encode_to(&input, &mut storage).unwrap();
    assert_eq!(frame.mode(), ByteMode::Raw);
    assert_eq!(frame.as_slice().len(), 272);
    assert_eq!(&frame.as_slice()[16..], input);
}
#[test]
fn every_quantum_preserves_high_constant_base() {
    macro_rules! check {
        ($q:literal) => {{
            let mut storage = [0; 80];
            let frame = ByteQps::<$q>::encode_to(&[200; 64], &mut storage).unwrap();
            assert_eq!(frame.mode(), ByteMode::Packed);
            assert_eq!(&frame.as_slice()[..8], &[81, 80, 66, 51, $q, 1, 200, 0]);
            assert_eq!(frame.as_slice().len(), 16 + 64 / $q);
            assert!(frame.as_slice()[16..].iter().all(|&b| b == 0));
            let mut restored = [0; 64];
            assert_eq!(frame.decode_to(&mut restored).unwrap(), &[200; 64]);
        }};
    }
    check!(2);
    check!(4);
    check!(8);
    check!(16);
}
