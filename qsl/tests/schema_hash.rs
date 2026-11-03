use qsl::{parse_qsl, SchemaHashAlgorithm as Algorithm, SchemaHashKind as Kind, SchemaHashes};
fn hashes(source: &str) -> SchemaHashes {
    parse_qsl(source).unwrap()[0]
        .hashes(Algorithm::default())
        .unwrap()
}
#[test]
fn strict_names_and_light_wire_identity_are_separate() {
    let a = hashes("&Packet{u32 id;u8? flag;}");
    let b = hashes("&Message{u32 sequence;u8? status;}");
    assert_ne!(a.strict, b.strict);
    assert_eq!(a.light, b.light);
    assert_eq!(a, hashes(" /* doc */ &Packet { u32 id ; u8? flag; } "));
    assert_eq!(a.strict_hex().len(), 16);
    assert!(a.complete);
    let b = parse_qsl("&Packet{u32 id;u8? flag;}").unwrap()[0]
        .hashes(Algorithm::Xxh3_128)
        .unwrap();
    assert_eq!(b.strict_hex().len(), 32);
    assert_eq!(b.algorithm.bits(), 128);
    assert_ne!(a, b);
}
#[test]
fn codegen_policies_and_initializers_do_not_change_hashes() {
    let base = hashes("&P{u32 id;u8? a;}");
    for flags in [
        "safety",
        "inline",
        "inline=always",
        "inline=never",
        "alignment",
        "safety,inline=always,alignment",
    ] {
        assert_eq!(
            base,
            hashes(&format!(
                "&P[{flags}]{{u32 id|default=7|;u8? a|default=3|;}}"
            )),
            "{flags}"
        );
    }
    let base = hashes("&P{str(1..100) text;}");
    for mode in ["false", "true", "view", "guard"] {
        assert_eq!(
            base,
            hashes(&format!("&P{{str(1..100) text|binary_opt={mode}|;}}")),
            "{mode}"
        );
    }
    assert_eq!(base, hashes("&P[lifetime]{str(1..100) text;}"));
    assert_ne!(
        base.light,
        hashes("&P{str(1..100) text|binary_opt=inline|;}").light
    );
}
#[test]
fn tag_dispatch_preserves_default_identity_and_prediction_is_distinct() {
    let a = hashes("&P{u32 id;u8? a;u16? b;}");
    assert_eq!(
        a,
        hashes("&P[opt_table,safety,inline]{u32 id;u8? a;u16? b;}")
    );
    let combined = hashes("&P[opt_table,predict_seq]{u32 id;u8? a;u16? b;}");
    assert_ne!(a.light, combined.light);
    // Published mode-3 identity of the former mixed opt_table is retained.
    assert_eq!(combined.strict, 3654834731878387164);
    assert_eq!(combined.light, 845696593770569169);
    let a = hashes("&P[state_bits=Q1]{u8? a;u16? b;u32? c;u64? d;}");
    for q in [2, 4, 8, 16] {
        assert_eq!(
            a,
            hashes(&format!(
                "&P[state_bits=Q{q}]{{u8? a;u16? b;u32? c;u64? d;}}"
            ))
        );
    }
}
#[test]
fn changes_to_data_domains_order_and_wire_modes_change_both_hashes() {
    let base = hashes("&P{u32 id;u8? a;u16? b;}");
    for source in [
        "&P{u64 id;u8? a;u16? b;}",
        "&P{u32 id;u16? b;u8? a;}",
        "&P{u32 id;u8 a;u16? b;}",
        "&P{u32 id;u8?(0..7) a;u16? b;}",
        "&P[compact]{u32 id;u8? a;u16? b;}",
        "&P[optionals=bitmap]{u32 id;u8? a;u16? b;}",
        "&P[opt_table,predict_seq]{u32 id;u8? a;u16? b;}",
        "&P[opt_table=legacy]{u32 id;u8? a;u16? b;}",
        "&P[predict_seq]{u32 id;u8? a;u16? b;}",
        "&P[state_bits]{u32 id;u8? a;u16? b;}",
    ] {
        let other = hashes(source);
        assert_ne!(base.strict, other.strict, "{source}");
        assert_ne!(base.light, other.light, "{source}");
    }
    assert_ne!(
        hashes("&P[compact]{u8(0..7) a;u8(0..7) b;}").light,
        hashes("&P[compact,alignment]{u8(0..7) a;u8(0..7) b;}").light
    );
    assert_ne!(
        hashes("&P{str(1..100) a;}").light,
        hashes("&P{str(1..101) a;}").light
    );
    assert_ne!(
        hashes("&P{str(1..100) a;}").light,
        hashes("&P{str(1..100) a|decompress=true|;}").light
    );
    assert_ne!(
        hashes("&P{array<u8>(1..10) a;}").light,
        hashes("&P{array<u16>(1..10) a;}").light
    );
}
#[test]
fn qps_aliases_are_canonical_and_quantums_are_distinct() {
    let a = hashes("&P[qps]{u64(0..15) a;u64(0..15) b;}");
    for flags in ["qps=Q16", "quant=Q16", "quant=Q16,safety,inline"] {
        assert_eq!(
            a,
            hashes(&format!("&P[{flags}]{{u64(0..15) a;u64(0..15) b;}}"))
        );
    }
    for q in [2, 4, 8] {
        assert_ne!(
            a.light,
            hashes(&format!("&P[qps=Q{q}]{{u64(0..15) a;u64(0..15) b;}}")).light
        );
    }
    assert_ne!(
        hashes("&P[qps]{str text;}").light,
        hashes("&P{str text;}").light
    );
}
#[test]
fn constants_are_hashed_after_evaluation_not_by_identifier() {
    let constants = qsl::parse_deq("!define MAX: usize = 10 * 10;").unwrap();
    let schema =
        qsl::parse_qsl_with_constants("&P{str(1..!MAX) text;}", &constants.integer_values())
            .unwrap();
    assert_eq!(
        schema[0].hashes(Algorithm::default()).unwrap(),
        hashes("&P{str(1..100) text;}")
    );
}
#[test]
fn opaque_custom_codec_is_never_claimed_complete() {
    let a = hashes("&P{External custom;}");
    assert!(!a.complete);
    assert_ne!(a.light, hashes("&P{Another custom;}").light);
}
#[cfg(feature = "rust-macros")]
#[test]
fn nested_rust_macros_are_hashed_after_expansion() {
    let rust =
        "#[qsl_macro] struct Leaf { id:u32 } #[schem_macro] struct Meta { leaf:Leaf, value:u8 }";
    let schemas = qsl::rust_macro::parse_qsl_with_rust(
        "&P{Meta meta;u8? tail;}",
        &Default::default(),
        &[rust],
    )
    .unwrap();
    let p = schemas.iter().find(|s| s.name == "P").unwrap();
    assert_eq!(
        p.hashes(Algorithm::default()).unwrap(),
        hashes("&P{u32 meta_leaf_id;u8 meta_value;u8? tail;}")
    );
    assert!(p.hashes(Algorithm::default()).unwrap().complete);
}
#[test]
fn deq_hash_selection_is_validated_and_merged_independently() {
    assert_eq!(
        qsl::deq::GenerationConfig::default().use_hash,
        Algorithm::Xxh3_64
    );
    for (name, algorithm) in [
        ("xxHash", Algorithm::Xxh3_64),
        ("XXH3_64", Algorithm::Xxh3_64),
        ("xxHash128", Algorithm::Xxh3_128),
        ("XXH3_128", Algorithm::Xxh3_128),
    ] {
        assert_eq!(
            qsl::parse_deq(&format!("!config USE_HASH = {name};"))
                .unwrap()
                .generation
                .use_hash,
            algorithm
        );
    }
    for source in [
        "!config USE_HASH = md5;",
        "!config USE_HASH = 128;",
        "!config USE_HASH = true;",
        "!config USE_HASH = xxHash;!config USE_HASH = xxHash128;",
    ] {
        assert!(qsl::parse_deq(source).is_err());
    }
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("schema-hash-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let schema = dir.join("test.qsl");
    let hash = dir.join("hash.deq");
    let rt = dir.join("rt.deq");
    std::fs::write(&schema, "&P{u32 id;}").unwrap();
    std::fs::write(&hash, "!config USE_HASH = xxHash128;").unwrap();
    std::fs::write(&rt, "!config QSL_RT_DEFAULT = 7;").unwrap();
    let out = qsl::build_qsl_files(
        &[schema.to_str().unwrap()],
        &[hash.to_str().unwrap(), rt.to_str().unwrap()],
        &dir,
    )
    .unwrap();
    let generated = std::fs::read_to_string(out).unwrap();
    let expected = parse_qsl("&P{u32 id;}").unwrap()[0]
        .hashes(Algorithm::Xxh3_128)
        .unwrap();
    assert!(generated.contains(&format!("strict: 0x{}u128", expected.strict_hex())));
    assert!(qsl::build_qsl_files(
        &[schema.to_str().unwrap()],
        &[hash.to_str().unwrap(), hash.to_str().unwrap()],
        &dir
    )
    .is_err());
}
#[test]
fn descriptor_has_explicit_version_domain_and_little_endian_lengths() {
    let s = &parse_qsl("&P{u8 x;}").unwrap()[0];
    let strict = s.hash_descriptor(Kind::Strict).unwrap();
    assert_eq!(&strict[..14], b"QSL-SCHEMA\0\x01\0\x01");
    assert_eq!(&strict[14..22], &1u64.to_le_bytes());
    assert_eq!(strict[22], b'P');
    assert_ne!(strict, s.hash_descriptor(Kind::Light).unwrap());
    assert_eq!(strict, s.hash_descriptor(Kind::Strict).unwrap());
}

#[test]
fn canonical_v1_golden_hashes_are_stable_for_both_algorithms() {
    let schema = &parse_qsl("&P{u8 x;}").unwrap()[0];
    let a = schema.hashes(Algorithm::Xxh3_64).unwrap();
    assert_eq!(a.strict_hex(), "c9007ab817611938");
    assert_eq!(a.light_hex(), "37232e631d533f72");
    let b = schema.hashes(Algorithm::Xxh3_128).unwrap();
    assert_eq!(b.strict_hex(), "5469848a69db6f5d925e5e88c73c0537");
    assert_eq!(b.light_hex(), "af4df4a6c83f9028122821b46c0879c7");
}

#[test]
fn native_bridge_agrees_with_generation_across_processes_and_rejects_duplicate_config() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let run = |args: &[&str]| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_qsl-analyze"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(b"&P{u8 x;}").unwrap();
        let result = child.wait_with_output().unwrap();
        assert!(result.status.success());
        String::from_utf8(result.stdout).unwrap()
    };
    let a = run(&["--generate"]);
    assert_eq!(a, run(&["--generate"]));
    assert!(a.contains("\"strict\":\"c9007ab817611938\""));
    assert!(a.contains("strict: 0xc9007ab817611938u128"));
    let b = run(&["--generate", "--deq-text", "!config USE_HASH = xxHash128;"]);
    assert!(b.contains("5469848a69db6f5d925e5e88c73c0537"));
    let error = run(&[
        "--deq-text",
        "!config USE_HASH = xxHash;",
        "--deq-text",
        "!config USE_HASH = xxHash128;",
    ]);
    assert!(error.contains("\"ok\":false"));
    assert!(error.contains("more than once"));
}
