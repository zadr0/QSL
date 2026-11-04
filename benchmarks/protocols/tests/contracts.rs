use qsl_protocol_comparison::*;
#[test]
fn every_codec_restores_identical_semantics_and_optional_states() {
    for kind in [Kind::Numeric, Kind::Mixed] {
        for codec in Codec::ALL {
            let mut scratch = vec![0; 8192];
            let mut encoder = Encoder::new(&mut scratch);
            for i in 0..32 {
                let f = Fixture::new(i);
                let wire = encoder.encode(codec, kind, &f).unwrap();
                assert_eq!(
                    read(codec, kind, wire).unwrap(),
                    expected(kind, &f.record),
                    "{codec:?} {kind:?} {i}"
                );
            }
        }
    }
}
#[test]
fn all_codecs_enforce_shared_domains_and_input_caps() {
    for kind in [Kind::Numeric, Kind::Mixed] {
        for codec in Codec::ALL {
            let mut scratch = vec![0; 8192];
            let mut encoder = Encoder::new(&mut scratch);
            let mut f = Fixture::new(3);
            f.record.v[0] = 4096;
            f.numeric.values[0] = 4096;
            f.mixed.values[0] = 4096;
            assert!(
                encoder.encode(codec, kind, &f).is_err(),
                "{codec:?} {kind:?}"
            );
            assert!(read(codec, kind, &vec![0; MAX_RECORD + 1]).is_err());
        }
    }
}
#[test]
fn malformed_frames_do_not_panic() {
    for kind in [Kind::Numeric, Kind::Mixed] {
        for codec in Codec::ALL {
            let mut scratch = vec![0; 8192];
            let mut encoder = Encoder::new(&mut scratch);
            let f = Fixture::new(3);
            let valid = encoder.encode(codec, kind, &f).unwrap().to_vec();
            for len in [
                0,
                1,
                2,
                3,
                4,
                7,
                8,
                15,
                16,
                valid.len() / 2,
                valid.len() - 1,
            ] {
                let b = &valid[..len.min(valid.len())];
                assert!(std::panic::catch_unwind(|| {
                    let _ = read(codec, kind, b);
                })
                .is_ok());
            }
            for seed in 0..256 {
                let mut bad = valid.clone();
                let at = seed * 73 % bad.len();
                bad[at] ^= ((seed % 255) + 1) as u8;
                assert!(
                    std::panic::catch_unwind(|| {
                        let _ = read(codec, kind, &bad);
                    })
                    .is_ok(),
                    "{codec:?} {kind:?} {seed}"
                );
            }
        }
    }
}

#[test]
fn prepared_qsl_numeric_has_initialized_padding_and_preserves_exact_bytes() {
    let mut scratch = vec![0; 8192];
    let mut e = Encoder::new(&mut scratch);
    for i in 0..256 {
        let f = Fixture::new(i);
        let frame = Frame::new(e.encode(Codec::QslOptimized, Kind::Numeric, &f).unwrap());
        assert_eq!(
            read_frame::<1, 0>(&frame).unwrap(),
            read(Codec::QslOptimized, Kind::Numeric, frame.as_slice()).unwrap()
        );
    }
    let bad = Frame::new(&[]);
    assert!(read_frame::<1, 0>(&bad).is_err());
}

#[test]
fn prost_reuse_matches_owned_decode() {
    let mut decoder = Decoder::default();
    let mut scratch = vec![0; 8192];
    let mut encoder = Encoder::new(&mut scratch);
    for k in [Kind::Numeric, Kind::Mixed] {
        for i in 0..64 {
            let f = Fixture::new(i);
            let bytes = encoder.encode(Codec::Protobuf, k, &f).unwrap();
            assert_eq!(
                decoder.protobuf(k, bytes).unwrap(),
                read(Codec::Protobuf, k, bytes).unwrap()
            );
        }
    }
}

#[test]
fn narrowing_never_hides_out_of_range_original_values() {
    for codec in Codec::ALL {
        let mut f = Fixture::new(3);
        f.record.v[0] = 65537;
        f.numeric.values[0] = 65537;
        f.mixed.values[0] = 65537;
        let mut scratch = vec![0; 8192];
        let mut encoder = Encoder::new(&mut scratch);
        assert!(encoder.encode(codec, Kind::Mixed, &f).is_err());
    }
}
