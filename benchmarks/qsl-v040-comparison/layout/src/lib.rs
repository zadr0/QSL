pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
}

pub const HOT_COUNT: usize = 256;
pub const LARGE_COUNT: usize = 1 << 18;
pub const FRAME_SIZE: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fields {
    pub flag: u8,
    pub count: u64,
    pub id: u16,
    pub time: u32,
    pub quality: u8,
}
impl Fields {
    pub fn new(index: usize) -> Self {
        let count = (index as u64)
            .wrapping_mul(0xd6e8_feb8_6659_fd93)
            .rotate_left(23);
        Self {
            flag: (count >> 16) as u8,
            count,
            id: (count >> 32) as u16,
            time: (count >> 8) as u32,
            quality: (count >> 56) as u8,
        }
    }
    #[inline]
    pub fn digest(self) -> u64 {
        self.count
            ^ (self.time as u64).rotate_left(19)
            ^ (self.id as u64).rotate_left(37)
            ^ (self.flag as u64).rotate_left(53)
            ^ self.quality as u64
    }
}
pub trait Layout: Sized {
    const NAME: &'static str;
    const OFFSETS: [usize; 5];
    fn from_fields(value: Fields) -> Self;
    fn fields(&self) -> Fields;
    fn serialize(&self, frame: &mut [u8]) -> usize;
    fn deserialize(frame: &[u8]) -> Self;
}
macro_rules! layout {
    ($name:ident, $label:literal) => {
        impl Layout for generated::$name {
            const NAME: &'static str = $label;
            const OFFSETS: [usize; 5] = [
                std::mem::offset_of!(Self, flag),
                std::mem::offset_of!(Self, count),
                std::mem::offset_of!(Self, id),
                std::mem::offset_of!(Self, time),
                std::mem::offset_of!(Self, quality),
            ];
            #[inline]
            fn from_fields(v: Fields) -> Self {
                Self {
                    flag: v.flag,
                    count: v.count,
                    id: v.id,
                    time: v.time,
                    quality: v.quality,
                }
            }
            #[inline]
            fn fields(&self) -> Fields {
                Fields {
                    flag: self.flag,
                    count: self.count,
                    id: self.id,
                    time: self.time,
                    quality: self.quality,
                }
            }
            #[inline]
            fn serialize(&self, frame: &mut [u8]) -> usize {
                self.serialize_checked_to(frame).unwrap()
            }
            #[inline]
            fn deserialize(frame: &[u8]) -> Self {
                Self::deserialize_checked(frame).unwrap()
            }
        }
    };
}
layout!(Declared, "declared");
layout!(Auto, "auto");
layout!(Canonical, "auto_wire_1");
layout!(Manual, "manual");
pub fn example_frame<T: Layout>(fields: Fields) -> [u8; FRAME_SIZE] {
    let mut frame = [0; FRAME_SIZE];
    assert_eq!(T::from_fields(fields).serialize(&mut frame), FRAME_SIZE);
    frame
}
pub fn validate<T: Layout>() {
    for i in 0..HOT_COUNT {
        let fields = Fields::new(i);
        let frame = example_frame::<T>(fields);
        assert_eq!(T::deserialize(&frame).fields(), fields);
        assert_eq!(
            example_frame::<generated::Declared>(fields),
            example_frame::<generated::Auto>(fields)
        );
    }
}
