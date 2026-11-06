//! DEQ declares typed constants and explicit generation configuration.
//!
//! Implemented syntax: `!define NAME: usize = expression;`.

use std::collections::{HashMap, HashSet};

use crate::parser::{Cursor, ParseError};

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

pub use crate::config::GenerationConfig;

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
