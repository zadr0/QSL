//! Rust backend implementation and schema-builder extension methods.
use crate::{QslResult, QslSchemaBuilder, RustGenerator, SchemaBackend};

impl SchemaBackend for RustGenerator {
    type Output = Vec<u8>;
    type Error = crate::QslError;
    fn compile(&mut self, builder: &QslSchemaBuilder) -> QslResult<Self::Output> {
        self.generate_analyzed(builder)?;
        Ok(std::mem::take(&mut self.output))
    }
}

/// Import this trait to select Rust generation on the shared builder.
pub trait InterpreterBuilderExt {
    fn use_interpreter(&self) -> QslResult<Vec<u8>>;
    /// Spelling-compatible alias; prefer use_interpreter in new code.
    fn use_interpenter(&self) -> QslResult<Vec<u8>> {
        self.use_interpreter()
    }
}
impl InterpreterBuilderExt for QslSchemaBuilder {
    fn use_interpreter(&self) -> QslResult<Vec<u8>> {
        self.use_backend(&mut RustGenerator::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_builder_can_drive_multiple_backends_without_schema_mutation() {
        struct OtherBackend;
        impl SchemaBackend for OtherBackend {
            type Output = usize;
            type Error = std::convert::Infallible;
            fn compile(&mut self, builder: &QslSchemaBuilder) -> Result<usize, Self::Error> {
                Ok(builder.schemas().iter().map(|s| s.fields.len()).sum())
            }
        }
        let builder =
            QslSchemaBuilder::from_source("&P[safety,aggressive]{u64 id;str(1..64) text;}")
                .unwrap();
        let hash = builder.schemas()[0].hashes(Default::default()).unwrap();
        let rust = builder.use_interpreter().unwrap();
        assert_eq!(rust, builder.use_interpenter().unwrap());
        assert!(String::from_utf8(rust).unwrap().contains("pub struct P"));
        assert_eq!(builder.use_backend(&mut OtherBackend).unwrap(), 2);
        assert_eq!(
            builder.schemas()[0].hashes(Default::default()).unwrap(),
            hash
        );
        assert_eq!(
            builder.schemas()[0].fields[1].storage,
            crate::modernization::StorageKind::Default
        );
    }
    #[test]
    fn failed_target_lowering_keeps_builder_reusable() {
        let builder =
            QslSchemaBuilder::from_source("&P[transmutate,safety]{str(1024) text;}").unwrap();
        assert!(builder.use_interpreter().is_err());
        assert_eq!(builder.schemas()[0].name, "P");
        assert_eq!(
            builder.schemas()[0].fields[0].storage,
            crate::modernization::StorageKind::Default
        );
    }
}
