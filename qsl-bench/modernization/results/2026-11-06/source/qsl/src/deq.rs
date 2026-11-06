//! DEQ declares typed constants and explicit generation configuration.
//!
//! Implemented syntax: `!define NAME: usize = expression;`.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::interpreter::QslInterpreter;
use crate::parser::{parse_qsl_raw_with_constants, Cursor, ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeqConstant {
    pub type_name: String,
    pub value: i128,
}

#[derive(Debug, Clone, Default)]
pub struct DeqConfig {
    pub constants: HashMap<String, DeqConstant>,
    pub generation: GenerationConfig,
    pub configured: HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationConfig {
    pub use_hash: crate::SchemaHashAlgorithm,
    pub max_data_size: usize,
    pub qsl_rt: bool,
    pub qsl_rt_default: usize,
    pub stack_limit: usize,
}
impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            use_hash: crate::SchemaHashAlgorithm::default(),
            max_data_size: crate::validation::DEFAULT_MAX_DATA_SIZE,
            qsl_rt: false,
            qsl_rt_default: 16,
            stack_limit: 256,
        }
    }
}

impl DeqConfig {
    /// QSL currently interprets numeric constant values at build time, while
    /// the DEQ AST retains the source type for validation and diagnostics.
    pub fn integer_values(&self) -> HashMap<String, i128> {
        self.constants
            .iter()
            .map(|(k, v)| (k.clone(), v.value))
            .collect()
    }
}

pub fn parse_deq(source: &str) -> Result<DeqConfig, ParseError> {
    let mut parser = Cursor::new(source);
    let mut result = DeqConfig::default();
    let mut values = HashMap::new();

    while !parser.eof()? {
        parser.expect("!")?;
        let directive = parser.ident()?;
        if directive == "config" {
            let name = parser.ident()?.to_owned();
            parser.expect("=")?;
            if !result.configured.insert(name.clone()) {
                return Err(parser.error("duplicate generation config"));
            }
            match name.as_str() {
                "USE_HASH" => {
                    result.generation.use_hash = parser
                        .ident()?
                        .parse()
                        .map_err(|_| parser.error("USE_HASH expects xxHash or xxHash128"))?;
                }
                "MAX_DATA_SIZE" => {
                    let n = parser.expr(&values)?;
                    if n <= 0 || n > isize::MAX as i128 {
                        return Err(parser.error("MAX_DATA_SIZE must be 1..isize::MAX bytes"));
                    }
                    result.generation.max_data_size = n as usize;
                }
                "QSL_RT" => {
                    result.generation.qsl_rt = match parser.ident()? {
                        "true" => true,
                        "false" => false,
                        _ => return Err(parser.error("QSL_RT expects true or false")),
                    }
                }
                "STACK_LIMIT" => {
                    let n = parser.expr(&values)?;
                    if n < 0 || n > u32::MAX as i128 {
                        return Err(parser.error("STACK_LIMIT must be 0..u32::MAX bytes"));
                    }
                    result.generation.stack_limit = n as usize;
                }
                "QSL_RT_DEFAULT" => {
                    let n = parser.expr(&values)?;
                    if !(1..=64).contains(&n) {
                        return Err(parser.error("QSL_RT_DEFAULT must be 1..64 slots"));
                    }
                    result.generation.qsl_rt_default = n as usize;
                }
                _ => return Err(parser.error("unknown generation configuration")),
            }
            parser.expect(";")?;
            continue;
        }
        if directive != "define" {
            return Err(parser.error(format!("unknown DEQ directive `!{directive}`")));
        }
        let name = parser.ident()?.to_owned();
        parser.expect(":")?;
        let type_name = parser.ident()?.to_owned();
        parser.expect("=")?;
        let value = parser.expr(&values)?;
        parser.expect(";")?;

        validate_constant(&type_name, value, &parser)?;
        if result
            .constants
            .insert(name.clone(), DeqConstant { type_name, value })
            .is_some()
        {
            return Err(parser.error(format!("duplicate DEQ constant `{name}`")));
        }
        values.insert(name, value);
    }
    Ok(result)
}

