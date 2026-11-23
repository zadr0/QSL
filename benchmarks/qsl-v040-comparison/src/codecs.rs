use crate::{fixtures::*, generated as q, records_capnp as cap};
use prost::Message;
use std::hint::black_box;

pub struct Buffers<'a> {
    pub out: Vec<u8>,
    pub cap: capnp::message::ScratchSpaceHeapAllocator<'a>,
}
impl<'a> Buffers<'a> {
    pub fn new(scratch: &'a mut [u8]) -> Self {
        Self {
            out: vec![0; CAPACITY],
            cap: capnp::message::ScratchSpaceHeapAllocator::new(scratch),
        }
    }
}
pub trait Codec {
    type Input<'a>;
    const NAME: &'static str;
    const KIND: Kind;
    const BORROWED: bool = false;
    const WRITE: bool = true;
    fn prepare<'a>(f: &'a Fixture) -> Self::Input<'a>;
    fn encode(v: &Self::Input<'_>, buf: &mut Buffers<'_>) -> Result<usize>;
    fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R>;
}

macro_rules! qnum {
    ($name:ident,$ty:ident,$label:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = q::$ty;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Numeric;
            fn prepare(f: &Fixture) -> q::$ty {
                q::$ty {
                    f0: f.numbers[0],
                    f1: f.numbers[1],
                    f2: f.numbers[2],
                    f3: f.numbers[3],
                }
            }
            #[inline]
            fn encode(v: &q::$ty, b: &mut Buffers<'_>) -> Result<usize> {
                Ok(v.serialize_checked_to(&mut b.out)?)
            }
            #[inline]
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let v = q::$ty::deserialize_checked(input)?;
                black_box(&v);
                Ok(consume(View::Numeric([v.f0, v.f1, v.f2, v.f3])))
            }
        }
    };
}
qnum!(QNumDefault, NumDefault, "qsl_default");
qnum!(QNumCompact, NumCompact, "qsl_compact");
qnum!(QNumAligned, NumAligned, "qsl_compact_alignment");
qnum!(QNumPointing, NumPointing, "qsl_pointing");
qnum!(QNumBmi2, NumBmi2, "qsl_compact_group_bmi2");
macro_rules! qopt {
    ($name:ident,$ty:ident,$label:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = q::$ty;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Optionals;
            fn prepare(f: &Fixture) -> q::$ty {
                q::$ty::try_new(f.id, f.a, f.b, f.c).unwrap()
            }
            #[inline]
            fn encode(v: &q::$ty, b: &mut Buffers<'_>) -> Result<usize> {
                Ok(v.serialize_checked_to(&mut b.out)?)
            }
            #[inline]
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let v = q::$ty::deserialize_checked(input)?;
                black_box(&v);
                Ok(consume(View::Optionals(v.id, v.a(), v.b(), v.c())))
            }
        }
    };
}
qopt!(QOptDefault, OptDefault, "qsl_default");
qopt!(QOptTable, OptTable, "qsl_ot");
qopt!(QOptPredict, OptPredict, "qsl_ps");
qopt!(QOptPredictTable, OptPredictTable, "qsl_ps_ot");
qopt!(QOptBitmap, OptBitmap, "qsl_bm");
qopt!(QOptBitmapTable, OptBitmapTable, "qsl_bm_ot");
pub struct QTextHeap;
pub struct QTextInline;
pub struct QTextBorrow;
macro_rules! qtext {
    ($name:ident,$ty:ident,$make:expr,$label:literal) => {
        impl Codec for $name {
            type Input<'a> = q::$ty;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Text;
            fn prepare(f: &Fixture) -> q::$ty {
                q::$ty {
                    id: f.id,
                    text: ($make)(f),
                }
            }
            #[inline]
            fn encode(v: &q::$ty, b: &mut Buffers<'_>) -> Result<usize> {
                Ok(v.serialize_checked_to(&mut b.out)?)
            }
            #[inline]
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let v = q::$ty::deserialize_checked(input)?;
                black_box(&v);
                Ok(consume(View::Text(v.id, v.text.as_str())))
            }
        }
    };
}
qtext!(
    QTextHeap,
    TextHeap,
    |f: &Fixture| f.text.clone(),
    "qsl_heap"
);
qtext!(
    QTextInline,
    TextInline,
    |f: &Fixture| qsl::StackString::try_from_str(&f.text).unwrap(),
    "qsl_inline"
);
impl Codec for QTextBorrow {
    type Input<'a> = q::TextBorrow<'a>;
    const NAME: &'static str = "qsl_lifetime";
    const KIND: Kind = Kind::Text;
    const BORROWED: bool = true;
    fn prepare(f: &Fixture) -> q::TextBorrow<'_> {
        q::TextBorrow {
            id: f.id,
            text: &f.text,
        }
    }
    #[inline]
    fn encode(v: &q::TextBorrow<'_>, b: &mut Buffers<'_>) -> Result<usize> {
        Ok(v.serialize_checked_to(&mut b.out)?)
    }
    #[inline]
    fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
        let v = q::TextBorrow::deserialize_checked(input)?;
        black_box(&v);
        Ok(consume(View::Text(v.id, v.text)))
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct ProtoNumeric {
    #[prost(uint32, tag = "1")]
    pub f0: u32,
    #[prost(uint32, tag = "2")]
    pub f1: u32,
    #[prost(uint32, tag = "3")]
    pub f2: u32,
    #[prost(uint32, tag = "4")]
    pub f3: u32,
}
#[derive(Clone, PartialEq, Message)]
pub struct ProtoFixed {
    #[prost(fixed32, tag = "1")]
    pub f0: u32,
    #[prost(fixed32, tag = "2")]
    pub f1: u32,
    #[prost(fixed32, tag = "3")]
    pub f2: u32,
    #[prost(fixed32, tag = "4")]
    pub f3: u32,
}
#[derive(Clone, PartialEq, Message)]
pub struct ProtoOptionals {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(uint32, optional, tag = "2")]
    pub a: Option<u32>,
    #[prost(uint32, optional, tag = "3")]
    pub b: Option<u32>,
    #[prost(uint32, optional, tag = "4")]
    pub c: Option<u32>,
}
#[derive(Clone, PartialEq, Message)]
pub struct ProtoText {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(string, tag = "2")]
    pub text: String,
}
fn proto_encode<M: Message>(v: &M, b: &mut Buffers<'_>) -> Result<usize> {
    let mut dest = &mut b.out[..];
    v.encode(&mut dest)?;
    Ok(CAPACITY - dest.len())
}
macro_rules! pnum {
    ($name:ident,$ty:ident,$label:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = $ty;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Numeric;
            fn prepare(f: &Fixture) -> $ty {
                $ty {
                    f0: f.numbers[0],
                    f1: f.numbers[1],
                    f2: f.numbers[2],
                    f3: f.numbers[3],
                }
            }
            #[inline]
            fn encode(v: &$ty, b: &mut Buffers<'_>) -> Result<usize> {
                numeric_valid([v.f0, v.f1, v.f2, v.f3])?;
                proto_encode(v, b)
            }
            #[inline]
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let v = $ty::decode(input)?;
                let n = [v.f0, v.f1, v.f2, v.f3];
                numeric_valid(n)?;
                black_box(&v);
                Ok(consume(View::Numeric(n)))
            }
        }
    };
}
pnum!(PNumVar, ProtoNumeric, "protobuf_varint");
pnum!(PNumFixed, ProtoFixed, "protobuf_fixed32");
pub struct POptionals;
impl Codec for POptionals {
    type Input<'a> = ProtoOptionals;
    const NAME: &'static str = "protobuf";
    const KIND: Kind = Kind::Optionals;
    fn prepare(f: &Fixture) -> ProtoOptionals {
        ProtoOptionals {
            id: f.id,
            a: f.a.map(u32::from),
            b: f.b.map(u32::from),
            c: f.c,
        }
    }
    fn encode(v: &ProtoOptionals, b: &mut Buffers<'_>) -> Result<usize> {
        if v.a.is_some_and(|v| v > 255) || v.b.is_some_and(|v| v > 65535) {
            return Err("optional domain".into());
        }
        proto_encode(v, b)
    }
    fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
        let v = ProtoOptionals::decode(input)?;
        let a = v.a.map(u8::try_from).transpose()?;
        let b = v.b.map(u16::try_from).transpose()?;
        black_box(&v);
        Ok(consume(View::Optionals(v.id, a, b, v.c)))
    }
}
pub struct PText;
impl Codec for PText {
    type Input<'a> = ProtoText;
    const NAME: &'static str = "protobuf";
    const KIND: Kind = Kind::Text;
    fn prepare(f: &Fixture) -> ProtoText {
        ProtoText {
            id: f.id,
            text: f.text.clone(),
        }
    }
    fn encode(v: &ProtoText, b: &mut Buffers<'_>) -> Result<usize> {
        text_valid(&v.text)?;
        proto_encode(v, b)
    }
    fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
        let v = ProtoText::decode(input)?;
        text_valid(&v.text)?;
        black_box(&v);
        Ok(consume(View::Text(v.id, &v.text)))
    }
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct BinNum(pub [u32; 4]);
#[derive(bincode::Encode, bincode::Decode)]
pub struct BinNarrow(pub [u16; 4]);
#[derive(bincode::Encode, bincode::Decode)]
pub struct BinOpt(pub u32, pub Option<u8>, pub Option<u16>, pub Option<u32>);
#[derive(bincode::Encode, bincode::Decode)]
pub struct BinText(pub u32, pub String);
#[derive(bincode::Encode, bincode::BorrowDecode)]
pub struct BinTextRef<'a>(pub u32, pub &'a str);
macro_rules! bnum {
    ($name:ident,$config:expr,$label:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = BinNum;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Numeric;
            fn prepare(f: &Fixture) -> BinNum {
                BinNum(f.numbers)
            }
            fn encode(v: &BinNum, b: &mut Buffers<'_>) -> Result<usize> {
                numeric_valid(v.0)?;
                Ok(bincode::encode_into_slice(v, &mut b.out, $config)?)
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let (v, n): (BinNum, usize) = bincode::decode_from_slice(input, $config)?;
                exact(n, input)?;
                numeric_valid(v.0)?;
                black_box(&v);
                Ok(consume(View::Numeric(v.0)))
            }
        }
    };
}
bnum!(
    BNumVar,
    bincode::config::standard().with_limit::<8192>(),
    "bincode_varint"
);
bnum!(
    BNumFixed,
    bincode::config::standard()
        .with_fixed_int_encoding()
        .with_limit::<8192>(),
    "bincode_fixint"
);
pub struct BNumNarrow;
impl Codec for BNumNarrow {
    type Input<'a> = BinNarrow;
    const NAME: &'static str = "bincode_u16_fixint";
    const KIND: Kind = Kind::Numeric;
    fn prepare(f: &Fixture) -> BinNarrow {
        numeric_valid(f.numbers).unwrap();
        BinNarrow(f.numbers.map(|v| v as u16))
    }
    fn encode(v: &BinNarrow, b: &mut Buffers<'_>) -> Result<usize> {
        numeric_valid(v.0.map(u32::from))?;
        Ok(bincode::encode_into_slice(
            v,
            &mut b.out,
            bincode::config::standard()
                .with_fixed_int_encoding()
                .with_limit::<8192>(),
        )?)
    }
    fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
        let (v, n): (BinNarrow, usize) = bincode::decode_from_slice(
            input,
            bincode::config::standard()
                .with_fixed_int_encoding()
                .with_limit::<8192>(),
        )?;
        exact(n, input)?;
        let numbers = v.0.map(u32::from);
        numeric_valid(numbers)?;
        black_box(&v);
        Ok(consume(View::Numeric(numbers)))
    }
}
macro_rules! bopt {
    ($name:ident,$config:expr,$label:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = BinOpt;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Optionals;
            fn prepare(f: &Fixture) -> BinOpt {
                BinOpt(f.id, f.a, f.b, f.c)
            }
            fn encode(v: &BinOpt, b: &mut Buffers<'_>) -> Result<usize> {
                Ok(bincode::encode_into_slice(v, &mut b.out, $config)?)
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let (v, n): (BinOpt, usize) = bincode::decode_from_slice(input, $config)?;
                exact(n, input)?;
                black_box(&v);
                Ok(consume(View::Optionals(v.0, v.1, v.2, v.3)))
            }
        }
    };
}
bopt!(
    BOptVar,
    bincode::config::standard().with_limit::<8192>(),
    "bincode_varint"
);
bopt!(
    BOptFixed,
    bincode::config::standard()
        .with_fixed_int_encoding()
        .with_limit::<8192>(),
    "bincode_fixint"
);
macro_rules! btext {
    ($name:ident,$config:expr,$label:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = BinText;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Text;
            fn prepare(f: &Fixture) -> BinText {
                BinText(f.id, f.text.clone())
            }
            fn encode(v: &BinText, b: &mut Buffers<'_>) -> Result<usize> {
                text_valid(&v.1)?;
                Ok(bincode::encode_into_slice(v, &mut b.out, $config)?)
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let (v, n): (BinText, usize) = bincode::decode_from_slice(input, $config)?;
                exact(n, input)?;
                text_valid(&v.1)?;
                black_box(&v);
                Ok(consume(View::Text(v.0, &v.1)))
            }
        }
    };
}
btext!(
    BTextVar,
    bincode::config::standard().with_limit::<8192>(),
    "bincode_varint"
);
btext!(
    BTextFixed,
    bincode::config::standard()
        .with_fixed_int_encoding()
        .with_limit::<8192>(),
    "bincode_fixint"
);
macro_rules! bborrow {
    ($name:ident,$config:expr,$label:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = BinTextRef<'a>;
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Text;
            const BORROWED: bool = true;
            const WRITE: bool = false;
            fn prepare(f: &Fixture) -> BinTextRef<'_> {
                BinTextRef(f.id, &f.text)
            }
            fn encode(v: &BinTextRef<'_>, b: &mut Buffers<'_>) -> Result<usize> {
                text_valid(v.1)?;
                Ok(bincode::encode_into_slice(v, &mut b.out, $config)?)
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let (v, n): (BinTextRef<'_>, usize) =
                    bincode::borrow_decode_from_slice(input, $config)?;
                exact(n, input)?;
                text_valid(v.1)?;
                black_box(&v);
                Ok(consume(View::Text(v.0, v.1)))
            }
        }
    };
}
bborrow!(
    BTextBorrowVar,
    bincode::config::standard().with_limit::<8192>(),
    "bincode_varint"
);
bborrow!(
    BTextBorrowFixed,
    bincode::config::standard()
        .with_fixed_int_encoding()
        .with_limit::<8192>(),
    "bincode_fixint"
);

