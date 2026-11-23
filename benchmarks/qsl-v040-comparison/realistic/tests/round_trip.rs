use qsl_realistic_bench::{
    fixture::{make, RECORDS, WORKLOADS},
    normalize, CODECS,
};
#[test]
fn all_codecs_preserve_union_nested_presence_and_values() {
    for codec in CODECS {
        for workload in WORKLOADS {
            for i in 0..RECORDS {
                let value = make(workload, i);
                let frame = codec.encode(value.clone()).unwrap();
                assert_eq!(
                    codec.decode(&frame).unwrap(),
                    value,
                    "{codec}/{workload}/{i}"
                );
                if workload == "api" {
                    let accepted = normalize(value).unwrap();
                    let frame = codec.encode(accepted.clone()).unwrap();
                    assert_eq!(codec.decode(&frame).unwrap(), accepted);
                }
            }
        }
    }
}
