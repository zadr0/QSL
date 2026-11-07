//! Schema collection and backend contracts. No code generator is required.
pub use crate::frontend::QslSchemaBuilder;
use crate::{deq::parse_deq, parser::parse_qsl_raw_with_constants};
use crate::{GenerationConfig, QslResult, QslSchema};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

/// A target-specific consumer of an analyzed schema collection.
/// Each backend owns its output and error types and derives its own lowering.
pub trait SchemaBackend {
    type Output;
    type Error;
    fn compile(&mut self, builder: &QslSchemaBuilder) -> Result<Self::Output, Self::Error>;
}

impl QslSchemaBuilder {
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
            config: GenerationConfig::default(),
            primitives: Default::default(),
        }
    }
    /// Register an explicit transparent primitive before collecting source.
    pub fn with_primitive<T: crate::QslPrimitive>(mut self, name: &str) -> QslResult<Self> {
        let p = T::REPRESENTATION;
        p.validate()?;
        let n = p.bits as usize / 8;
        let align = crate::modernization::native_type_layout(&p.data(), &Default::default())
            .ok_or(crate::QslError::InvalidSchema)?
            .alignment;
        if name.is_empty()
            || !name
                .chars()
                .enumerate()
                .all(|(i, c)| c == '_' || c.is_ascii_alphabetic() || i > 0 && c.is_ascii_digit())
            || T::SIZE != n
            || T::COMP_SIZE > n
            || std::mem::size_of::<T>() != n
            || std::mem::align_of::<T>() != align
            || std::mem::needs_drop::<T>()
        {
            return Err(crate::QslError::InvalidSchema);
        }
        if self.primitives.insert(name.into(), p).is_some() {
            return Err(crate::QslError::InvalidSchema);
        }
        Ok(self)
    }
    /// Registered metadata is applied by the parser before its existing checks.
    pub fn with_source(mut self, source: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut raw = crate::parser::parse_qsl_raw_with_primitives(
            source,
            &Default::default(),
            &self.primitives,
        )?;
        raw.splice(0..0, self.schemas);
        self.schemas = crate::frontend::expand_schemas(&raw)?;
        for schema in &self.schemas {
            schema.validate()?;
        }
        Ok(self)
    }

    #[cfg(feature = "rust-macros")]
    pub fn with_rust_templates(
        mut self,
        sources: &[&str],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut expanded = crate::rust_macro::expand_with_rust(&self.schemas, sources)?;
        for schema in &mut expanded {
            crate::apply_primitive_bindings(schema, &self.primitives)?;
        }
        self.schemas = crate::frontend::expand_schemas(&expanded)?;
        for schema in &self.schemas {
            schema.validate()?;
        }
        Ok(self)
    }
    /// Build from already parsed schemas, e.g. an editor or Rust macro importer.
    pub fn from_schemas(schemas: &[QslSchema], config: GenerationConfig) -> QslResult<Self> {
        crate::analyze_schemas(schemas, config)
    }
    pub fn from_source(source: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_source_with_config(source, GenerationConfig::default())
    }
    pub fn from_source_with_config(
        source: &str,
        config: GenerationConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let schemas = crate::parse_qsl(source)?;
        Ok(Self::from_schemas(&schemas, config)?)
    }
    /// Load explicit QSL/DEQ paths in caller order; never emit or write code.
    pub fn from_files(
        qsl_files: &[&str],
        deq_files: &[&str],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        load_inputs(qsl_files, deq_files, &[])
    }
    #[cfg(feature = "rust-macros")]
    pub fn from_files_with_rust(
        qsl_files: &[&str],
        deq_files: &[&str],
        rust_files: &[&str],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        load_inputs(qsl_files, deq_files, rust_files)
    }
    /// Scan one QSL file or a directory's immediate QSL and DEQ files.
    /// Directory entries are sorted. Rust template inputs are always explicit.
    pub fn scan(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        if path.is_file() {
            let name = path.to_str().ok_or("QSL path is not UTF-8")?;
            return Self::from_files(&[name], &[]);
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let mut paths = fs::read_dir(path)?
            .map(|entry| entry.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort();
        let mut qsl = Vec::new();
        let mut deq = Vec::new();
        for file in &paths {
            if !file.is_file() {
                continue;
            }
            match file.extension().and_then(|e| e.to_str()) {
                Some("qsl") => qsl.push(file.to_str().ok_or("QSL path is not UTF-8")?),
                Some("deq") => deq.push(file.to_str().ok_or("DEQ path is not UTF-8")?),
                _ => {}
            }
        }
        Self::from_files(&qsl, &deq)
    }
    /// Dispatch once during the build. The backend borrows shared data.
    pub fn use_backend<B: SchemaBackend>(&self, backend: &mut B) -> Result<B::Output, B::Error> {
        backend.compile(self)
    }
}

/// Collect schemas and configuration without selecting a generator.
pub fn build_qsl_schem(
    path: impl AsRef<Path>,
) -> Result<QslSchemaBuilder, Box<dyn std::error::Error>> {
    QslSchemaBuilder::scan(path)
}

fn load_inputs(
    qsl_files: &[&str],
    deq_files: &[&str],
    rust_files: &[&str],
) -> Result<QslSchemaBuilder, Box<dyn std::error::Error>> {
    let mut constants = HashMap::new();
    let mut generation = GenerationConfig::default();
    let mut configured = HashSet::new();
    for path in deq_files {
        println!("cargo:rerun-if-changed={path}");
        let text = fs::read_to_string(path)?;
        let parsed = parse_deq(&text).map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{path}: {err}"))
        })?;
        for key in &parsed.configured {
            if !configured.insert(key.clone()) {
                return Err(format!("DEQ config `{key}` is defined more than once").into());
            }
            match key.as_str() {
                "USE_HASH" => generation.use_hash = parsed.generation.use_hash,
                "MAX_DATA_SIZE" => generation.max_data_size = parsed.generation.max_data_size,
                "QSL_RT" => generation.qsl_rt = parsed.generation.qsl_rt,
                "QSL_RT_DEFAULT" => generation.qsl_rt_default = parsed.generation.qsl_rt_default,
                "STACK_LIMIT" => generation.stack_limit = parsed.generation.stack_limit,
                _ => unreachable!(),
            }
        }
        for (name, constant) in parsed.constants {
            if constants.insert(name.clone(), constant.value).is_some() {
                return Err(format!("DEQ constant `{name}` is defined more than once").into());
            }
        }
    }

    let mut schemas = Vec::new();
    let mut names = HashSet::new();
    for path in qsl_files {
        println!("cargo:rerun-if-changed={path}");
        let text = fs::read_to_string(path)?;
        let parsed = parse_qsl_raw_with_constants(&text, &constants).map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{path}: {err}"))
        })?;
        for schema in parsed {
            if !names.insert(schema.name.clone()) {
                return Err(
                    format!("QSL schema `{}` is defined more than once", schema.name).into(),
                );
            }
            schemas.push(schema);
        }
    }

    let mut rust_sources = Vec::new();
    for path in rust_files {
        println!("cargo:rerun-if-changed={path}");
        rust_sources.push(fs::read_to_string(path)?);
    }
    #[cfg(feature = "rust-macros")]
    let schemas = crate::rust_macro::expand_with_rust(
        &schemas,
        &rust_sources.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    #[cfg(not(feature = "rust-macros"))]
    let schemas = crate::schema_macro::expand_macros(&schemas)?;
    crate::analyze_schemas(&schemas, generation).map_err(Into::into)
}

impl Default for QslSchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}
