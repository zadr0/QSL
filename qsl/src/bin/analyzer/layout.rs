//! Editor-only wire bounds. Never infer repr(Rust) object layout from wire order.
use super::json;
use qsl::{QslSchema, QslType};
fn number(n: Option<usize>) -> String {
    n.map_or("null".into(), |n| n.to_string())
}
fn add(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    a?.checked_add(b?)
}
fn bounds(t: &QslType, raw: bool) -> (Option<usize>, Option<usize>) {
    if let QslType::Array(e, min)
    | QslType::RangedArray {
        element: e, min, ..
    } = t
    {
        let unit = e.fixed_wire_size().ok().flatten();
        let low = unit
            .and_then(|n| n.checked_mul(*min))
            .and_then(|n| n.checked_add(4))
            .and_then(|n| n.checked_mul(8));
        let max = if let QslType::RangedArray { max, .. } = t {
            Some(*max)
        } else {
            None
        };
        let high = unit
            .zip(max)
            .and_then(|(n, m)| n.checked_mul(m))
            .and_then(|n| n.checked_add(4))
            .and_then(|n| n.checked_mul(8));
        return (low, high);
    }
    let string = match t {
        QslType::String(min) => (*min, None, 4),
        QslType::RangedString { min, max } => (*min, Some(*max), 4),
        QslType::BorrowedString { min, max } => (*min, *max, 4),
        QslType::InlineRawString { min, max } => (
            *min,
            Some(*max),
            if *max <= 127 {
                1
            } else if *max <= 32767 {
                2
            } else {
                4
            },
        ),
        QslType::CompressedString { .. } | QslType::FixedCompressedString(_) => {
            return (
                None,
                qsl::interpreter::schema_maximum_size(&QslSchema {
                    name: "EditorField".into(),
                    fields: vec![qsl::QslField {
                        storage: Default::default(),
                        name: "value".into(),
                        data: t.clone(),
                        tag: 1,
                        ..Default::default()
                    }],
                    max_tag: 1,
                    ..Default::default()
                })
                .ok()
                .flatten()
                .and_then(|n| n.checked_mul(8)),
            )
        }
        _ => {
            let size = t
                .fixed_wire_size()
                .ok()
                .flatten()
                .and_then(|n| n.checked_mul(8));
            return (size, size);
        }
    };
    let header = if raw { 0 } else { string.2 };
    (
        (string.0.checked_add(header)).and_then(|n| n.checked_mul(8)),
        string
            .1
            .and_then(|n| n.checked_add(header))
            .and_then(|n| n.checked_mul(8)),
    )
}
fn wire_fields(s: &QslSchema) -> Vec<&qsl::QslField> {
    if s.bitmap() {
        s.fields
            .iter()
            .filter(|f| !f.optional)
            .chain(s.fields.iter().rev().filter(|f| f.optional))
            .collect()
    } else {
        s.fields.iter().collect()
    }
}
pub(super) fn describe(s: &QslSchema) -> String {
    if let Ok(Some(p)) = qsl::hybrid::HybridPlan::analyze(s) {
        return hybrid(s, &p);
    }
    let quant = qsl::payload_quant::PayloadQuantPlan::analyze(s).ok();
    let packed = quant.as_ref().is_some_and(|q| q.active);
    let graph = if s.predicted_opt_table() && !packed {
        qsl::static_optional::StaticOptionalPlan::analyze(s).ok()
    } else {
        None
    };
    let n = s.optional_count();
    let (hmin, hmax) = if packed {
        let h = quant.as_ref().unwrap().header_bytes * 8;
        (h, h)
    } else if let Some(g) = &graph {
        (g.header_bytes() * 8, g.header_bytes() * 8)
    } else if s.state_bits() {
        (32, 32)
    } else if s.predict_seq() {
        (0, 0)
    } else if s.legacy_opt_table() {
        ((1 + 4 * n) * 8, (1 + 4 * n) * 8)
    } else if s.bitmap() {
        (s.bitmap_width(), s.bitmap_width())
    } else if n == 1 {
        (1, 1)
    } else if n > 1 {
        (s.count_bits(), s.count_bits() + n * s.tag_bits())
    } else {
        (0, 0)
    };
    let byte_aligned = !packed && !s.compact();
    let align = |n: usize| if byte_aligned { n.div_ceil(8) * 8 } else { n };
    let (mut low, mut high) = (Some(align(hmin)), Some(align(hmax)));
    let raw = matches!(
        graph.as_ref().map(|g| &g.presence),
        Some(qsl::static_optional::StaticPresence::Graph(_))
    );
    let mut rows = Vec::new();
    let mut prior = Vec::new();
    for (i, f) in wire_fields(s).into_iter().enumerate() {
        let (min, max) = if packed {
            let width = quant.as_ref().unwrap().fields[i].bits;
            (Some(width), Some(width))
        } else if s.compact() {
            match f.data {
                QslType::Byte
                | QslType::Integer { .. }
                | QslType::RangedInteger { .. }
                | QslType::Flags(_) => {
                    let w = f.data.rank_width(f.bits).ok();
                    (w, w)
                }
                _ => (None, None),
            }
        } else {
            bounds(&f.data, f.optional && (raw || s.state_bits()))
        };
        let ty = qsl::modernization::field_rust_type(f, s.borrow_strings())
            .unwrap_or_else(|_| "?".into());
        let ty = if f.optional {
            format!("Option<{ty}>")
        } else {
            ty
        };
        let expression = if prior.is_empty() {
            "H".into()
        } else {
            format!("H + {}", prior.join(" + "))
        };
        rows.push(format!("{{\"name\":{},\"rustType\":{},\"optional\":{},\"tag\":{},\"minBits\":{},\"maxBits\":{},\"offsetMinBits\":{},\"offsetMaxBits\":{},\"offsetExpr\":{}}}",json(&f.name),json(&ty),f.optional,f.tag,number(min),number(max),number(low),number(high),json(&expression)));
        low = add(low, if f.optional { Some(0) } else { min });
        high = add(high, max);
        prior.push(if f.optional {
            format!("present({0}) * size({0})", f.name)
        } else {
            format!("size({})", f.name)
        });
    }
    format!("{{\"unit\":{},\"headerMinBits\":{},\"headerMaxBits\":{},\"minimumBytes\":{},\"nativeSize\":null,\"fields\":[{}]}}",json(if packed||s.compact(){"bit"}else{"byte"}),align(hmin),align(hmax),number(low.map(|n|n.div_ceil(8))),rows.join(","))
}

