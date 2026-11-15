//! Build-time lowering of exact integer domains into scalar Rust expressions.
//! Only plain integer values are reconstructed; no user type is transmuted.
use super::*;
use qsl_lexer::domain::{low_mask, NumericDomain};
fn bits(d: &NumericDomain, value: &str) -> String {
    format!("({value})as u128&{}u128", low_mask(d.physical_bits()))
}
pub(super) fn copied(reference: &str) -> String {
    reference
        .strip_prefix('&')
        .map(str::to_owned)
        .unwrap_or_else(|| format!("*({reference})"))
}
pub(super) fn valid(d: &NumericDomain, value: &str) -> String {
    let value = bits(d, value);
    match d {
        NumericDomain::Interval(i) => {
            let delta = format!(
                "({value}).wrapping_sub({}u128)&{}u128",
                i.minimum_bits(),
                low_mask(i.physical_bits())
            );
            let span =
                i.maximum_bits().wrapping_sub(i.minimum_bits()) & low_mask(i.physical_bits());
            if i.maximum_rank() == 0 {
                return format!("({value}=={}u128)", i.minimum_bits());
            }
            if i.step() == 1 && span == low_mask(i.physical_bits()) {
                return "true".into();
            }
            if i.step() == 1 {
                format!("(({delta})<={span}u128)")
            } else {
                format!(
                    "((({delta})<={span}u128)&(({delta}).is_multiple_of({}u128)))",
                    i.step()
                )
            }
        }
        NumericDomain::Set(s) => format!(
            "matches!({value},{})",
            s.values()
                .iter()
                .map(|n| format!("{n}u128"))
                .collect::<Vec<_>>()
                .join("|")
        ),
    }
}
pub(super) fn encode(d: &NumericDomain, value: &str) -> String {
    if d.encoded_bits() == 0 {
        return "0u128".into();
    }
    let value = bits(d, value);
    match d {
        NumericDomain::Interval(i) => format!(
            "(({value}).wrapping_sub({}u128)&{}u128)/{}u128",
            i.minimum_bits(),
            low_mask(i.physical_bits()),
            i.step()
        ),
        NumericDomain::Set(s) => format!(
            "unsafe{{match {value}{{{},_=>std::hint::unreachable_unchecked()}}}}",
            s.values()
                .iter()
                .enumerate()
                .map(|(rank, value)| format!("{value}u128=>{rank}u128"))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
/// Caller has established rank<=maximum_rank, or is under the unchecked contract.
pub(super) fn decode(d: &NumericDomain, rank: &str) -> String {
    let value = match d {
        NumericDomain::Interval(i) => format!(
            "({rank}).wrapping_mul({}u128).wrapping_add({}u128)&{}u128",
            i.step(),
            i.minimum_bits(),
            low_mask(i.physical_bits())
        ),
        NumericDomain::Set(s) => format!(
            "unsafe{{match {rank}{{{},_=>std::hint::unreachable_unchecked()}}}}",
            s.values()
                .iter()
                .enumerate()
                .map(|(rank, value)| format!("{rank}u128=>{value}u128"))
                .collect::<Vec<_>>()
                .join(",")
        ),
    };
    format!("({value})as {}", d.primitive())
}
pub(super) fn check_value(d: &NumericDomain, out: &mut String, value: &str) {
    let condition = valid(d, value);
    if condition != "true" {
        ln(
            out,
            format!("if !{condition}{{return Err(::qsl::QslError::InvalidValue);}}"),
        );
    }
}
pub(super) fn checked(d: &NumericDomain, out: &mut String, compact: bool) -> QslResult<()> {
    ln(out, format!("c.charge({})?;", d.physical_bits() / 8));
    if compact {
        let width = d.encoded_bits();
        if width == 0 {
            ln(out, "let __qsl_rank=0u128;");
        } else if width <= 64 {
            ln(out, format!("let __qsl_rank=c.bits({width})?as u128;"));
        } else {
            ln(out,format!("let __qsl_low=c.bits(64)?as u128;let __qsl_high=c.bits({})?as u128;let __qsl_rank=__qsl_low|(__qsl_high<<64);",width-64));
        }

        if d.maximum_rank() < low_mask(d.encoded_bits()) {
            ln(
                out,
                format!(
                    "if __qsl_rank>{}u128{{return Err(::qsl::QslError::InvalidValue);}}",
                    d.maximum_rank()
                ),
            );
        }
        ln(out, format!("let value={};", decode(d, "__qsl_rank")));
    } else {
        ln(
            out,
            format!(
                "let value=c.uint128({})?as {};",
                d.physical_bits() / 8,
                d.primitive()
            ),
        );
        check_value(d, out, "value");
    }
    ln(out, "let _=value;");
    Ok(())
}
