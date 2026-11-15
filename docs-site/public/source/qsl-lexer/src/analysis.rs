//! Build-time bit-window analysis. No schema decisions are made by hot I/O.
mod compilation;
mod regions;
use crate::{QslError, QslResult, QslSchema, QslType, FLAG_ALIGNMENT};
pub use compilation::{CompilationPlan, FieldPlan, InitExtent, Operation};
pub use regions::{RegionKind, WirePlan, WireRegion};

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
/// Build-time bit windows inside an array field, including its length header.
/// Offset bit n means the cursor can enter with n used bits in its byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArrayWindow {
    pub field: usize,
    pub entry_offsets: u8,
    pub reader: BitRegister,
    pub writer: BitRegister,
}
#[derive(Clone, Debug)]
pub struct QslIoPlan {
    pub blocks: Vec<BitBlock>,
    pub field_blocks: Vec<usize>,
    /// A compact rank can be copied as exact LE bytes at this boundary.
    pub aligned_ranks: Vec<bool>,
    pub array_windows: Vec<ArrayWindow>,
    pub pointing: Option<crate::optimizer::PointingPlan>,
    pub native_checked_into: bool,
    /// Measured four-lane PEXT writer; consumer cfg-gated. The reader stays scalar.
    pub fixed_bmi2_write: bool,
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
    fn scalar(&mut self, width: usize) {
        self.append(width.min(64));
        if width > 64 {
            self.append(width - 64);
        }
    }
    /// Compute a loop's reachable residues, never iterate over runtime capacity.
    fn repeat(
        &mut self,
        element: &QslType,
        count: usize,
        exact: bool,
        depth: usize,
    ) -> QslResult<()> {
        if exact {
            let mut seen = [None; 256];
            let mut i = 0;
            while i < count {
                let state = self.offsets as usize;
                if let Some(previous) = seen[state] {
                    let period = i - previous;
                    let skip = (count - i) / period * period;
                    i += skip;
                    if i == count {
                        break;
                    }
                }
                seen[state] = Some(i);
                self.value(element, depth)?;
                i += 1;
            }
        } else {
            for _ in 0..count {
                let before = self.offsets;
                self.value(element, depth)?;
                self.offsets |= before;
                if self.offsets == before {
                    break;
                }
            }
        }
        Ok(())
    }
    fn array(
        &mut self,
        ty: &QslType,
        width: usize,
        aligned_header: bool,
        depth: usize,
    ) -> QslResult<()> {
        let (element, count, exact) = match ty {
            QslType::Array(e, _) => (e.as_ref(), usize::MAX, false),
            QslType::RangedArray { element, max, .. } => (element.as_ref(), *max, false),
            QslType::FixedArray(e, n) => (e.as_ref(), *n, true),
            _ => return Err(QslError::InvalidSchema),
        };
        if !exact && !aligned_header {
            self.append(width);
        }
        if matches!(element, QslType::Byte) {
            // Byte-copy codec flushes the length prefix before its payload.
            // Native array storage may consume these bytes through read_bits(8).
            self.offsets = 1;
            if count != 0 {
                self.append(8);
            }
        } else {
            self.repeat(element, count, exact, depth + 1)?;
        }
        // Array payloads finish on a byte boundary in the existing wire codec.
        self.offsets = 1;
        Ok(())
    }
    fn value(&mut self, ty: &QslType, depth: usize) -> QslResult<()> {
        if depth > 64 {
            return Err(QslError::InvalidSchema);
        }
        match ty {
            QslType::Byte
            | QslType::Integer { .. }
            | QslType::RangedInteger { .. }
            | QslType::DomainInteger(_)
            | QslType::Flags(_) => self.scalar(ty.default_compact_bits()?),
            QslType::Array(..) | QslType::RangedArray { .. } | QslType::FixedArray(..) => {
                self.array(ty, ty.default_compact_bits()?, false, depth)?;
            }
            QslType::String(_) | QslType::RangedString { .. } | QslType::BorrowedString { .. } => {
                self.append(ty.default_compact_bits()?);
                self.offsets = 1;
            }
            QslType::BitPadding(n) => {
                if *n >= 64 {
                    self.append(64);
                }
                if n % 64 != 0 {
                    self.append(n % 64);
                }
            }
            QslType::None => {}
            _ => self.offsets = 1,
        }
        Ok(())
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
        let memory = crate::optimizer::MemoryPlan::analyze(schema)?;
        Self::with_memory(schema, &memory)
    }
    pub fn with_memory(
        schema: &QslSchema,
        memory: &crate::optimizer::MemoryPlan,
    ) -> QslResult<Self> {
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
                    if n * schema.tag_bits() <= 64 {
                        b.choices((0..=n).map(|count| count * schema.tag_bits()));
                    } else {
                        for _ in 0..n {
                            b.choices([0, schema.tag_bits()]);
                        }
                    }
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
        let mut array_windows = Vec::new();
        for field in &schema.fields {
            fields.push(b.blocks.len());
            let before = b.offsets;
            let width = field.data.rank_width(field.bits)?;
            let rank = matches!(
                field.data,
                QslType::Byte
                    | QslType::Integer { .. }
                    | QslType::RangedInteger { .. }
                    | QslType::DomainInteger(_)
                    | QslType::Flags(_)
            );
            let length = matches!(
                field.data,
                QslType::String(_) | QslType::RangedString { .. } | QslType::BorrowedString { .. }
            ) || matches!(&field.data, QslType::Array(element, _) | QslType::RangedArray {element, ..} if element.fixed_wire_size().ok().flatten().is_some());
            let array = matches!(
                field.data,
                QslType::Array(..) | QslType::RangedArray { .. } | QslType::FixedArray(..)
            );
            let numeric_array = matches!(&field.data, QslType::Array(e, _) | QslType::RangedArray { element: e, .. } | QslType::FixedArray(e, _) if !matches!(e.as_ref(), QslType::Byte));
            let native = schema.compact()
                && !numeric_array
                && !matches!(field.data, QslType::DomainInteger(_))
                && (rank || length)
                && before == 1
                && width != 0
                && width.is_multiple_of(8);
            aligned.push(native);
            if array && schema.compact() {
                let mut window = Builder {
                    offsets: before,
                    read: 0,
                    write: 0,
                    blocks: Vec::new(),
                };
                window.array(&field.data, width, native, 0)?;
                array_windows.push(ArrayWindow {
                    field: fields.len() - 1,
                    entry_offsets: before,
                    reader: BitRegister::for_bits(window.read)?,
                    writer: BitRegister::for_bits(window.write)?,
                });
                b.read = b.read.max(window.read);
                b.write = b.write.max(window.write);
                b.offsets = window.offsets;
            } else if let QslType::BitPadding(n) = field.data {
                if n >= 64 {
                    b.append(64);
                }
                if n % 64 != 0 {
                    b.append(n % 64);
                }
            } else if rank && schema.compact() && !native {
                b.scalar(width);
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
            array_windows,
            pointing: crate::optimizer::PointingPlan::with_memory(schema, memory)?,
            native_checked_into: native_checked_into(schema),
            fixed_bmi2_write: false,
        })
    }
}