fn validate_constant(ty: &str, value: i128, parser: &Cursor<'_>) -> Result<(), ParseError> {
    let fits = match ty {
        "usize" => usize::try_from(value).is_ok(),
        "isize" => isize::try_from(value).is_ok(),
        "u8" => u8::try_from(value).is_ok(),
        "u16" => u16::try_from(value).is_ok(),
        "u32" => u32::try_from(value).is_ok(),
        "u64" => u64::try_from(value).is_ok(),
        "i8" => i8::try_from(value).is_ok(),
        "i16" => i16::try_from(value).is_ok(),
        "i32" => i32::try_from(value).is_ok(),
        "i64" => i64::try_from(value).is_ok(),
        _ => return Err(parser.error(format!("unsupported DEQ constant type `{ty}`"))),
    };
    if fits {
        Ok(())
    } else {
        Err(parser.error(format!("DEQ constant does not fit `{ty}`")))
    }
}

/// Explicit file paths belong in build.rs; DEQ provides values and opt-in settings.
/// Lists are traversed in caller order, preserving deterministic schema order.
pub fn build_qsl_files(
    qsl_files: &[&str],
    deq_files: &[&str],
    out_dir: impl AsRef<Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    build_inputs(qsl_files, deq_files, &[], out_dir.as_ref())
}
#[cfg(feature = "rust-macros")]
pub fn build_qsl_files_with_rust(
    qsl_files: &[&str],
    deq_files: &[&str],
    rust_files: &[&str],
    out_dir: impl AsRef<Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    build_inputs(qsl_files, deq_files, rust_files, out_dir.as_ref())
}
fn build_inputs(
    qsl_files: &[&str],
    deq_files: &[&str],
    rust_files: &[&str],
    out_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
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
    let mut interpreter = QslInterpreter::new();
    interpreter.config = generation;
    interpreter.generate(&schemas)?;
    let out_path = out_dir.join("qsl_gen.rs");
    fs::write(&out_path, interpreter.output)?;
    Ok(out_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generation_configs_are_explicit_and_bounded() {
        let c = parse_deq("!config QSL_RT = true; !config QSL_RT_DEFAULT = 16;").unwrap();
        assert!(c.generation.qsl_rt);
        assert_eq!(c.generation.qsl_rt_default, 16);
        for s in [
            "!config QSL_RT = 1;",
            "!config QSL_RT_DEFAULT = 0;",
            "!config QSL_RT_DEFAULT = 65;",
            "!config QSL_RT = true;!config QSL_RT = false;",
        ] {
            assert!(parse_deq(s).is_err());
        }
    }
    #[test]
    fn maximum_data_size_has_a_safe_default_and_checked_arithmetic() {
        assert_eq!(GenerationConfig::default().max_data_size, 65535);
        assert_eq!(
            parse_deq("!config MAX_DATA_SIZE = 1024 * 64 - 1;")
                .unwrap()
                .generation
                .max_data_size,
            65535
        );
        for source in [
            "!config MAX_DATA_SIZE = 0;",
            "!config MAX_DATA_SIZE = -1;",
            "!config MAX_DATA_SIZE = true;",
            "!config MAX_DATA_SIZE = 65535; !config MAX_DATA_SIZE = 8;",
        ] {
            assert!(parse_deq(source).is_err(), "{source}");
        }
    }
    #[test]
    fn arithmetic_precedence_is_shared_with_qsl() {
        let d = parse_deq("!define MAX: usize = 2 + sizeof(u64) * 16 / 2 % 50;").unwrap();
        assert_eq!(d.constants["MAX"].value, 16);
        for expr in [
            "1 / 0",
            "1 % 0",
            "170141183460469231731687303715884105727 * 2",
        ] {
            assert!(parse_deq(&format!("!define X: usize = {expr};")).is_err());
        }
    }

    #[test]
    fn typed_constants_and_dependencies() {
        let defs = parse_deq(
            "!define M: usize = sizeof(u64) + 2;\n\
            !define NEXT: u16 = !M + 1;",
        )
        .unwrap();
        assert_eq!(defs.constants["M"].type_name, "usize");
        assert_eq!(defs.constants["M"].value, 10);
        assert_eq!(defs.constants["NEXT"].value, 11);
    }

    #[test]
    fn rejects_untyped_or_build_directives() {
        assert!(parse_deq("source = \"foo.qsl\";").is_err());
        assert!(parse_deq("output = \"qsl_gen.rs\";").is_err());
        assert!(parse_deq("define M = 4;").is_err());
        assert!(parse_deq("!define M = 4;").is_err());
    }

    #[test]
    fn checks_types_and_duplicates() {
        assert!(parse_deq("!define X: u8 = 256;").is_err());
        assert!(parse_deq("!define X: usize = -1;").is_err());
        assert!(parse_deq("!define X: usize = 1; !define X: usize = 2;").is_err());
        assert!(parse_deq("!define X: usize = !NOPE;").is_err());
    }
}
