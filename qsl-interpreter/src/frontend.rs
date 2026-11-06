//! Rust-target preparation, isolated from the shared analyzed schema model.
use crate::{GenerationConfig, QslResult, QslSchema};

#[derive(Clone, Debug)]
pub struct PreparedSchema {
    schema: QslSchema,
    overlay: bool,
}
impl PreparedSchema {
    pub fn schema(&self) -> &QslSchema {
        &self.schema
    }
    pub fn native_overlay(&self) -> bool {
        self.overlay
    }
}

#[derive(Clone, Debug)]
pub struct PreparedModule {
    pub(crate) schemas: Vec<PreparedSchema>,
    pub(crate) config: GenerationConfig,
}
impl PreparedModule {
    pub fn schemas(&self) -> &[PreparedSchema] {
        &self.schemas
    }
    pub fn config(&self) -> GenerationConfig {
        self.config
    }
}

/// Expand templates, choose native storage and validate before Rust emission.
pub fn prepare_schemas(
    schemas: &[QslSchema],
    config: GenerationConfig,
) -> QslResult<PreparedModule> {
    prepare_expanded(expand_schemas(schemas)?, config)
}
#[doc(hidden)]
pub fn expand_schemas(schemas: &[QslSchema]) -> QslResult<Vec<QslSchema>> {
    qsl_lexer::frontend::expand_schemas(schemas)
}
#[doc(hidden)]
pub fn prepare_expanded(
    expanded: Vec<QslSchema>,
    config: GenerationConfig,
) -> QslResult<PreparedModule> {
    let mut schemas = Vec::with_capacity(expanded.len());
    for mut schema in expanded {
        let overlay = schema.options.transmutate && schema.borrow_strings();
        if overlay {
            schema.options = schema.options.with_native_overlay();
            schema.flags &= !(crate::FLAG_LIFETIME | crate::FLAG_FIELD_BORROW);
        }
        let schema = crate::modernization::lower_storage(&schema, config.stack_limit)?;
        schema.validate()?;
        schemas.push(PreparedSchema { schema, overlay });
    }
    Ok(PreparedModule { schemas, config })
}
