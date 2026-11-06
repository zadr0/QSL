//! Shared schema analysis before target-specific lowering.
use crate::{GenerationConfig, QslError, QslResult, QslSchema};

/// Immutable expanded schemas and configuration, without Rust storage lowering.
/// Backends may borrow this model and derive their own target-specific plans.
#[derive(Clone, Debug)]
pub struct QslSchemaBuilder {
    schemas: Vec<QslSchema>,
    config: GenerationConfig,
}
impl QslSchemaBuilder {
    pub fn schemas(&self) -> &[QslSchema] {
        &self.schemas
    }
    pub fn config(&self) -> GenerationConfig {
        self.config
    }
}

/// Compatibility name for the same immutable schema collection.
pub type AnalyzedModule = QslSchemaBuilder;

/// Expand templates and validate the current QSL schema contracts.
/// Storage selection, native-overlay rewriting and source emission are deferred.
/// Existing Rust custom-codec names remain opaque annotations in this model;
/// a different backend must explicitly support or reject their ABI.
pub fn analyze_schemas(
    schemas: &[QslSchema],
    config: GenerationConfig,
) -> QslResult<AnalyzedModule> {
    let schemas = expand_schemas(schemas)?;
    for schema in &schemas {
        schema.validate()?;
    }
    Ok(QslSchemaBuilder { schemas, config })
}

#[doc(hidden)]
pub fn expand_schemas(schemas: &[QslSchema]) -> QslResult<Vec<QslSchema>> {
    crate::schema_macro::expand_macros(schemas).map_err(|_| QslError::InvalidSchema)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{modernization::StorageKind, parse_qsl};
    #[test]
    fn analysis_keeps_storage_unselected_and_input_owned() {
        let module = {
            let schemas = parse_qsl("&Packet[safety,aggressive]{str(1..64) text;}").unwrap();
            analyze_schemas(&schemas, GenerationConfig::default()).unwrap()
        };
        assert_eq!(module.schemas()[0].fields[0].storage, StorageKind::Default);
        assert_eq!(module.schemas()[0].name, "Packet");
    }
    #[test]
    fn analysis_keeps_declared_overlay_intent_for_each_backend() {
        let schemas =
            parse_qsl("&View[transmutate,lifetime,safety]{u64 id;u32 value;str(4) tag;}").unwrap();
        let module = analyze_schemas(&schemas, GenerationConfig::default()).unwrap();
        assert!(module.schemas()[0].borrow_strings());
        assert!(!module.schemas()[0].options.native_overlay());
        assert_eq!(module.config(), GenerationConfig::default());
    }
}