macro_rules! capnum {
    ($name:ident,$module:ident,$int:ty,$label:literal) => {
        pub struct $name;
        impl Codec for $name {
            type Input<'a> = [u32; 4];
            const NAME: &'static str = $label;
            const KIND: Kind = Kind::Numeric;
            fn prepare(f: &Fixture) -> [u32; 4] {
                f.numbers
            }
            fn encode(v: &[u32; 4], b: &mut Buffers<'_>) -> Result<usize> {
                numeric_valid(*v)?;
                let mut message = capnp::message::Builder::new(&mut b.cap);
                {
                    let mut r = message.init_root::<cap::$module::Builder>();
                    r.set_f0(v[0] as $int);
                    r.set_f1(v[1] as $int);
                    r.set_f2(v[2] as $int);
                    r.set_f3(v[3] as $int);
                }
                let mut dest = &mut b.out[..];
                capnp::serialize::write_message(&mut dest, &message)?;
                Ok(CAPACITY - dest.len())
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let mut rest = input;
                let message = capnp::serialize::read_message_from_flat_slice_no_alloc(
                    &mut rest,
                    reader_options(),
                )?;
                exact(input.len() - rest.len(), input)?;
                let r = message.get_root::<cap::$module::Reader>()?;
                let n = [
                    u32::from(r.get_f0()),
                    u32::from(r.get_f1()),
                    u32::from(r.get_f2()),
                    u32::from(r.get_f3()),
                ];
                numeric_valid(n)?;
                black_box(&n);
                Ok(consume(View::Numeric(n)))
            }
        }
    };
}
fn reader_options() -> capnp::message::ReaderOptions {
    capnp::message::ReaderOptions {
        traversal_limit_in_words: Some(1024),
        nesting_limit: 16,
    }
}
capnum!(CNum, numeric, u32, "capnp_u32");
capnum!(CNumNarrow, numeric_narrow, u16, "capnp_u16");
pub struct COptionals;
impl Codec for COptionals {
    type Input<'a> = (u32, Option<u8>, Option<u16>, Option<u32>);
    const NAME: &'static str = "capnp";
    const KIND: Kind = Kind::Optionals;
    fn prepare(f: &Fixture) -> Self::Input<'_> {
        (f.id, f.a, f.b, f.c)
    }
    fn encode(v: &Self::Input<'_>, b: &mut Buffers<'_>) -> Result<usize> {
        let mut message = capnp::message::Builder::new(&mut b.cap);
        {
            let mut r = message.init_root::<cap::optionals::Builder>();
            r.set_id(v.0);
            r.set_has_a(v.1.is_some());
            r.set_a(v.1.unwrap_or(0));
            r.set_has_b(v.2.is_some());
            r.set_b(v.2.unwrap_or(0));
            r.set_has_c(v.3.is_some());
            r.set_c(v.3.unwrap_or(0));
        }
        let mut dest = &mut b.out[..];
        capnp::serialize::write_message(&mut dest, &message)?;
        Ok(CAPACITY - dest.len())
    }
    fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
        let mut rest = input;
        let message =
            capnp::serialize::read_message_from_flat_slice_no_alloc(&mut rest, reader_options())?;
        exact(input.len() - rest.len(), input)?;
        let r = message.get_root::<cap::optionals::Reader>()?;
        let v = (
            r.get_id(),
            r.get_has_a().then_some(r.get_a()),
            r.get_has_b().then_some(r.get_b()),
            r.get_has_c().then_some(r.get_c()),
        );
        black_box(&v);
        Ok(consume(View::Optionals(v.0, v.1, v.2, v.3)))
    }
}
pub struct CText;
pub struct CTextBorrow;
fn cap_text_encode(v: &(u32, &str), b: &mut Buffers<'_>) -> Result<usize> {
    text_valid(v.1)?;
    let mut message = capnp::message::Builder::new(&mut b.cap);
    {
        let mut r = message.init_root::<cap::text_record::Builder>();
        r.set_id(v.0);
        r.set_text(v.1);
    }
    let mut dest = &mut b.out[..];
    capnp::serialize::write_message(&mut dest, &message)?;
    Ok(CAPACITY - dest.len())
}
macro_rules! captext {
    ($name:ident,$borrow:literal) => {
        impl Codec for $name {
            type Input<'a> = (u32, &'a str);
            const NAME: &'static str = "capnp";
            const KIND: Kind = Kind::Text;
            const BORROWED: bool = $borrow;
            const WRITE: bool = !$borrow;
            fn prepare(f: &Fixture) -> Self::Input<'_> {
                (f.id, &f.text)
            }
            fn encode(v: &Self::Input<'_>, b: &mut Buffers<'_>) -> Result<usize> {
                cap_text_encode(v, b)
            }
            fn read<R>(input: &[u8], consume: impl FnOnce(View<'_>) -> R) -> Result<R> {
                let mut rest = input;
                let message = capnp::serialize::read_message_from_flat_slice_no_alloc(
                    &mut rest,
                    reader_options(),
                )?;
                exact(input.len() - rest.len(), input)?;
                let r = message.get_root::<cap::text_record::Reader>()?;
                let text = r.get_text()?.to_str()?;
                text_valid(text)?;
                if $borrow {
                    Ok(consume(View::Text(r.get_id(), text)))
                } else {
                    let owned = text.to_owned();
                    black_box(&owned);
                    Ok(consume(View::Text(r.get_id(), &owned)))
                }
            }
        }
    };
}
captext!(CText, false);
captext!(CTextBorrow, true);
