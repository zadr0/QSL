#![allow(clippy::type_complexity)]
pub use qsl_v040_comparison::{Buffers, Codec, Fixture, Kind, Result, View, CAPACITY, FIXTURES};
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
}
macro_rules! numeric {
    ($name:ident,$scratch:ty,$label:literal,$borrow:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = generated::$name;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Numeric;
            fn prepare(f: &Fixture) -> Self::Input<'_> {
                generated::$name {
                    f0: f.numbers[0],
                    f1: f.numbers[1],
                    f2: f.numbers[2],
                    f3: f.numbers[3],
                }
            }
            fn encode(v: &Self::Input<'_>, b: &mut Buffers<'_>) -> Result<usize> {
                Ok(v.serialize_checked_to(&mut b.out)?)
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let mut scratch = [0 as $scratch; 4];
                let v = generated::$name::deserialize_checked(input, &mut scratch)?;
                std::hint::black_box(&v);
                Ok(consume(View::Numeric([v.f0, v.f1, v.f2, v.f3])))
            }
        }
    };
}
macro_rules! optional {
    ($name:ident,$scratch:ty,$label:literal,$borrow:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = generated::$name;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Optionals;
            fn prepare(f: &Fixture) -> Self::Input<'_> {
                generated::$name {
                    id: f.id,
                    a: f.a,
                    b: f.b,
                    c: f.c,
                }
            }
            fn encode(v: &Self::Input<'_>, b: &mut Buffers<'_>) -> Result<usize> {
                Ok(v.serialize_checked_to(&mut b.out)?)
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let mut scratch = [0 as $scratch; 4];
                let v = generated::$name::deserialize_checked(input, &mut scratch)?;
                std::hint::black_box(&v);
                Ok(consume(View::Optionals(v.id, v.a, v.b, v.c)))
            }
        }
    };
}
macro_rules! text {
    ($name:ident,$scratch:ty,$label:literal,false) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = generated::$name;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Text;
            fn prepare(f: &Fixture) -> Self::Input<'_> {
                generated::$name {
                    id: f.id,
                    text: f.text.clone(),
                }
            }
            fn encode(v: &Self::Input<'_>, b: &mut Buffers<'_>) -> Result<usize> {
                Ok(v.serialize_checked_to(&mut b.out)?)
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let mut scratch = [0 as $scratch; 2];
                let v = generated::$name::deserialize_checked(input, &mut scratch)?;
                std::hint::black_box(&v);
                Ok(consume(View::Text(v.id, &v.text)))
            }
        }
    };
    ($name:ident,$scratch:ty,$label:literal,true) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = generated::$name<'a>;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Text;
            const BORROWED: bool = true;
            const WRITE: bool = false;
            fn prepare(f: &Fixture) -> Self::Input<'_> {
                generated::$name {
                    id: f.id,
                    text: &f.text,
                }
            }
            fn encode(v: &Self::Input<'_>, b: &mut Buffers<'_>) -> Result<usize> {
                Ok(v.serialize_checked_to(&mut b.out)?)
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let mut scratch = [0 as $scratch; 2];
                let v = generated::$name::deserialize_checked(input, &mut scratch)?;
                std::hint::black_box(&v);
                Ok(consume(View::Text(v.id, v.text)))
            }
        }
    };
}
numeric!(NumLine16, u64, "dyn_line_16", false);
optional!(OptLine16, u64, "dyn_line_16", false);
text!(TextLine16, u64, "dyn_line_16", false);
text!(BorrowLine16, u64, "dyn_line_16", true);
numeric!(NumLine32, u64, "dyn_line_32", false);
optional!(OptLine32, u64, "dyn_line_32", false);
text!(TextLine32, u64, "dyn_line_32", false);
text!(BorrowLine32, u64, "dyn_line_32", true);
numeric!(NumLine64, u64, "dyn_line_64", false);
optional!(OptLine64, u64, "dyn_line_64", false);
text!(TextLine64, u64, "dyn_line_64", false);
text!(BorrowLine64, u64, "dyn_line_64", true);
numeric!(NumDynamic32, u32, "dyn_index_dynamic_32", false);
optional!(OptDynamic32, u32, "dyn_index_dynamic_32", false);
text!(TextDynamic32, u32, "dyn_index_dynamic_32", false);
text!(BorrowDynamic32, u32, "dyn_index_dynamic_32", true);
numeric!(
    NumCompactDynamic32,
    u32,
    "dyn_index_dynamic_32_compact",
    false
);
numeric!(NumDynamic64, u32, "dyn_index_dynamic_64", false);
optional!(OptDynamic64, u32, "dyn_index_dynamic_64", false);
text!(TextDynamic64, u32, "dyn_index_dynamic_64", false);
text!(BorrowDynamic64, u32, "dyn_index_dynamic_64", true);
numeric!(
    NumCompactDynamic64,
    u32,
    "dyn_index_dynamic_64_compact",
    false
);
numeric!(NumStrict32, u32, "dyn_index_strict_32", false);
optional!(OptStrict32, u32, "dyn_index_strict_32", false);
text!(TextStrict32, u32, "dyn_index_strict_32", false);
text!(BorrowStrict32, u32, "dyn_index_strict_32", true);
numeric!(
    NumCompactStrict32,
    u32,
    "dyn_index_strict_32_compact",
    false
);
numeric!(NumStrict64, u32, "dyn_index_strict_64", false);
optional!(OptStrict64, u32, "dyn_index_strict_64", false);
text!(TextStrict64, u32, "dyn_index_strict_64", false);
text!(BorrowStrict64, u32, "dyn_index_strict_64", true);
numeric!(
    NumCompactStrict64,
    u32,
    "dyn_index_strict_64_compact",
    false
);
#[macro_export]
macro_rules! visit_dyn_cases {
    ($callback:ident,$context:expr) => {
        $callback::<$crate::NumLine16>($context, 0);
        $callback::<$crate::OptLine16>($context, 0);
        $callback::<$crate::TextLine16>($context, 1024);
        $callback::<$crate::BorrowLine16>($context, 1024);
        $callback::<$crate::NumLine32>($context, 0);
        $callback::<$crate::OptLine32>($context, 0);
        $callback::<$crate::TextLine32>($context, 1024);
        $callback::<$crate::BorrowLine32>($context, 1024);
        $callback::<$crate::NumLine64>($context, 0);
        $callback::<$crate::OptLine64>($context, 0);
        $callback::<$crate::TextLine64>($context, 1024);
        $callback::<$crate::BorrowLine64>($context, 1024);
        $callback::<$crate::NumDynamic32>($context, 0);
        $callback::<$crate::OptDynamic32>($context, 0);
        $callback::<$crate::TextDynamic32>($context, 1024);
        $callback::<$crate::BorrowDynamic32>($context, 1024);
        $callback::<$crate::NumCompactDynamic32>($context, 0);
        $callback::<$crate::NumDynamic64>($context, 0);
        $callback::<$crate::OptDynamic64>($context, 0);
        $callback::<$crate::TextDynamic64>($context, 1024);
        $callback::<$crate::BorrowDynamic64>($context, 1024);
        $callback::<$crate::NumCompactDynamic64>($context, 0);
        $callback::<$crate::NumStrict32>($context, 0);
        $callback::<$crate::OptStrict32>($context, 0);
        $callback::<$crate::TextStrict32>($context, 1024);
        $callback::<$crate::BorrowStrict32>($context, 1024);
        $callback::<$crate::NumCompactStrict32>($context, 0);
        $callback::<$crate::NumStrict64>($context, 0);
        $callback::<$crate::OptStrict64>($context, 0);
        $callback::<$crate::TextStrict64>($context, 1024);
        $callback::<$crate::BorrowStrict64>($context, 1024);
        $callback::<$crate::NumCompactStrict64>($context, 0);
    };
}