fn hybrid(s: &QslSchema, p: &qsl::hybrid::HybridPlan) -> String {
    let q = qsl::payload_quant::PayloadQuantPlan::analyze(&p.prefix).unwrap();
    let h = (p.length_header + q.header_bytes) * 8;
    let (mut low, mut high) = (Some(h), Some(h));
    let mut rows = Vec::new();
    let row = |f: &qsl::QslField,
               min: Option<usize>,
               max: Option<usize>,
               low: Option<usize>,
               high: Option<usize>,
               expr: &str| {
        let ty = qsl::modernization::field_rust_type(f, s.borrow_strings()).unwrap_or("?".into());
        let ty = if f.optional {
            format!("Option<{ty}>")
        } else {
            ty
        };
        format!("{{\"name\":{},\"rustType\":{},\"optional\":{},\"tag\":{},\"minBits\":{},\"maxBits\":{},\"offsetMinBits\":{},\"offsetMaxBits\":{},\"offsetExpr\":{}}}",json(&f.name),json(&ty),f.optional,f.tag,number(min),number(max),number(low),number(high),json(expr))
    };
    for (f, qf) in p.prefix.fields.iter().zip(&q.fields) {
        rows.push(row(
            f,
            Some(qf.bits),
            Some(qf.bits),
            low,
            high,
            "QPS prefix presence + prior scalar bits",
        ));
        if !f.optional {
            low = add(low, Some(qf.bits))
        }
        high = add(high, Some(qf.bits));
    }
    let tail = &p.tail;
    let optional = tail.optional_count();
    let graph = if tail.predicted_opt_table() {
        qsl::static_optional::StaticOptionalPlan::analyze(tail).ok()
    } else {
        None
    };
    let (hmin, hmax) = if let Some(g) = &graph {
        (g.header_bytes() * 8, g.header_bytes() * 8)
    } else if tail.state_bits() {
        (32, 32)
    } else if tail.predict_seq() {
        (0, 0)
    } else if tail.legacy_opt_table() {
        ((1 + 4 * optional) * 8, (1 + 4 * optional) * 8)
    } else if tail.bitmap() {
        (tail.bitmap_width(), tail.bitmap_width())
    } else if optional == 1 {
        (8, 8)
    } else if optional > 1 {
        (
            tail.count_bits().div_ceil(8) * 8,
            (tail.count_bits() + optional * tail.tag_bits()).div_ceil(8) * 8,
        )
    } else {
        (0, 0)
    };
    low = Some((p.length_header + p.prefix_minimum) * 8 + hmin);
    high = Some((p.length_header + p.prefix_bytes) * 8 + hmax);
    let raw = matches!(
        graph.as_ref().map(|g| &g.presence),
        Some(qsl::static_optional::StaticPresence::Graph(_))
    );
    for f in wire_fields(tail) {
        let (min, max) = bounds(&f.data, f.optional && (raw || tail.state_bits()));
        rows.push(row(
            f,
            min,
            max,
            low,
            high,
            "native prefix length + tail metadata + prior tail fields",
        ));
        low = add(low, if f.optional { Some(0) } else { min });
        high = add(high, max);
    }
    format!("{{\"unit\":\"bit\",\"headerMinBits\":{h},\"headerMaxBits\":{h},\"minimumBytes\":{},\"nativeSize\":null,\"scalarPrefix\":true,\"fields\":[{}]}}",number(low.map(|n|n.div_ceil(8))),rows.join(","))
}
