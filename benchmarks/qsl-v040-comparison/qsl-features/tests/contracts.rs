use qsl_feature_bench::{generated::*, *};

#[test]
fn compatible_native_readers_validate_layout_and_fields() {
    let (aligned, _) = native_frames();
    let mut bad = Aligned(aligned[0].0);
    bad.0[12..14].copy_from_slice(&4096u16.to_le_bytes());
    assert!(Plain::deserialize_checked(&bad.0).is_err());
    assert!(NativeCopy::deserialize_transmutate_copy(&bad.0).is_err());
    assert!(Native::deserialize_overlay(&bad.0).is_err());
}
#[test]
fn string_variants_preserve_bytes_and_raw_stack_wire() {
    let mut buffers = [[0u8; BUFFER]; 3];
    for corpus in CORPORA {
        for i in 0..RECORDS {
            let original = text(corpus, i);
            let raw = HeapText::prepare(i as u32, original.clone()).write(&mut buffers[0]);
            let stack = StackText::prepare(i as u32, original.clone()).write(&mut buffers[1]);
            let lz = CompressedText::prepare(i as u32, original.clone()).write(&mut buffers[2]);
            assert_eq!(&buffers[0][..raw], &buffers[1][..stack]);
            assert_eq!(
                CompressedText::read(&buffers[2][..lz]).fields(),
                (i as u32, original.as_str())
            );
            if corpus == "periodic_16" {
                assert!(lz < raw / 2, "short periodic prefix should compress");
            } else {
                assert_eq!(lz, 1030, "logs and noise use the raw fallback");
            }
        }
    }
}
