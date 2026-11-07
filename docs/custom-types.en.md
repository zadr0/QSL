# Custom types and reusable Rust structures

QslCustom declares exact normal SIZE and compact COMP_SIZE, validators for wire bytes/values and explicit unsafe pointer hooks. Both validators must authorize the custom domain before checked code calls its trusted hooks. A changed COMP_SIZE requires matching compact read/write implementations; the default hooks use the normal codec. Source arrays accept built-ins and explicitly registered primitives.

## Explicit scalar wrapper macros

A schema field-expansion macro and a scalar serialization contract serve different purposes. Existing user `QslCustom` codecs remain opaque and retain their byte format. They are not automatically reinterpreted as integers.

In a Rust module shared by the application and build script:

```rust
qsl_lexer::qsl_primitive!(pub struct DeviceId(u32);range=(1,100););
```

Or bind an existing one-field integer tuple struct:

```rust
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DeviceId(pub u32);
qsl_lexer::qsl_primitive!(DeviceId,u32,range=(1,100));
```

Supported representations: u8/i8, u16/i16, u32/i32 and u64/i64. The macro checks integer kind, size, alignment, field-zero offset, absence of Drop and domain bounds. It constructs values with typed Rust constructors. Normal encoding is little endian; compact encoding stores `value-min` in the minimum whole-byte width for the declared domain. This is byte-oriented custom compression, not automatic bit merging with neighboring built-in fields. A constant domain occupies zero compact bytes. Existing PS restrictions on zero-width optional codecs still apply.

Register the same type before parsing:

```rust,ignore
use qsl_interpreter::prelude::*;
let builder = QslSchemaBuilder::new()
    .with_primitive::<DeviceId>("DeviceId")?
    .with_source("&Packet[safety,compact,optionals=bitmap,opt_table]{DeviceId id;DeviceId? extra;}")?;
let rust = builder.use_interpreter()?;
```

Registration checks build-side ABI and retains normal custom width. Generated const assertions check target-side size, alignment, Drop and codec widths, including array elements. Primitive metadata is reapplied after explicit Rust-template imports. Registered primitives can also appear as QSL array elements; unregistered opaque array elements are rejected by the source parser.

For native regions, the application additionally opts into the existing runtime contract:

```rust,ignore
qsl_lexer::qsl_native_primitive!(DeviceId,u32);
```

The native macro validates scalar values before creating a checked view. Manual `unsafe impl QslPrimitive` / `NativeField` remains an explicit responsibility: layout checks cannot prove arbitrary Rust method semantics. No general user-type transmute or automatic merged-register kernel is introduced. Nested optional schemas inside a custom codec keep their own masks and marker scope.

## Real nested native regions

```qsl
&Parent[safety] {
    #region(aggressive_transmutate="Meta");
    u64 id;
    u32 value;
    str(4) tag;
    #end_region;
    str(3..10) name;
}
```

This produces `#[repr(C)] Meta { id:u64, value:u32, tag:[u8;4] }` and `Parent { Meta:Meta, name:String }`. Region fields are not flattened into Parent. The historical `aggresive_transmutate` spelling is accepted as an alias.

COPY owns the nested object and falls back to field reconstruction where native and wire layouts differ. LIFETIME borrows a typed region only when wire/native sizes and field offsets match, the host is little endian, input alignment is sufficient, and all field validators succeed. An ordinary Vec buffer does not promise alignment for arbitrary native types; misaligned checked borrowing returns an error. COPY does not require borrowed-input alignment.

Nested regions produce a type hierarchy. Identical definitions are reused; conflicting names, crossed/empty regions, recursive names, unknown native types, dynamic fields and optional fields inside native regions are rejected. Fixed strings use byte arrays. The configured stack limit applies to native string/array fields too. Native descriptors support at most 64 initialized leaves; numeric arrays use their existing coalesced leaf representation. Region nesting is capped at 64.

A Rust `#[qsl::schem_macro(aggressive_transmutate)]` template reuses its existing Rust type as a nested field; the generator does not emit a duplicate struct. Ordinary field-expansion macros keep their previous flattening behavior. Explicit `repr(align(...))` is accounted for; extra padding can make LIFETIME unavailable while COPY remains usable.
