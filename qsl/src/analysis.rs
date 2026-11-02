//! Build-time bit-window analysis. No schema decisions are made by hot I/O.
use crate::{QslError, QslResult, QslSchema, QslType, FLAG_ALIGNMENT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitRegister {
    None,
    U8,
    U16,
    U32,
    U64,
    U128,
}
impl BitRegister {
    pub fn for_bits(bits: usize) -> QslResult<Self> {
        Ok(match bits {
            0 => Self::None,
            1..=8 => Self::U8,
            9..=16 => Self::U16,
            17..=32 => Self::U32,
            33..=64 => Self::U64,
            65..=128 => Self::U128,
            _ => return Err(QslError::InvalidWidth),
        })
    }
    pub fn bits(self) -> usize {
        match self {
            Self::None => 0,
            Self::U8 => 8,
            Self::U16 => 16,
            Self::U32 => 32,
            Self::U64 => 64,
            Self::U128 => 128,
        }
    }
    pub fn reader(self) -> &'static str {
        match self {
            Self::None => "::qsl::QslByteReader",
            Self::U8 => "::qsl::QslBitReader8",
            Self::U16 => "::qsl::QslBitReader16",
            Self::U32 => "::qsl::QslBitReader32",
            Self::U64 => "::qsl::QslBitReader64",
            Self::U128 => "::qsl::QslBitReader128",
        }
    }
    pub fn writer(self) -> &'static str {
        match self {
            Self::None => "::qsl::QslByteWriter",
            Self::U8 => "::qsl::QslBitWriter8",
            Self::U16 => "::qsl::QslBitWriter16",
            Self::U32 => "::qsl::QslBitWriter32",
            Self::U64 => "::qsl::QslBitWriter64",
            Self::U128 => "::qsl::QslBitWriter128",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct BitBlock {
    pub reader: BitRegister,
    pub writer: BitRegister,
}
#[derive(Debug)]
pub struct QslIoPlan {
    pub blocks: Vec<BitBlock>,
    pub field_blocks: Vec<usize>,
    /// A compact rank can be copied as exact LE bytes at this boundary.
    pub aligned_ranks: Vec<bool>,
}
struct Builder {
    offsets: u8,
    read: usize,
    write: usize,
    blocks: Vec<BitBlock>,
}
impl Builder {
    fn append(&mut self, width: usize) {
        let mut next = 0u8;
        for offset in 0..8 {
            if self.offsets & (1 << offset) == 0 {
                continue;
            }
            let remaining = (8 - offset) & 7;
            let loaded = width.saturating_sub(remaining).div_ceil(8) * 8;
            self.read = self.read.max(remaining + loaded);
            self.write = self.write.max(offset + width);
            next |= 1 << ((offset + width) & 7);
        }
        self.offsets = next;
    }
    fn choices(&mut self, widths: impl IntoIterator<Item = usize>) {
        let initial = self.offsets;
        let mut next = 0;
        for width in widths {
            self.offsets = initial;
            self.append(width);
            next |= self.offsets;
        }
        self.offsets = next;
    }
    fn end_block(&mut self) -> QslResult<()> {
        self.blocks.push(BitBlock {
            reader: BitRegister::for_bits(self.read)?,
            writer: BitRegister::for_bits(self.write)?,
        });
        self.read = 0;
        self.write = 0;
        self.offsets = 1;
        Ok(())
    }
}
impl QslIoPlan {
    pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
        let mut b = Builder {
            offsets: 1,
            read: 0,
            write: 0,
            blocks: Vec::new(),
        };
        if !schema.sequential() && !schema.bitmap() {
            match schema.optional_count() {
                0 => {}
                1 => b.append(1),
                n => {
                    b.append(schema.count_bits());
                    let start = b.offsets;
                    let old_read = b.read;
                    let old_write = b.write;
                    b.choices((0..=n).map(|count| count * schema.tag_bits()));
                    let reader_max = b.read;
                    b.offsets = start;
                    b.read = old_read;
                    b.write = old_write;
                    for _ in 0..n {
                        b.choices([0, schema.tag_bits()]);
                    }
                    b.read = b.read.max(reader_max);
                }
            }
            if !schema.compact() && schema.optional_count() != 0 {
                b.end_block()?;
            }
        }
        let mut fields = Vec::new();
        let mut aligned = Vec::new();
        for field in &schema.fields {
            fields.push(b.blocks.len());
            let before = b.offsets;
            let width = field.data.rank_width(field.bits)?;
            let rank = matches!(
                field.data,
                QslType::Byte
                    | QslType::Integer { .. }
                    | QslType::RangedInteger { .. }
                    | QslType::Flags(_)
            );
            let length = matches!(
                field.data,
                QslType::String(_)
                    | QslType::RangedString { .. }
                    | QslType::BinaryString { .. }
                    | QslType::GuardString { .. }
                    | QslType::CopyGuardString { .. }
            ) || matches!(&field.data, QslType::Array(element, _) | QslType::RangedArray {element, ..} if matches!(element.as_ref(), QslType::Byte));
            let native = schema.compact()
                && (rank || length)
                && before == 1
                && width != 0
                && width.is_multiple_of(8);
            aligned.push(native);
            if rank && schema.compact() && !native {
                b.append(width);
            } else if length && schema.compact() && !native {
                b.append(width);
                b.offsets = 1;
            } else if !matches!(field.data, QslType::None) {
                b.offsets = 1;
            }
            if field.optional {
                b.offsets |= before;
            }
            if schema.compact() && schema.flags & FLAG_ALIGNMENT != 0 {
                b.offsets = 1;
            }
            if b.offsets == 1 {
                b.end_block()?;
            }
        }
        if b.blocks.is_empty() || fields.last().is_some_and(|&index| index == b.blocks.len()) {
            b.end_block()?;
        }
        Ok(Self {
            blocks: b.blocks,
            field_blocks: fields,
            aligned_ranks: aligned,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bit_windows_and_byte_barriers_select_minimal_registers() {
        for (source, read, write) in [
            (
                "&A[compact]{u8(0..7) p;str(1..100) t|binary_opt=view|;u8(0..7) q;}",
                vec![16, 8],
                vec![16, 8],
            ),
            (
                "&A[compact]{u8(0..7) p;str t|binary_opt=view|;u16 q;}",
                vec![64, 0],
                vec![64, 0],
            ),
            (
                "&A[compact]{u8(0..7) p;u64 n;str(0..100) t|binary_opt=view|;u8(0..7) q;}",
                vec![128, 8],
                vec![128, 8],
            ),
            ("&A{str t;}", vec![0], vec![0]),
        ] {
            let schema = crate::parse_qsl(source).unwrap();
            let plan = QslIoPlan::analyze(&schema[0]).unwrap();
            assert_eq!(
                plan.blocks
                    .iter()
                    .map(|b| b.reader.bits())
                    .collect::<Vec<_>>(),
                read
            );
            assert_eq!(
                plan.blocks
                    .iter()
                    .map(|b| b.writer.bits())
                    .collect::<Vec<_>>(),
                write
            );
        }
    }
}
