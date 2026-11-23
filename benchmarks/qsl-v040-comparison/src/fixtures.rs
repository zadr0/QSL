use std::error::Error;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
pub const FIXTURES: usize = 256;
pub const CAPACITY: usize = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Numeric,
    Optionals,
    Text,
}
impl Kind {
    pub fn name(self, len: usize) -> String {
        match self {
            Self::Numeric => "numeric".into(),
            Self::Optionals => "optionals".into(),
            Self::Text => format!("text_{len}"),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Fixture {
    pub numbers: [u32; 4],
    pub id: u32,
    pub a: Option<u8>,
    pub b: Option<u16>,
    pub c: Option<u32>,
    pub text: String,
}
#[derive(Debug, PartialEq, Eq)]
pub enum View<'a> {
    Numeric([u32; 4]),
    Optionals(u32, Option<u8>, Option<u16>, Option<u32>),
    Text(u32, &'a str),
}
impl View<'_> {
    pub fn digest(&self) -> u64 {
        match self {
            Self::Numeric(n) => n
                .iter()
                .fold(0x9e3779b97f4a7c15, |h, &v| h.rotate_left(13) ^ u64::from(v)),
            Self::Optionals(id, a, b, c) => {
                u64::from(*id).wrapping_mul(0x9e3779b97f4a7c15)
                    ^ a.map_or(0, |v| 0x100 | u64::from(v)).rotate_left(7)
                    ^ b.map_or(0, |v| 0x10000 | u64::from(v)).rotate_left(19)
                    ^ c.map_or(0, |v| 0x100000000 | u64::from(v)).rotate_left(31)
            }
            Self::Text(id, text) => u64::from(*id) ^ xxhash_rust::xxh3::xxh3_64(text.as_bytes()),
        }
    }
}
impl Fixture {
    pub fn new(i: usize, text_len: usize) -> Self {
        let mut seed = (i as u32).wrapping_add(0xa381d9ef);
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed
        };
        let numbers = std::array::from_fn(|_| next() & 4095);
        let id = next();
        let c = next();
        let mut text = String::with_capacity(text_len);
        let prefix = format!("{i:08x}|");
        for ch in prefix
            .chars()
            .chain("QSL Δ測🙂 telemetry абв 0123456789 ".chars().cycle())
        {
            if text.len() + ch.len_utf8() > text_len {
                break;
            }
            text.push(ch);
        }
        while text.len() < text_len {
            text.push('x')
        }
        Self {
            numbers,
            id,
            a: (i & 1 != 0).then_some(if i & 8 == 0 { 0 } else { i as u8 }),
            b: (i & 2 != 0).then_some(if i & 8 == 0 { 0 } else { (i * 257) as u16 }),
            c: (i & 4 != 0).then_some(if i & 8 == 0 { 0 } else { c }),
            text,
        }
    }
    pub fn view(&self, kind: Kind) -> View<'_> {
        match kind {
            Kind::Numeric => View::Numeric(self.numbers),
            Kind::Optionals => View::Optionals(self.id, self.a, self.b, self.c),
            Kind::Text => View::Text(self.id, &self.text),
        }
    }
}
pub fn numeric_valid(n: [u32; 4]) -> Result<()> {
    if n.iter().any(|&v| v > 4095) {
        return Err("numeric domain exceeded".into());
    }
    Ok(())
}
pub fn text_valid(text: &str) -> Result<()> {
    if text.len() > 1024 {
        return Err("text domain exceeded".into());
    }
    Ok(())
}
pub fn exact(used: usize, input: &[u8]) -> Result<()> {
    if used != input.len() {
        return Err("trailing bytes".into());
    }
    Ok(())
}
