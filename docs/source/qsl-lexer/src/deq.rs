//! DEQ declares typed constants and explicit generation configuration.
//!
//! Implemented syntax: `!define NAME: usize = expression;`.

use std::collections::{HashMap, HashSet};

use crate::parser::{Cursor, ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeqConstant {
    pub type_name: String,
    pub value: crate::WideNumber,
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
    pub fn integer_values(&self) -> Result<HashMap<String, i128>, ParseError> {
        self.constants.iter().map(|(k,v)|v.value.signed().map(|n|(k.clone(),n)).ok_or_else(||ParseError{offset:0,message:format!("constant `{k}` does not fit i128; use wide_values and parse_qsl_with_wide_constants")})).collect()
    }
    pub fn wide_values(&self) -> HashMap<String, crate::WideNumber> {
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
                "CPU_PROFILE" => {
                    result.generation.cpu_profile = match parser.ident()? {
                        "portable" => crate::optimizer::registers::CpuProfile::Portable,
                        "kaby_lake" => crate::optimizer::registers::CpuProfile::KabyLake,
                        _ => return Err(parser.error("CPU_PROFILE expects portable or kaby_lake")),
                    };
                }
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
        let value = parser.wide_expr(&values)?;
        parser.expect(";")?;

        validate_constant(&type_name, value, &parser)?;
        if result
            .constants
            .insert(name.clone(), DeqConstant { type_name, value })
            .is_some()
        {
            return Err(parser.error(format!("duplicate DEQ constant `{name}`")));
        }
        parser.wide_constants.insert(name.clone(), value);
        if let Some(n) = value.signed() {
            values.insert(name, n);
        }
    }
    Ok(result)
}

fn validate_constant(
    ty: &str,
    value: crate::WideNumber,
    parser: &Cursor<'_>,
) -> Result<(), ParseError> {
    let (bits, signed) = match ty {
        "usize" => (usize::BITS as u16, false),
        "isize" => (isize::BITS as u16, true),
        _ => {
            let bytes = ty.as_bytes();
            if bytes.len() < 2 || !matches!(bytes[0], b'u' | b'i') {
                return Err(parser.error(format!("unsupported DEQ constant type `{ty}`")));
            }
            let bits = ty[1..]
                .parse::<u16>()
                .map_err(|_| parser.error("unsupported DEQ constant type"))?;
            if !matches!(bits, 8 | 16 | 32 | 64 | 128) {
                return Err(parser.error("unsupported DEQ constant type"));
            }
            (bits, bytes[0] == b'i')
        }
    };
    if value.native(bits, signed).is_some() {
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

#[cfg(test)]
mod wide_tests {
    use super::*;
    #[test]
    fn full_deq_range_is_exact_and_narrow_projection_is_fallible() {
        let d=parse_deq("!define TOP:u128=340282366920938463463374607431768211455;!define LOW:i128=-170141183460469231731687303715884105728;!define LAST:u128=!TOP-1;!define SMALL:u8=!TOP-!LAST;").unwrap();
        assert_eq!(d.constants["TOP"].value.unsigned(), Some(u128::MAX));
        assert_eq!(d.constants["LOW"].value.signed(), Some(i128::MIN));
        assert_eq!(d.constants["SMALL"].value, 1);
        assert!(d.integer_values().is_err());
        let s = crate::parse_qsl_with_wide_constants(
            "&P[compact,safety]{u128(0..!TOP) x|default=!LAST|;i128(!LOW..0) y;u8(0..!SMALL) z;}",
            &d.wide_values(),
        )
        .unwrap();
        assert_eq!(s[0].fields[0].default, Some(-2));
        for src in [
            "!define X:u128=340282366920938463463374607431768211455+1;",
            "!define X:i128=170141183460469231731687303715884105728;",
            "!define X:u128=-1;",
        ] {
            assert!(parse_deq(src).is_err(), "{src}");
        }
    }
}

#[cfg(test)]
mod cpu_profile_tests {
    use super::*;
    #[test]
    fn explicit_cpu_profiles_do_not_depend_on_the_analysis_host() {
        assert_eq!(
            parse_deq("!config CPU_PROFILE=kaby_lake;")
                .unwrap()
                .generation
                .cpu_profile,
            crate::optimizer::registers::CpuProfile::KabyLake
        );
        assert_eq!(
            parse_deq("!config CPU_PROFILE=portable;")
                .unwrap()
                .generation
                .cpu_profile,
            crate::optimizer::registers::CpuProfile::Portable
        );
        assert!(parse_deq("!config CPU_PROFILE=host;").is_err());
    }
}
