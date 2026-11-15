//! Schema collection and backend contracts. No code generator is required.
pub use crate::frontend::QslSchemaBuilder;
use crate::{deq::parse_deq, parser::parse_qsl_raw_with_wide_primitives};
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
            declaration: Default::default(),
            config: GenerationConfig::default(),
            primitives: Default::default(),
        }
    }
    /// Register an explicit transparent primitive before collecting source.
    pub fn with_primitive<T: crate::QslPrimitive>(mut self, name: &str) -> QslResult<Self> {
        let p = T::REPRESENTATION;
        p.validate()?;
        let n = p.bits as usize / 8;
        let align = crate::layout::native_type_layout(
            &crate::types::primitive_type(p),
            &Default::default(),
        )
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
                "CPU_PROFILE" => generation.cpu_profile = parsed.generation.cpu_profile,
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
        let parsed = parse_qsl_raw_with_wide_primitives(&text, &constants, &Default::default())
            .map_err(|err| {
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

/// Compatibility spelling from the declaration API; this is the existing builder.
pub type QslSchemBuilder = QslSchemaBuilder;
pub use crate::builders::{DeqBuilder, QslFieldType, QslMacro};

#[derive(Clone, Debug, Default)]
pub(crate) struct DeclarationState {
    selected: Option<usize>,
    field: Option<String>,
    error: Option<crate::QslError>,
}

/// Configure and collect schemas before selecting a backend.
#[derive(Clone, Debug, Default)]
pub struct QslBuilder {
    schemas: QslSchemaBuilder,
    auto: bool,
}
impl QslBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn config(&mut self, configure: impl FnOnce(&mut DeqBuilder)) -> &mut Self {
        let mut deq = DeqBuilder::from_config(self.schemas.config);
        configure(&mut deq);
        match deq.build() {
            Ok(c) => self.schemas.config = c,
            Err(e) => self.schemas.declaration.error = Some(e),
        }
        self
    }
    pub fn schem(&mut self, declare: impl FnOnce(&mut QslSchemaBuilder)) -> &mut Self {
        let mut one = QslSchemaBuilder::new();
        one.config = self.schemas.config;
        declare(&mut one);
        match one.finish() {
            Ok(mut one) => self.schemas.schemas.append(&mut one.schemas),
            Err(e) => self.schemas.declaration.error = Some(e),
        }
        self
    }
    /// Permit physical layout changes only; the wire order remains pinned.
    pub fn auto(&mut self) -> &mut Self {
        self.auto = true;
        self
    }
    pub fn build(&self) -> QslResult<QslSchemaBuilder> {
        let mut out = self.schemas.clone();
        if self.auto {
            for schema in &mut out.schemas {
                schema.options.auto_layout = true;
            }
        }
        out.finish()
    }
}

impl QslSchemaBuilder {
    /// Begin a named declaration in this same schema collection.
    pub fn name(&mut self, name: &str) -> &mut Self {
        if self.declaration.field.is_some() {
            self.declaration.error = Some(crate::QslError::InvalidSchema);
        }
        self.schemas.push(QslSchema {
            name: name.into(),
            ..Default::default()
        });
        self.declaration.selected = Some(self.schemas.len() - 1);
        self
    }
    /// Name the next field; consuming a type declaration commits it.
    pub fn field(&mut self, name: &str) -> &mut Self {
        if self.declaration.field.is_some() {
            self.declaration.error = Some(crate::QslError::InvalidSchema);
        }
        self.declaration.field = Some(name.into());
        self
    }
    fn declared(&mut self) -> Option<&mut QslSchema> {
        if let Some(i) = self.declaration.selected {
            self.schemas.get_mut(i)
        } else {
            self.declaration.error = Some(crate::QslError::InvalidSchema);
            None
        }
    }
    fn declare_flag(&mut self, flag: usize) -> &mut Self {
        if let Some(s) = self.declared() {
            s.flags |= flag;
        }
        self
    }
    pub fn compact(&mut self) -> &mut Self {
        self.declare_flag(crate::FLAG_COMPACT)
    }
    pub fn safety(&mut self) -> &mut Self {
        self.declare_flag(crate::FLAG_SAFETY)
    }
    pub fn alignment(&mut self) -> &mut Self {
        self.declare_flag(crate::FLAG_ALIGNMENT)
    }
    pub fn inline(&mut self) -> &mut Self {
        self.declare_flag(crate::FLAG_INLINE)
    }
    pub fn lifetime(&mut self) -> &mut Self {
        self.declare_flag(crate::FLAG_LIFETIME)
    }
    pub fn ot(&mut self) -> &mut Self {
        self.declare_flag(crate::FLAG_OPT_TABLE)
    }
    pub fn ps(&mut self) -> &mut Self {
        self.declare_flag(crate::FLAG_PREDICT_SEQ)
    }
    pub fn bm(&mut self) -> &mut Self {
        self.declare_flag(crate::FLAG_OPTIONAL_BITMAP)
    }
    pub fn q16(&mut self) -> &mut Self {
        self.declare_flag(crate::FLAG_QPS | (3 << crate::PAYLOAD_QUANT_SHIFT))
    }
    pub fn dyn_line(&mut self, width: u32, revision: u32) -> &mut Self {
        self.dyn_mode(qsl_core::dynamic::DynMode::Line, width, revision)
    }
    pub fn dyn_index(&mut self, width: u32, strict: bool, revision: u32) -> &mut Self {
        self.dyn_mode(
            if strict {
                qsl_core::dynamic::DynMode::StrictIndex
            } else {
                qsl_core::dynamic::DynMode::DynamicIndex
            },
            width,
            revision,
        )
    }
    fn dyn_mode(
        &mut self,
        mode: qsl_core::dynamic::DynMode,
        width: u32,
        revision: u32,
    ) -> &mut Self {
        let width = match width {
            16 => qsl_core::dynamic::DynWidth::W16,
            32 => qsl_core::dynamic::DynWidth::W32,
            64 => qsl_core::dynamic::DynWidth::W64,
            _ => {
                self.declaration.error = Some(crate::QslError::InvalidWidth);
                return self;
            }
        };
        if let Some(s) = self.declared() {
            s.options.dynamic = Some(qsl_core::dynamic::DynHeader {
                mode,
                width,
                revision,
            });
        }
        self
    }
    /// Set a stable dyn tag on the just-committed field.
    pub fn tag(&mut self, tag: u64) -> &mut Self {
        let Ok(tag) = usize::try_from(tag) else {
            self.declaration.error = Some(crate::QslError::InvalidValue);
            return self;
        };
        if let Some(s) = self.declared() {
            if s.options.dynamic.is_none() || s.fields.is_empty() {
                self.declaration.error = Some(crate::QslError::InvalidSchema);
            } else {
                s.fields.last_mut().unwrap().tag = tag;
                s.max_tag = s.fields.iter().map(|f| f.tag).max().unwrap_or(0);
            }
        }
        self
    }
    pub fn transmutate(&mut self) -> &mut Self {
        if let Some(s) = self.declared() {
            s.options.transmutate = true;
        }
        self
    }
    pub fn loyal_cache(&mut self) -> &mut Self {
        if let Some(s) = self.declared() {
            s.options.loyal_cache = true;
        }
        self
    }
    pub fn cache_size(&mut self, bytes: u32) -> &mut Self {
        if let Some(s) = self.declared() {
            s.options.cache_limit = Some(bytes as usize);
        }
        self
    }
    pub fn auto(&mut self) -> &mut Self {
        if let Some(s) = self.declared() {
            s.options.auto_layout = true;
        }
        self
    }
    /// Enable physical layout selection and an explicit wire canonicalization version.
    pub fn auto_wire(&mut self, version: u16) -> &mut Self {
        if let Some(s) = self.declared() {
            s.options.auto_layout = true;
            s.options.auto_wire = Some(version);
        }
        self
    }
    /// Spelling retained from the requested builder contract.
    pub fn aggresive(&mut self, shift: bool, bitmap: bool) -> &mut Self {
        if let Some(s) = self.declared() {
            s.options.aggressive = true;
            s.options.aggressive_shift = Some(shift);
            s.options.aggressive_bitmap = bitmap;
        }
        self
    }
    pub fn pointing(&mut self, enabled: bool) -> &mut Self {
        if let Some(s) = self.declared() {
            s.options.aggressive_pointing = enabled;
        }
        self
    }
    pub fn group(&mut self, enabled: bool) -> &mut Self {
        if let Some(s) = self.declared() {
            s.options.aggressive_group = enabled;
        }
        self
    }
    fn push_field(
        &mut self,
        data: crate::QslType,
        default: Option<i128>,
        optional: bool,
    ) -> &mut Self {
        let Some(name) = self.declaration.field.take() else {
            self.declaration.error = Some(crate::QslError::InvalidSchema);
            return self;
        };
        let bits = match data.default_compact_bits() {
            Ok(n) => n,
            Err(e) => {
                self.declaration.error = Some(e);
                return self;
            }
        };
        if let Some(s) = self.declared() {
            let tag = s.fields.len() + 1;
            s.max_tag = tag;
            s.fields.push(crate::QslField {
                shifted_length: false,
                name,
                data,
                bits,
                optional,
                default,
                tag,
                storage: Default::default(),
            });
        }
        self
    }
    /// Declare an exact domain. default_bits is a native-width bit pattern.
    pub fn domain(
        &mut self,
        domain: impl Into<crate::domain::NumericDomain>,
        default_bits: Option<u128>,
        optional: bool,
    ) -> &mut Self {
        self.push_field(
            crate::QslType::DomainInteger(domain.into()),
            default_bits.map(|n| n as i128),
            optional,
        )
    }
    pub fn u128(&mut self, start: u128, end: u128, default: u128, optional: bool) -> &mut Self {
        match crate::domain::IntegerDomain::unsigned(128, start, end) {
            Ok(d) => {
                self.domain(d, Some(default), optional);
            }
            Err(e) => self.declaration.error = Some(e),
        }
        self
    }
    pub fn i128(&mut self, start: i128, end: i128, default: i128, optional: bool) -> &mut Self {
        match crate::domain::IntegerDomain::signed(128, start, end) {
            Ok(d) => {
                self.domain(d, Some(default as u128), optional);
            }
            Err(e) => self.declaration.error = Some(e),
        }
        self
    }
    pub fn str(&mut self, start: u32, end: u32, decompress: bool, optional: bool) -> &mut Self {
        self.push_field(
            if start == end {
                if decompress {
                    crate::QslType::FixedCompressedString(end as usize)
                } else {
                    crate::QslType::FixedString(end as usize)
                }
            } else if decompress {
                crate::QslType::CompressedString {
                    min: start as usize,
                    max: end as usize,
                }
            } else {
                crate::QslType::RangedString {
                    min: start as usize,
                    max: end as usize,
                }
            },
            None,
            optional,
        )
    }
    pub fn fstr(&mut self, size: u32, optional: bool) -> &mut Self {
        self.push_field(crate::QslType::FixedString(size as usize), None, optional)
    }
    pub fn flags(&mut self, bits: u32, optional: bool) -> &mut Self {
        self.push_field(crate::QslType::Flags(bits as usize), None, optional)
    }
    pub fn array<T: QslFieldType>(&mut self, start: u32, end: u32, optional: bool) -> &mut Self {
        if T::OPTIONAL || !T::VALID {
            self.declaration.error = Some(crate::QslError::InvalidSchema);
        }
        self.push_field(
            crate::QslType::RangedArray {
                element: Box::new(T::qsl_type()),
                min: start as usize,
                max: end as usize,
            },
            None,
            optional,
        )
    }
    pub fn farray<T: QslFieldType>(&mut self, size: u32, optional: bool) -> &mut Self {
        if T::OPTIONAL || !T::VALID {
            self.declaration.error = Some(crate::QslError::InvalidSchema);
        }
        self.push_field(
            crate::QslType::FixedArray(Box::new(T::qsl_type()), size as usize),
            None,
            optional,
        )
    }
    /// Padding is expressed in bits; a non-byte count requires COMPACT.
    pub fn padding(&mut self, bits: u32) -> &mut Self {
        self.push_field(
            if bits.is_multiple_of(8) {
                crate::QslType::Padding(bits as usize / 8)
            } else {
                crate::QslType::BitPadding(bits as usize)
            },
            None,
            false,
        )
    }
    pub fn r#macro<T: QslMacro>(&mut self) -> &mut Self {
        let prefix = self.declaration.field.take();
        match T::qsl_fields() {
            Ok(fields) => {
                if let Some(s) = self.declared() {
                    for mut f in fields {
                        if let Some(p) = &prefix {
                            f.name = format!("{p}_{}", f.name);
                        }
                        f.tag = s.fields.len() + 1;
                        s.max_tag = f.tag;
                        s.fields.push(f);
                    }
                }
            }
            Err(e) => self.declaration.error = Some(e),
        }
        self
    }
    pub fn validate_declarations(&self) -> QslResult<()> {
        if let Some(e) = self.declaration.error {
            return Err(e);
        }
        if self.declaration.field.is_some() {
            return Err(crate::QslError::InvalidSchema);
        }
        let mut names = std::collections::HashSet::new();
        for s in &self.schemas {
            if !names.insert(&s.name) {
                return Err(crate::QslError::InvalidSchema);
            }
            s.validate()?;
        }
        Ok(())
    }
    /// Validate and freeze declarations for a backend; no code is emitted here.
    pub fn finish(&self) -> QslResult<Self> {
        self.validate_declarations()?;
        let mut out = Self::from_schemas(&self.schemas, self.config)?;
        out.primitives = self.primitives.clone();
        Ok(out)
    }
}
macro_rules! numeric_declaration { ($($name:ident:$t:ty,$bits:expr,$signed:expr);* $(;)?) => { $(
    impl QslSchemaBuilder {pub fn $name(&mut self,start:$t,end:$t,default:$t,optional:bool)->&mut Self {
        self.push_field(crate::QslType::RangedInteger{bits:$bits,signed:$signed,min:start as i128,max:end as i128},Some(default as i128),optional)
    }}
)* }; }
numeric_declaration!(u8:u8,8,false;u16:u16,16,false;u32:u32,32,false;u64:u64,64,false;i8:i8,8,true;i16:i16,16,true;i32:i32,32,true;i64:i64,64,true;);

#[cfg(test)]
mod declarations_tests {
    use super::*;
    #[test]
    fn declaration_and_text_have_identical_wire_identity() {
        let built = QslBuilder::new()
            .config(|c| {
                c.max_data_size(65535)
                    .stack_limit(128)
                    .default_rt_slots(16)
                    .use_xxHash128();
            })
            .schem(|s| {
                s.name("Packet").safety().ot().ps();
                s.field("id").u64(1, 100, 1, false);
                s.field("text").str(1, 64, false, false);
                s.field("extra").u8(0, 15, 2, true);
            })
            .build()
            .unwrap();
        let text=QslSchemaBuilder::from_source_with_config("&Packet[safety,opt_table,predict_seq]{u64(1..100) id|default=1|;str(1..64) text;u8?(0..15) extra|default=2|;}",built.config()).unwrap();
        assert_eq!(
            built.schemas()[0].hashes(built.config().use_hash).unwrap(),
            text.schemas()[0].hashes(text.config().use_hash).unwrap()
        );
        assert_eq!(built.config().stack_limit, 128);
        assert_eq!(built.config().max_data_size, 65535);
    }
    #[test]
    fn errors_are_not_silently_discarded() {
        assert!(QslBuilder::new()
            .schem(|s| {
                s.name("P").field("bad").u8(3, 1, 2, false);
            })
            .build()
            .is_err());
        assert!(QslBuilder::new()
            .schem(|s| {
                s.name("P").field("x").u8(0, 5, 6, false);
            })
            .build()
            .is_err());
        assert!(QslBuilder::new()
            .schem(|s| {
                s.name("P")
                    .field("a")
                    .u8(0, 5, 1, true)
                    .field("b")
                    .u8(0, 5, 1, false);
            })
            .build()
            .is_err());
        assert!(QslBuilder::new()
            .schem(|s| {
                s.name("P").field("missing");
            })
            .build()
            .is_err());
        assert!(QslBuilder::new()
            .config(|d| {
                d.default_rt_slots(0);
            })
            .build()
            .is_err());
        assert!(QslBuilder::new()
            .schem(|s| {
                s.name("P");
            })
            .schem(|s| {
                s.name("P");
            })
            .build()
            .is_err());
    }
}

#[cfg(test)]
mod fixed_declaration_parity_tests {
    use super::*;
    #[test]
    fn fixed_codecs_have_identical_textual_and_builder_hashes() {
        let mut b = QslSchemaBuilder::new();
        b.name("P")
            .compact()
            .field("tag")
            .fstr(3, false)
            .field("values")
            .farray::<u16>(3, false);
        let b = b.finish().unwrap();
        let text =
            QslSchemaBuilder::from_source("&P[compact]{str(3) tag;array<u16>(3) values;}").unwrap();
        assert_eq!(
            b.schemas()[0].hashes(Default::default()).unwrap(),
            text.schemas()[0].hashes(Default::default()).unwrap()
        );
    }
}