fn native_checked_into(s: &QslSchema) -> bool {
    use crate::layout::StorageKind;
    fn native(t: &QslType) -> bool {
        match t {
            QslType::Byte
            | QslType::Integer { .. }
            | QslType::RangedInteger { .. }
            | QslType::DomainInteger(_)
            | QslType::Flags(_)
            | QslType::FixedString(_)
            | QslType::None
            | QslType::Padding(_) => true,
            QslType::FixedArray(e, _) => native(e),
            _ => false,
        }
    }
    s.optional_count() == 0
        && !s.qps()
        && s.options.dynamic.is_none()
        && !s.options.transmutate
        && !s.options.native_overlay()
        && s.options.regions.is_empty()
        && s.fields.iter().any(|f| {
            matches!(f.storage, StorageKind::StackString(_))
                || matches!(
                    f.data,
                    QslType::FixedString(_)
                        | QslType::CompressedString { .. }
                        | QslType::InlineRawString { .. }
                )
        })
        && s.fields.iter().all(|f| match f.storage {
            StorageKind::StackString(_) | StorageKind::StrView => true,
            StorageKind::FixedArray(_) => native(&f.data),
            StorageKind::Default => {
                native(&f.data)
                    || matches!(
                        f.data,
                        QslType::CompressedString { .. } | QslType::InlineRawString { .. }
                    )
                    || s.borrow_strings()
                        && matches!(
                            f.data,
                            QslType::String(_)
                                | QslType::RangedString { .. }
                                | QslType::BorrowedString { .. }
                        )
            }
            _ => false,
        })
}

#[cfg(test)]
mod array_tests {
    use super::*;
    fn window(offset: usize) -> Builder {
        Builder {
            offsets: 1 << offset,
            read: 0,
            write: 0,
            blocks: Vec::new(),
        }
    }
    #[test]
    fn repeated_numeric_windows_match_each_small_reference_loop() {
        for width in 0..=128u16 {
            let domain =
                crate::domain::IntegerDomain::unsigned(128, 0, crate::domain::low_mask(width))
                    .unwrap();
            let element = QslType::DomainInteger(domain.into());
            for offset in 0..8 {
                for count in 0..=24 {
                    let mut plan = window(offset);
                    plan.repeat(&element, count, true, 0).unwrap();
                    let mut at = offset;
                    let mut read = 0;
                    let mut write = 0;
                    for _ in 0..count {
                        for chunk in [
                            usize::from(width).min(64),
                            usize::from(width).saturating_sub(64),
                        ]
                        .into_iter()
                        .take(if width > 64 { 2 } else { 1 })
                        {
                            let available = (8 - at) % 8;
                            let needed = chunk.saturating_sub(available);
                            let bytes = needed.div_ceil(8);
                            read = read.max(available + bytes * 8);
                            write = write.max(at + chunk);
                            at = (at + chunk) % 8;
                        }
                    }
                    assert_eq!(
                        (plan.offsets, plan.read, plan.write),
                        (1 << at, read, write),
                        "width={width} offset={offset} count={count}"
                    );
                }
            }
        }
    }
    #[test]
    fn huge_array_analysis_terminates_and_preserves_residue() {
        let element = QslType::Flags(3);
        let mut b = window(5);
        b.repeat(&element, usize::MAX, true, 0).unwrap();
        assert_eq!(b.offsets, 1 << ((5 + (usize::MAX % 8) * 3) % 8));
        assert_eq!(BitRegister::for_bits(b.write).unwrap(), BitRegister::U16);
        let mut b = window(0);
        b.repeat(&element, usize::MAX, false, 0).unwrap();
        assert_eq!(b.offsets, 255);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn bit_windows_and_byte_barriers_select_minimal_registers() {
        for (source, read, write) in [
            (
                "&A[compact,lifetime]{u8(0..7) p;str(1..100) t;u8(0..7) q;}",
                vec![16, 8],
                vec![16, 8],
            ),
            (
                "&A[compact,lifetime]{u8(0..7) p;str t;u16 q;}",
                vec![64, 0],
                vec![64, 0],
            ),
            (
                "&A[compact,lifetime]{u8(0..7) p;u64 n;str(0..100) t;u8(0..7) q;}",
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
