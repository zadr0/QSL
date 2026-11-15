//! Typed reflection stays on the compiler side; container implementations stay in core.
use core::num::NonZeroU8;
use qsl::{InterpreterBuilderExt, QslFieldType, QslMacro};
#[allow(dead_code)]
#[qsl::qsl_macro]
struct StorageMeta {
    name: Box<qsl::InlineString<512>>,
    values: qsl::InlineArray<u16, 8, NonZeroU8>,
    extra: Option<qsl::InlineArray<u8, 4, NonZeroU8>>,
}
#[allow(dead_code)]
#[qsl::qsl_macro]
struct InvalidArray {
    values: qsl::InlineArray<u8, 255, NonZeroU8>,
}
#[allow(dead_code)]
#[qsl::qsl_macro]
struct InvalidString {
    name: qsl::StackString<255, NonZeroU8>,
}
#[test]
fn typed_macro_preserves_container_domains_optionality_and_codegen() {
    let fields = StorageMeta::qsl_fields().unwrap();
    assert_eq!(fields.len(), 3);
    assert!(matches!(
        fields[0].data,
        qsl::QslType::CompressedString { min: 0, max: 512 }
    ));
    assert!(matches!(
        fields[1].data,
        qsl::QslType::RangedArray { min: 0, max: 8, .. }
    ));
    assert!(fields[2].optional);
    assert!(matches!(
        fields[2].data,
        qsl::QslType::RangedArray { min: 0, max: 4, .. }
    ));
    let mut builder = qsl::QslBuilder::new();
    builder.schem(|s| {
        s.name("Packet")
            .safety()
            .aggresive(true, false)
            .r#macro::<StorageMeta>();
    });
    let rust = String::from_utf8(builder.build().unwrap().use_interpreter().unwrap()).unwrap();
    assert!(rust.contains("::qsl::alloc::boxed::Box<::qsl::InlineString<512>>"));
    assert!(rust.contains("::qsl::InlineArray<u16, 8, core::num::NonZeroU8>"));
    assert!(rust.contains("decode_compressed_boxed::<512,u32>"));
}
#[test]
fn invalid_container_lengths_fail_during_reflection_without_constructing_values() {
    assert!(InvalidArray::qsl_fields().is_err());
    assert!(InvalidString::qsl_fields().is_err());
    const {
        assert!(!<Box<qsl::InlineString<65536>> as QslFieldType>::VALID);
    }
    const {
        assert!(!<qsl::InlineArray<Option<u8>, 4> as QslFieldType>::VALID);
    }
    const {
        assert!(<Option<qsl::InlineArray<u16, 8, NonZeroU8>> as QslFieldType>::VALID);
    }
}

#[cfg(feature = "rust-macros")]
#[test]
fn source_import_and_typed_builder_produce_identical_parent_wire_hashes() {
    #[allow(dead_code)]
    #[qsl::qsl_macro]
    struct RawMeta {
        name: qsl::StackString<32, NonZeroU8>,
        values: qsl::InlineArray<u16, 8, NonZeroU8>,
        extra: Option<qsl::InlineArray<u8, 4, NonZeroU8>>,
    }
    let mut typed = qsl::QslBuilder::new();
    typed.schem(|s| {
        s.name("Packet")
            .safety()
            .aggresive(true, false)
            .field("meta")
            .r#macro::<RawMeta>();
    });
    let typed = typed.build().unwrap();
    let source=qsl::rust_macro::parse_qsl_with_rust("&Packet[safety,aggressive[shift=true]]{RawMeta meta;}",&Default::default(),&["#[qsl_macro]struct RawMeta{name:StackString<32,NonZeroU8>,values:InlineArray<u16,8,NonZeroU8>,extra:Option<InlineArray<u8,4,NonZeroU8>>}"]).unwrap();
    let source = source.iter().find(|s| s.name == "Packet").unwrap();
    for algorithm in [
        qsl::SchemaHashAlgorithm::Xxh3_64,
        qsl::SchemaHashAlgorithm::Xxh3_128,
    ] {
        assert_eq!(
            typed.schemas()[0].hashes(algorithm).unwrap(),
            source.hashes(algorithm).unwrap()
        );
    }
    let from_source =
        qsl::QslSchemaBuilder::from_schemas(core::slice::from_ref(source), typed.config()).unwrap();
    let left = String::from_utf8(typed.use_interpreter().unwrap()).unwrap();
    let right = String::from_utf8(from_source.use_interpreter().unwrap()).unwrap();
    if left != right {
        let at = left
            .bytes()
            .zip(right.bytes())
            .position(|(a, b)| a != b)
            .unwrap_or(left.len().min(right.len()));
        panic!("generated Rust diverges at {at}: typed {:?}; source {:?}; flags {} / {}; options {:?} / {:?}",&left[at.saturating_sub(60)..(at+140).min(left.len())],&right[at.saturating_sub(60)..(at+140).min(right.len())],typed.schemas()[0].flags,source.flags,typed.schemas()[0].options,source.options);
    }
}

#[cfg(feature = "rust-macros")]
#[test]
fn inline_lz_source_and_typed_builder_have_identical_hashes_and_rust() {
    let mut typed = qsl::QslBuilder::new();
    typed.schem(|s| {
        s.name("Packet")
            .safety()
            .aggresive(true, false)
            .field("meta")
            .r#macro::<StorageMeta>();
    });
    let typed = typed.build().unwrap();
    let source=qsl::rust_macro::parse_qsl_with_rust("&Packet[safety,aggressive[shift=true]]{StorageMeta meta;}",&Default::default(),&["#[qsl_macro]struct StorageMeta{name:Box<InlineString<512>>,values:InlineArray<u16,8,NonZeroU8>,extra:Option<InlineArray<u8,4,NonZeroU8>>}"]).unwrap();
    let parent = source.iter().find(|s| s.name == "Packet").unwrap();
    for algorithm in [
        qsl::SchemaHashAlgorithm::Xxh3_64,
        qsl::SchemaHashAlgorithm::Xxh3_128,
    ] {
        assert_eq!(
            typed.schemas()[0].hashes(algorithm).unwrap(),
            parent.hashes(algorithm).unwrap()
        );
    }
    let source =
        qsl::QslSchemaBuilder::from_schemas(core::slice::from_ref(parent), typed.config()).unwrap();
    let a = typed.use_interpreter().unwrap();
    let b = source.use_interpreter().unwrap();
    assert!(
        a == b,
        "InlineString code generation differs between declaration methods"
    );
}
