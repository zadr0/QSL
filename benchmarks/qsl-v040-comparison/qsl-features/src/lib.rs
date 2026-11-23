use std::hint::black_box;
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
}
pub const RECORDS: usize = 256;
pub const TEXT_LEN: usize = 1024;
pub const BUFFER: usize = 8192;
pub const CORPORA: [&str; 3] = ["periodic_16", "structured_log", "noisy_ascii"];

pub fn text(corpus: &str, id: usize) -> String {
    if corpus == "periodic_16" {
        let token = format!("tick={:04};ok=yes", id % 10_000);
        assert_eq!(token.len(), 16);
        return token.repeat(TEXT_LEN / token.len());
    }
    let mut text = format!("trace-{id:08x} ");
    if corpus == "structured_log" {
        let message = "sensor online; station=west; queue=short; mode=normal\n";
        while text.len() < TEXT_LEN {
            text.push_str(message);
        }
        text.truncate(TEXT_LEN);
    } else {
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut state = (id as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        for _ in text.len()..TEXT_LEN {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            text.push(alphabet[(state as usize) & 63] as char);
        }
    }
    text
}
pub trait TextCodec: Sized {
    const NAME: &'static str;
    fn prepare(id: u32, text: String) -> Self;
    fn write(&self, buffer: &mut [u8]) -> usize;
    fn read(input: &[u8]) -> Self;
    fn fields(&self) -> (u32, &str);
    fn digest(&self) -> u64 {
        let (id, text) = self.fields();
        u64::from(id) ^ xxhash_rust::xxh3::xxh3_64(text.as_bytes())
    }
}
macro_rules! string_codec {
    ($name:ident, $label:literal, $make:expr) => {
        impl TextCodec for generated::$name {
            const NAME: &'static str = $label;
            fn prepare(id: u32, text: String) -> Self {
                Self {
                    id,
                    text: ($make)(text),
                }
            }
            fn write(&self, buffer: &mut [u8]) -> usize {
                self.serialize_checked_to(buffer).unwrap()
            }
            fn read(input: &[u8]) -> Self {
                Self::deserialize_checked(input).unwrap()
            }
            fn fields(&self) -> (u32, &str) {
                (self.id, self.text.as_str())
            }
        }
    };
}
string_codec!(HeapText, "string", |text| text);
string_codec!(StackText, "stack_string", |text: String| {
    qsl::StackString::try_from_str(&text).unwrap()
});
string_codec!(CompressedText, "qsl_lz", |text: String| {
    qsl::InlineString::try_from_str(&text).unwrap()
});

#[repr(align(8))]
pub struct Aligned(pub [u8; 16]);
#[repr(align(8))]
pub struct Unaligned(pub [u8; 17]);
pub const NATIVE_CASES: [&str; 4] = [
    "fields",
    "copy_aligned",
    "copy_unaligned",
    "overlay_aligned",
];
pub fn native_frames() -> (Vec<Aligned>, Vec<Unaligned>) {
    assert!(generated::Native::NATIVE_WIRE_COMPATIBLE);
    assert!(generated::NativeCopy::NATIVE_WIRE_COMPATIBLE);
    assert_eq!(std::mem::size_of::<generated::Native>(), 16);
    (0..RECORDS)
        .map(|i| {
            let value = generated::Plain {
                timestamp: 1_790_000_000_000 + i as u64,
                counter: (i as u32).wrapping_mul(0x5533aa77),
                region: (i as u16) * 7,
                code: (i % 19) as u8,
                flags: (i % 7) as u8,
            };
            let mut aligned = Aligned([0; 16]);
            let n = value.serialize_checked_to(&mut aligned.0).unwrap();
            assert_eq!(n, 16);
            let mut unaligned = Unaligned([0; 17]);
            unaligned.0[1..].copy_from_slice(&aligned.0);
            let expected = native_read("fields", &aligned.0, &unaligned.0[1..]);
            for name in NATIVE_CASES {
                assert_eq!(native_read(name, &aligned.0, &unaligned.0[1..]), expected);
            }
            assert!(generated::Native::deserialize_overlay(&unaligned.0[1..]).is_err());
            (aligned, unaligned)
        })
        .unzip()
}

macro_rules! digest {
    ($v:expr) => {
        $v.timestamp
            ^ u64::from($v.counter).rotate_left(17)
            ^ u64::from($v.region).rotate_left(37)
            ^ u64::from($v.code).rotate_left(41)
            ^ u64::from($v.flags).rotate_left(53)
    };
}
pub fn fields(input: &[u8]) -> u64 {
    let value = generated::Plain::deserialize_checked(input).unwrap();
    black_box(&value);
    digest!(value)
}
pub fn copied(input: &[u8]) -> u64 {
    let value = generated::NativeCopy::deserialize_transmutate_copy(input).unwrap();
    black_box(&value);
    digest!(value)
}
pub fn overlay(input: &[u8]) -> u64 {
    let value = generated::Native::deserialize_overlay(input).unwrap();
    black_box(value);
    digest!(value)
}
fn native_read(name: &str, aligned: &[u8], unaligned: &[u8]) -> u64 {
    match name {
        "fields" => fields(aligned),
        "copy_aligned" => copied(aligned),
        "copy_unaligned" => copied(unaligned),
        "overlay_aligned" => overlay(aligned),
        _ => unreachable!(),
    }
}
