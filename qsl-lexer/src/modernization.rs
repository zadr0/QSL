//! Build-time storage, register and native-layout analysis.
use crate::{QslError, QslResult, QslSchema, QslType};

#[derive(Debug, Clone, Default)]
pub struct SchemaOptions {
    pub aggressive: bool,
    pub transmutate: bool,
    pub(crate) native_overlay: bool,
    pub loyal_cache: bool,
    pub slots: Option<usize>,
    pub cache_limit: Option<usize>,
    pub regions: Vec<NativeRegion>,
}

impl SchemaOptions {
    #[doc(hidden)]
    pub fn with_native_overlay(mut self) -> Self {
        self.native_overlay = true;
        self
    }
    /// Whether preparation selected a borrowed native overlay.
    pub fn native_overlay(&self) -> bool {
        self.native_overlay
    }
}

/// Smallest Rust word containing a compile-time byte window (zero uses u8).
/// This function runs in the generator, never in a generated decoder.
#[doc(hidden)]
pub fn register_word(bytes: usize) -> QslResult<&'static str> {
    Ok(match bytes {
        0..=1 => "u8",
        2 => "u16",
        3..=4 => "u32",
        5..=8 => "u64",
        9..=16 => "u128",
        _ => return Err(QslError::InvalidWidth),
    })
}

