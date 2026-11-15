---
title: "Macros, primitive wrappers and custom codecs"
description: "How Rust templates expand into QSL fields, how primitive wrappers are validated, and when a custom codec is needed."
---

QSL supports three distinct extensions. A **template** contributes fields to a schema. A **primitive wrapper** gives an integer a Rust type and a range. A **custom codec** supplies its own fixed-size binary representation. Select the smallest contract that describes your type.

## Rust templates

Place `#[qsl::qsl_macro]` or its synonym `#[qsl::schem_macro]` on an ordinary Rust struct with named fields. The Rust struct remains available. The attribute implements `QslMacro`: during the build, it returns field descriptions, not serialized values.

A template field named `meta` containing `id` produces `meta_id`. Nested templates expand recursively: `meta.location.zone` becomes `meta_location_zone`. Tags are reassigned after expansion. Consequently, optional fields in a template must still end up after every required field of the resulting schema. Cycles, duplicate names and unsupported optional template groups are errors.

~~~rust
#[qsl::qsl_macro]
struct Location { zone: u16 }

#[qsl::qsl_macro]
struct Meta { id: u64, location: Location }

let mut builder = qsl::QslBuilder::new();
builder.schem(|schema| {
    schema.name("Packet").safety();
    schema.field("meta").r#macro::<Meta>();
});
let schemas = builder.build()?;
~~~

The generated Packet contains `meta_id: u64` and `meta_location_zone: u16`. It does not contain a nested Meta field. Field expansion is separate from the native-region mechanism, which deliberately creates a nested Rust type.

Source-file import requires the `rust-macros` build feature and `from_files_with_rust` or `build_qsl_files_with_rust`. The importer reads supported Rust definitions; it does not execute arbitrary Rust expressions. The typed Builder uses `QslFieldType` for built-in fields and `QslMacro` for nested templates. Generic and conditional source declarations have restrictions; diagnostics must be resolved rather than treating an unknown type as a byte array.

## Integer wrappers

`qsl_primitive!` defines or binds a one-field tuple struct. The declaration form adds `repr(transparent)`, Copy and Clone. The binding form checks the existing type. Supported representations are the sealed integer primitives up to 64 bits; built-in QSL u128/i128 fields do not extend this macro contract.

~~~rust
qsl::qsl_primitive!(pub struct DeviceId(u32); range=(1,100););

let builder = qsl::QslSchemaBuilder::new()
    .with_primitive::<DeviceId>("DeviceId")?
    .with_source("&Packet[safety]{DeviceId id;}")?;
~~~

The macro checks size, alignment, tuple-field offset zero, the absence of Drop and the validity of the range at compile time. It implements `QslPrimitive` metadata and `QslCustom` reading, writing and validation. Register the same definition before parsing a schema that names it. A name alone does not prove the representation of an external type.

In raw mode, DeviceId uses the little-endian u32 representation. Its custom compact codec stores a byte-rounded rank. For this example, 100 values need 7 rank bits and occupy 1 custom-codec byte. This is different from claiming that every custom field is packed bit by bit with its neighbours.

$$
N=b-a+1,\qquad r=x-a.
$$

$$
w=\lceil\log_2 N\rceil,\qquad B_{\mathrm{custom}}=\lceil w/8\rceil.
$$

For a constant domain the rank width is zero. Spare ranks and out-of-range raw values are rejected by checked methods. The public tuple field can still be assigned an invalid value; only validation or a restricted application constructor makes a particular value valid. The macro does not automatically create an Option niche.

## Native primitive validation

`qsl_native_primitive!(DeviceId, u32)` additionally implements `NativeField` in the application. Use it after primitive registration when an explicit native region contains that wrapper. It checks representation metadata and validates the numeric domain before a checked native copy/view can publish a value. It does not make padding initialized, remove alignment requirements or permit reading an arbitrary Rust object as wire bytes.

`#[qsl_macro(aggressive_transmutate)]` is a separate native-struct form: fixed named fields, no generic parameters, repr(C), optional explicit alignment, Copy/Clone and leaf validators. In that form, a fixed InlineString field is lowered to its fixed UTF-8 byte array. Packed/Rust/transparent representation options are rejected. Inspect the resulting type before relying on its layout.

## Custom binary formats

Implement `QslCustom` only when the existing numeric or template contracts do not describe the format. Declare `SIZE` and `COMP_SIZE` and provide raw/compact unsafe read and write methods. `qsl_validate` must reject malformed input bytes; `qsl_validate_values` must reject values that cannot be written correctly. The defaults reject an unknown codec in checked operations.

Validation must prove exactly the representation consumed by the unsafe decoder. A matching Rust name or object size is insufficient. The developer is responsible for initialized bytes, valid values, capacity, non-overlap and any codec-specific invariants. Core runtime contracts remain allocator-free; schema metadata and macro registration belong to the build-time lexer.

[Safety](/guides/safety/) · [Manual layout](/guides/manual-layout/)
