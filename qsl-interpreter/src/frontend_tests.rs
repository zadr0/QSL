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
            qsl_lexer::layout::field_rust_type(&module.schemas()[0].schema().fields[0], false)
                .unwrap(),
            "String"
        );
        let mut generator = RustGenerator::new();
        generator.generate_prepared(&module).unwrap();
        assert_eq!(generator.config, config);
        let output = String::from_utf8(generator.output).unwrap();
        assert!(output.contains("pub text: ::qsl::alloc::string::String"));
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
        layout::StorageKind::Default
    );
    assert!(matches!(
        lowered.schemas()[0].schema().fields[0].storage,
        layout::StorageKind::StackString(64)
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

#[test]
fn auto_wire_preserves_logical_builder_and_canonicalizes_only_explicit_versions() {
    use crate::*;
    let a = QslSchemaBuilder::from_source(
        "&P[safety,auto[wire=1]]{u8 z;str(0..10) text;u32 a;u16? opt;}",
    )
    .unwrap();
    let b = QslSchemaBuilder::from_source(
        "&P[safety,auto[wire=1]]{u32 a;u8 z;str(0..10) text;u16? opt;}",
    )
    .unwrap();
    let names = |s: &QslSchema| s.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
    let prepared = prepare_schemas(a.schemas(), a.config()).unwrap();
    let p = &prepared.schemas()[0];
    assert_eq!(names(&a.schemas()[0]), ["z", "text", "a", "opt"]);
    assert_eq!(names(p.logical_schema()), ["z", "text", "a", "opt"]);
    assert_eq!(names(p.schema()), ["a", "z", "text", "opt"]);
    assert_eq!(
        p.schema()
            .declaration_fields()
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["z", "text", "a", "opt"]
    );
    let ha = a.schemas()[0].hashes(Default::default()).unwrap();
    let hb = b.schemas()[0].hashes(Default::default()).unwrap();
    assert_eq!(ha, hb);
    assert_eq!(ha.version, 2);
    assert_eq!(ha, p.schema().hashes(Default::default()).unwrap());
    let plain =
        QslSchemaBuilder::from_source("&P[safety,auto]{u8 z;str(0..10) text;u32 a;u16? opt;}")
            .unwrap();
    assert_eq!(
        plain.schemas()[0]
            .hashes(Default::default())
            .unwrap()
            .version,
        1
    );
    assert_ne!(
        ha.light,
        plain.schemas()[0].hashes(Default::default()).unwrap().light
    );
    let rust = String::from_utf8(a.use_interpreter().unwrap()).unwrap();
    assert!(
        rust.contains(
            "fn try_new(z: u8,text: ::qsl::alloc::string::String,a: u32,opt: Option<u16>)"
        ),
        "{rust}"
    );
    assert!(QslSchemaBuilder::from_source("&P[auto[wire=2]]{u32 x;}").is_err());
    assert!(QslSchemaBuilder::from_source("&P[auto[wire=1],transmutate]{u32 x;}").is_err());
}

#[test]
fn prepared_ir_is_owned_and_is_the_one_consumed_by_the_rust_backend() {
    use crate::frontend::prepare_schemas;
    let b = qsl_lexer::QslSchemaBuilder::from_source(
        "&P[safety,compact,auto[wire=1]]{u8 z;u32(1..65535) a;str(0..16) text;}",
    )
    .unwrap();
    let prepared = prepare_schemas(b.schemas(), b.config()).unwrap();
    let p = prepared.schemas()[0].plan();
    assert_eq!(
        p.fields
            .iter()
            .map(|f| (&*f.name, f.logical, f.wire))
            .collect::<Vec<_>>(),
        [("a", 1, 0), ("z", 0, 1), ("text", 2, 2)]
    );
    assert!(p
        .operations
        .iter()
        .any(|o| matches!(o, crate::analysis::Operation::Normalize { field: 0 })));
    assert!(p.operations.iter().any(|o| matches!(
        o,
        crate::analysis::Operation::Barrier {
            field: 2,
            kind: crate::analysis::RegionKind::Dynamic
        }
    )));
    let mut a = crate::QslInterpreter::new();
    a.generate_prepared(&prepared).unwrap();
    let mut c = crate::QslInterpreter::new();
    c.config = b.config();
    c.generate(b.schemas()).unwrap();
    assert_eq!(a.output, c.output);
}