#[derive(Clone, Debug)]
pub struct RegisterGroup {
    pub first: usize,
    pub fields: usize,
    pub offset: Option<usize>,
    pub bytes: usize,
    pub register_bits: usize,
    pub metadata_bytes: usize,
}
pub fn register_groups(s: &QslSchema) -> QslResult<Vec<RegisterGroup>> {
    let mut out = Vec::new();
    if s.compact() || s.qps() {
        return Ok(out);
    }
    let mut i = 0;
    let mut offset = if s.optional_count() == 0 {
        Some(0usize)
    } else {
        None
    };
    while i < s.fields.len() {
        let start = i;
        let mut bytes = 0;
        while let Some(f) = s.fields.get(i) {
            if f.optional
                || !matches!(
                    f.data,
                    QslType::Byte
                        | QslType::Integer { .. }
                        | QslType::RangedInteger { .. }
                        | QslType::Flags(_)
                )
            {
                break;
            }
            bytes += f.data.fixed_wire_size()?.ok_or(QslError::InvalidSchema)?;
            i += 1;
        }
        if i != start {
            let count = i - start;
            let metadata = match s.fields.get(i).filter(|f| !f.optional).map(|f| &f.data) {
                Some(
                    QslType::String(_)
                    | QslType::RangedString { .. }
                    | QslType::GuardString { .. }
                    | QslType::CopyGuardString { .. }
                    | QslType::BinaryString { .. },
                ) => 4,
                Some(QslType::InlineRawString { max, .. }) => {
                    if *max <= 127 {
                        1
                    } else if *max <= 32767 {
                        2
                    } else {
                        4
                    }
                }
                _ => 0,
            };
            let total = bytes + metadata;
            for at in (0..total).step_by(16) {
                let n = (total - at).min(16);
                out.push(RegisterGroup {
                    first: start,
                    fields: count,
                    offset: offset.and_then(|v| v.checked_add(at)),
                    bytes: n,
                    register_bits: n.next_power_of_two() * 8,
                    metadata_bytes: metadata,
                });
            }
            offset = offset.and_then(|v| v.checked_add(bytes));
            if metadata != 0 {
                i += 1;
                offset = None;
            }
            continue;
        }
        let f = &s.fields[i];
        offset = if f.optional {
            None
        } else {
            offset.and_then(|v| v.checked_add(f.data.fixed_wire_size().ok().flatten()?))
        };
        i += 1;
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StorageKind {
    #[default]
    Default,
    StackString(usize),
    StackArray(usize),
    FixedArray(usize),
    StrView,
    HeapString,
}
pub fn field_rust_type(f: &crate::QslField, borrowed: bool) -> QslResult<String> {
    Ok(match f.storage {
        StorageKind::Default => f.data.rust_type(borrowed)?,
        StorageKind::StackString(n) => format!("::qsl::StackString<{n}>"),
        StorageKind::StackArray(n) => {
            let e = match &f.data {
                QslType::RangedArray { element, .. } | QslType::Array(element, _) => element,
                _ => return Err(QslError::InvalidSchema),
            };
            format!("::qsl::InlineArray<{}, {n}>", e.rust_type(false)?)
        }
        StorageKind::FixedArray(n) => {
            let QslType::FixedArray(e, _) = &f.data else {
                return Err(QslError::InvalidSchema);
            };
            format!("[{};{n}]", e.rust_type(false)?)
        }
        StorageKind::StrView => "&'a str".into(),
        StorageKind::HeapString => "String".into(),
    })
}
pub fn lower_storage(s: &QslSchema, limit: usize) -> QslResult<QslSchema> {
    let mut s = s.clone();
    check_storage_limit(&s, limit).map_err(|_| QslError::InvalidSchema)?;
    let borrowed = s.borrow_strings();
    for (index, f) in s.fields.iter_mut().enumerate() {
        let native = s.options.transmutate
            || s.options
                .regions
                .iter()
                .any(|r| r.start <= index && index < r.end);
        f.storage = match &f.data {
            QslType::FixedString(_) | QslType::InlineRawString { .. } if borrowed && !native => {
                StorageKind::StrView
            }
            QslType::FixedString(n) | QslType::FixedCompressedString(n)
                if *n > limit && !native =>
            {
                StorageKind::HeapString
            }
            QslType::CompressedString { max, .. } | QslType::InlineRawString { max, .. }
                if *max > limit =>
            {
                StorageKind::HeapString
            }
            QslType::GuardString { .. } => StorageKind::StrView,
            QslType::FixedArray(e, n)
                if (native || !borrowed)
                    && (native
                        || fixed_native_layout(e)
                            .map(|(size, _)| size)
                            .and_then(|width| width.checked_mul(*n))
                            .is_some_and(|n| n <= limit)) =>
            {
                StorageKind::FixedArray(*n)
            }
            QslType::RangedString { max, .. }
                if s.options.aggressive && !borrowed && *max <= limit =>
            {
                StorageKind::StackString(*max)
            }
            QslType::RangedArray { element, max, .. }
                if s.options.aggressive
                    && !borrowed
                    && element.primitive()?.is_some()
                    && fixed_native_layout(element)
                        .map(|(size, _)| size)
                        .and_then(|n| n.checked_mul(*max))
                        .is_some_and(|n| n <= limit) =>
            {
                StorageKind::StackArray(*max)
            }
            _ => StorageKind::Default,
        };
        // Compatibility syntax maps onto ordinary ownership in new generated code.
        if let QslType::CopyGuardString { min, max } = f.data {
            f.data = match max {
                Some(max) => QslType::RangedString { min, max },
                None => QslType::String(min),
            };
            f.storage = StorageKind::HeapString;
            if s.options.aggressive {
                if let Some(max) = max.filter(|&n| n <= limit) {
                    f.storage = StorageKind::StackString(max)
                }
            }
        }
    }
    Ok(s)
}

pub fn validate_options(s: &QslSchema) -> Result<(), String> {
    let mut names = [
        "validate",
        "validate_values",
        "serialize",
        "serialize_checked",
        "serialize_checked_to",
        "deserialize_checked",
        "deserialize_checked_into",
        "try_new",
        "deserialize_into",
        "deserialize_overlay",
        "deserialize_overlay_unchecked",
        "deserialize_transmutate_copy",
        "deserialize_transmutate_copy_unchecked",
        "serialize_cached",
        "serialize_quantized_to",
        "serialize_qps_to",
        "deserialize_qps",
        "deserialize_qps_into",
        "predict_index",
        "MAX_DATA_SIZE",
        "NATIVE_BYTES",
        "NATIVE_ALIGNMENT",
        "NATIVE_WIRE_COMPATIBLE",
        "WIRE_MAX_BYTES",
        "CACHE_BYTES",
        "SCHEMA_HASHES",
        "SCHEMA_HASH_STRICT",
        "SCHEMA_HASH_LIGHT",
        "QUANT_ACTIVE",
        "QUANTUM",
        "QUANT_PADDING_BYTES",
        "QUANT_MAX_BYTES",
        "QUANT_RESTORE_BYTES",
        "QUANT_WHOLE_WORD_READ",
        "QUANT_WHOLE_WORD_WRITE",
        "QPS_SCALAR_PREFIX_BYTES",
        "QPS_RAW_PREFIX_BYTES",
        "QPS_PREFIX_LENGTH_HEADER",
        "QPS_LENGTHS_QUANTIZED",
        "OPT_TABLE_EVENT_COUNT",
        "OPT_TABLE_MAX_PAYLOAD",
        "OPT_TABLE_HEADER_BYTES",
        "OPT_TABLE_TAG_BYTES",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<std::collections::HashSet<_>>();
    for i in 0..s.options.regions.len() {
        for method in ["copy", "view"] {
            names.insert(format!("region_{i}_{method}"));
            names.insert(format!("deserialize_region_{i}_{method}"));
            names.insert(format!("region_{i}_{method}_unchecked"));
            names.insert(format!("deserialize_region_{i}_{method}_unchecked"));
            names.insert(format!("REGION_{i}_{}_COMPATIBLE", method.to_uppercase()));
            names.insert(format!(
                "REGION_{i}_WIRE_{}_COMPATIBLE",
                method.to_uppercase()
            ));
        }
    }
    for f in &s.fields {
        if optional_representation(f, s.options.aggressive).is_some()
            && (!names.insert(f.name.clone()) || !names.insert(format!("set_{}", f.name)))
        {
            return Err("optional accessor conflicts with a generated method name".into());
        }
    }
    for f in &s.fields {
        if f.name.starts_with("__qsl_") {
            return Err("field names beginning with __qsl_ are reserved".into());
        }
        if optional_representation(f, s.options.aggressive).is_some()
            && [
                "validate",
                "validate_values",
                "serialize",
                "serialize_checked",
                "serialize_checked_to",
                "deserialize_checked",
                "deserialize_checked_into",
                "try_new",
            ]
            .contains(&f.name.as_str())
        {
            return Err("optional accessor conflicts with a generated method name".into());
        }
    }

    if s.options.slots.is_some_and(|n| n == 0 || n > 64) {
        return Err("slots must be 1..64".into());
    }
    if s.options.transmutate
        && (s.compact()
            || s.qps()
            || s.optional_count() != 0
            || s.fields.iter().any(|f| {
                f.data.fixed_wire_size().ok().flatten().is_none()
                    || matches!(
                        f.data,
                        QslType::Custom { .. } | QslType::FixedCompressedString(_)
                    )
            }))
    {
        return Err("transmutate requires fixed compatible fields; compact, QPS, dynamic and optional fields are not native overlays".into());
    }
    if s.options.transmutate && s.borrow_strings() {
        check_native_offsets(s)?;
    }
    if s.qps() {
        let mut dynamic = false;
        for f in &s.fields {
            let numeric = matches!(
                f.data,
                QslType::Byte
                    | QslType::Integer { .. }
                    | QslType::RangedInteger { .. }
                    | QslType::Flags(_)
            );
            if numeric && dynamic {
                return Err("QPS scalar numeric fields must precede strings and arrays; their lengths and contents are never quantized".into());
            }
            dynamic |= !numeric;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct NativeRegion {
    pub rust_type: String,
    pub start: usize,
    pub end: usize,
}
fn fixed_native_layout(t: &QslType) -> Option<(usize, usize)> {
    let native = match t {
        QslType::None | QslType::Padding(_) => (0, 1),
        QslType::FixedString(n) => (*n, 1),
        QslType::FixedArray(e, n) => {
            let (s, a) = fixed_native_layout(e)?;
            (s.checked_mul(*n)?, a)
        }
        _ => {
            let p = t.primitive().ok()??;
            let n = p[1..].parse::<usize>().ok()? / 8;
            let a = match n {
                1 => std::mem::align_of::<u8>(),
                2 => std::mem::align_of::<u16>(),
                4 => std::mem::align_of::<u32>(),
                8 => std::mem::align_of::<u64>(),
                _ => return None,
            };
            (n, a)
        }
    };
    Some(native)
}
pub fn check_native_offsets(s: &QslSchema) -> Result<(), String> {
    let (mut wire, mut native, mut alignment) = (0usize, 0usize, 1usize);
    for f in &s.fields {
        let (n, a) = fixed_native_layout(&f.data)
            .ok_or_else(|| format!("{}: native layout is not statically known", f.name))?;
        alignment = alignment.max(a);
        native = native
            .checked_add(a - 1)
            .map(|n| n & !(a - 1))
            .ok_or("native layout overflow")?;
        if !matches!(f.data, QslType::Padding(_) | QslType::None) && native != wire {
            return Err(format!("{}: transmutate wire offset {wire} differs from native offset {native}; add explicit padding or reorder fields",f.name));
        }
        native = native.checked_add(n).ok_or("native layout overflow")?;
        wire = wire
            .checked_add(
                f.data
                    .fixed_wire_size()
                    .ok()
                    .flatten()
                    .ok_or("variable native field")?,
            )
            .ok_or("wire layout overflow")?;
    }
    native = native
        .checked_add(alignment - 1)
        .map(|n| n & !(alignment - 1))
        .ok_or("native layout overflow")?;
    if native != wire {
        return Err(format!("transmutate wire size {wire} differs from native size {native}; add explicit trailing padding"));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct OptionalRepresentation {
    pub rust_type: String,
    pub niche: Option<String>,
    pub minimum: i128,
    pub maximum: i128,
    pub shifted: bool,
}
pub fn optional_representation(
    f: &crate::QslField,
    aggressive: bool,
) -> Option<OptionalRepresentation> {
    if !f.optional {
        return None;
    }
    let ty = f.data.primitive().ok()??.to_string();
    let (min, max) = match f.data {
        QslType::Byte => (0, 255),
        QslType::Flags(n) => (0, (1i128 << n) - 1),
        QslType::RangedInteger { min, max, .. } => (min, max),
        QslType::Integer { bits, signed, .. } => {
            if signed {
                (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
            } else {
                (0, (1i128 << bits) - 1)
            }
        }
        _ => return None,
    };
    let (niche, shifted) = if min > 0 || max < 0 {
        (
            Some(format!(
                "std::num::NonZero{}{}",
                if ty.starts_with('i') { "I" } else { "U" },
                &ty[1..]
            )),
            false,
        )
    } else if aggressive && max - min < (u64::MAX as i128) {
        let code = (max - min + 1) as u64;
        let bits = if code <= u8::MAX as u64 {
            8
        } else if code <= u16::MAX as u64 {
            16
        } else if code <= u32::MAX as u64 {
            32
        } else {
            64
        };
        if bits <= ty[1..].parse::<usize>().ok()? {
            (Some(format!("std::num::NonZeroU{bits}")), true)
        } else {
            (None, false)
        }
    } else {
        (None, false)
    };
    Some(OptionalRepresentation {
        rust_type: ty,
        niche,
        minimum: min,
        maximum: max,
        shifted,
    })
}

pub fn check_storage_limit(s: &QslSchema, limit: usize) -> Result<(), String> {
    if s.borrow_strings()
        && !s.options.transmutate
        && s.fields.iter().any(|f| {
            matches!(
                f.data,
                QslType::CompressedString { .. } | QslType::FixedCompressedString(_)
            )
        })
    {
        return Err(
            "LIFETIME cannot borrow compressed strings; use COPY with a bounded restoration buffer"
                .into(),
        );
    }
    for (index, f) in s.fields.iter().enumerate() {
        if !(s.options.transmutate
            || s.options
                .regions
                .iter()
                .any(|r| r.start <= index && index < r.end))
        {
            continue;
        }
        let bytes = match &f.data {
            QslType::FixedString(n) => Some(*n),
            QslType::FixedArray(e, n) => {
                fixed_native_layout(e).and_then(|(size, _)| size.checked_mul(*n))
            }
            _ => None,
        };
        if let Some(bytes) = bytes {
            if bytes > limit {
                return Err(format!("{}: fixed transmutate buffer {bytes} B exceeds STACK_LIMIT {limit} B; raise the limit explicitly",f.name));
            }
        }
    }
    Ok(())
}
