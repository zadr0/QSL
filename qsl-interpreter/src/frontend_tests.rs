mod tests {
    use crate::*;
    use crate::{parse_qsl, RustGenerator};
    #[test]
    fn prepared_data_owns_input_and_controls_generation_configuration() {
        let config = GenerationConfig {
            stack_limit: 8,
            max_data_size: 1024,
            ..Default::default()
        };
        let module = {
            let text = String::from("&Packet[safety,aggressive]{str(1..16) text;}");
            prepare_schemas(&parse_qsl(&text).unwrap(), config).unwrap()
        };
        assert_eq!(module.config(), config);
        assert_eq!(module.schemas()[0].schema().name, "Packet");
        assert_eq!(
            crate::modernization::field_rust_type(&module.schemas()[0].schema().fields[0], false)
                .unwrap(),
            "String"
        );
        let mut generator = RustGenerator::new();
        generator.generate_prepared(&module).unwrap();
        assert_eq!(generator.config, config);
        let output = String::from_utf8(generator.output).unwrap();
        assert!(output.contains("pub text: String"));
        assert!(output
            .split_whitespace()
            .collect::<String>()
            .contains("MAX_DATA_SIZE:usize=1024;"));
    }
    #[test]
    fn prepared_and_compatible_generation_have_identical_rust_bytes() {
        let source="&Plain[safety]{u64 id;str(1..100) name;u8? status;} &Q[qps,safety]{u64(0..15) x;u64?(0..15) y;} &View[transmutate,lifetime,safety]{u64 id;u32 value;str(4) tag;}";
        let schemas = parse_qsl(source).unwrap();
        for config in [
            GenerationConfig::default(),
            GenerationConfig {
                stack_limit: 8,
                qsl_rt: true,
                qsl_rt_default: 2,
                ..Default::default()
            },
        ] {
            let prepared = prepare_schemas(&schemas, config).unwrap();
            assert!(prepared.schemas()[2].native_overlay());
            let mut compatible = RustGenerator::new();
            compatible.config = config;
            compatible.generate(&schemas).unwrap();
            let mut direct = RustGenerator::new();
            direct.generate_prepared(&prepared).unwrap();
            assert_eq!(compatible.output, direct.output);
        }
    }
    #[test]
    fn invalid_preparation_is_rejected_before_emission() {
        let schemas = parse_qsl("&View[transmutate,safety]{str(1024) text;}").unwrap();
        assert!(matches!(
            prepare_schemas(&schemas, GenerationConfig::default()),
            Err(QslError::InvalidSchema)
        ));
        let config = GenerationConfig {
            stack_limit: 1024,
            ..Default::default()
        };
        assert!(prepare_schemas(&schemas, config).is_ok());
    }
}

#[test]
fn shared_analysis_survives_rust_lowering_and_keeps_wire_identity() {
    use crate::*;
    let schemas = parse_qsl("&Packet[safety,aggressive]{str(1..64) text;}").unwrap();
    let module = analyze_schemas(&schemas, GenerationConfig::default()).unwrap();
    let original_hashes = module.schemas()[0]
        .hashes(SchemaHashAlgorithm::default())
        .unwrap();
    let mut generator = RustGenerator::new();
    generator.generate_analyzed(&module).unwrap();
    let lowered = prepare_schemas(module.schemas(), module.config()).unwrap();
    assert_eq!(
        module.schemas()[0].fields[0].storage,
        modernization::StorageKind::Default
    );
    assert!(matches!(
        lowered.schemas()[0].schema().fields[0].storage,
        modernization::StorageKind::StackString(64)
    ));
    assert_eq!(
        original_hashes,
        module.schemas()[0]
            .hashes(SchemaHashAlgorithm::default())
            .unwrap()
    );
    let mut compatible = RustGenerator::new();
    compatible.generate(&schemas).unwrap();
    assert_eq!(generator.output, compatible.output);
}
