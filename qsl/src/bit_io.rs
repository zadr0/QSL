//! Concrete bit accumulators selected by QslIoPlan during code generation.
//! The legacy runtime-width API remains available in io.rs.
use std::ptr;
/// Exact native loads for a partial register; no memcpy call or tail overread.
#[inline(always)]
unsafe fn load_tail<const BYTES: usize>(input: *const u8, n: usize) -> u64 {
    std::hint::assert_unchecked(n <= BYTES);
    match n {
        0 => 0,
        1 => input.read() as u64,
        2 => u16::from_le(ptr::read_unaligned(input.cast())) as u64,
        3 => {
            u16::from_le(ptr::read_unaligned(input.cast())) as u64
                | ((input.add(2).read() as u64) << 16)
        }
        4 => u32::from_le(ptr::read_unaligned(input.cast())) as u64,
        5 => {
            u32::from_le(ptr::read_unaligned(input.cast())) as u64
                | ((input.add(4).read() as u64) << 32)
        }
        6 => {
            u32::from_le(ptr::read_unaligned(input.cast())) as u64
                | ((u16::from_le(ptr::read_unaligned(input.add(4).cast())) as u64) << 32)
        }
        7 => {
            u32::from_le(ptr::read_unaligned(input.cast())) as u64
                | ((u16::from_le(ptr::read_unaligned(input.add(4).cast())) as u64) << 32)
                | ((input.add(6).read() as u64) << 48)
        }
        8 => u64::from_le(ptr::read_unaligned(input.cast())),
        _ => std::hint::unreachable_unchecked(),
    }
}
/// Exact native stores for a committed partial register.
#[inline(always)]
unsafe fn store_tail<const BYTES: usize>(output: *mut u8, value: u64, n: usize) {
    std::hint::assert_unchecked(n <= BYTES);
    match n {
        0 => {}
        1 => output.write(value as u8),
        2 => ptr::write_unaligned(output.cast(), (value as u16).to_le()),
        3 => {
            ptr::write_unaligned(output.cast(), (value as u16).to_le());
            output.add(2).write((value >> 16) as u8);
        }
        4 => ptr::write_unaligned(output.cast(), (value as u32).to_le()),
        5 => {
            ptr::write_unaligned(output.cast(), (value as u32).to_le());
            output.add(4).write((value >> 32) as u8);
        }
        6 => {
            ptr::write_unaligned(output.cast(), (value as u32).to_le());
            ptr::write_unaligned(output.add(4).cast(), ((value >> 32) as u16).to_le());
        }
        7 => {
            ptr::write_unaligned(output.cast(), (value as u32).to_le());
            ptr::write_unaligned(output.add(4).cast(), ((value >> 32) as u16).to_le());
            output.add(6).write((value >> 48) as u8);
        }
        8 => ptr::write_unaligned(output.cast(), value.to_le()),
        _ => std::hint::unreachable_unchecked(),
    }
}
macro_rules! byte_reader_methods {
    () => {
        /// Decode using a native length word fixed by schema analysis.
        /// # Safety
        /// The next frame is valid for MAX and H; output follows decode_into's contract.
        #[inline(always)]
        pub unsafe fn read_inline_string_into_with<const MAX: usize, H: crate::StringHeader>(
            &mut self,
            out: *mut crate::InlineString<MAX>,
        ) {
            self.align();
            let n = crate::InlineString::<MAX>::decode_into_with::<H>(self.ptr, out);
            self.ptr = self.ptr.add(n);
        }
        /// # Safety
        /// A valid raw-only frame for MAX and H remains; out follows decode_raw_into.
        #[inline(always)]
        pub unsafe fn read_raw_inline_string_into_with<const MAX: usize, H: crate::StringHeader>(
            &mut self,
            out: *mut crate::InlineString<MAX>,
        ) {
            self.align();
            let n = crate::InlineString::<MAX>::decode_raw_into_with::<H>(self.ptr, out);
            self.ptr = self.ptr.add(n);
        }
        /// # Safety
        /// A complete trusted frame with canonical MAX and H remains.
        #[inline(always)]
        pub unsafe fn read_inline_string_with<const MAX: usize, H: crate::StringHeader>(
            &mut self,
        ) -> crate::InlineString<MAX> {
            self.align();
            let (value, n) = crate::InlineString::<MAX>::decode_from_with::<H>(self.ptr);
            self.ptr = self.ptr.add(n);
            value
        }
        /// # Safety
        /// A complete trusted raw-only frame with canonical MAX and H remains.
        #[inline(always)]
        pub unsafe fn read_raw_inline_string_with<const MAX: usize, H: crate::StringHeader>(
            &mut self,
        ) -> crate::InlineString<MAX> {
            self.align();
            let (value, n) = crate::InlineString::<MAX>::decode_raw_from_with::<H>(self.ptr);
            self.ptr = self.ptr.add(n);
            value
        }
        #[inline(always)]
        pub fn as_ptr(&self) -> *const u8 {
            self.ptr
        }
        /// # Safety
        /// The next n schema-valid bytes are readable.
        #[inline(always)]
        pub unsafe fn read_exact(&mut self, n: usize) -> *const u8 {
            self.align();
            let p = self.ptr;
            self.ptr = self.ptr.add(n);
            p
        }
        /// # Safety
        /// One byte is readable.
        #[inline(always)]
        pub unsafe fn read_u8(&mut self) -> u8 {
            self.read_exact(1).read()
        }
        /// # Safety
        /// Two bytes are readable.
        #[inline(always)]
        pub unsafe fn read_u16(&mut self) -> u16 {
            u16::from_le(ptr::read_unaligned(self.read_exact(2).cast()))
        }
        /// # Safety
        /// Four bytes are readable.
        #[inline(always)]
        pub unsafe fn read_u32(&mut self) -> u32 {
            u32::from_le(ptr::read_unaligned(self.read_exact(4).cast()))
        }
        /// # Safety
        /// Eight bytes are readable.
        #[inline(always)]
        pub unsafe fn read_u64(&mut self) -> u64 {
            u64::from_le(ptr::read_unaligned(self.read_exact(8).cast()))
        }
        /// # Safety
        /// One byte is readable.
        #[inline(always)]
        pub unsafe fn read_i8(&mut self) -> i8 {
            self.read_u8() as i8
        }
        /// # Safety
        /// Two bytes are readable.
        #[inline(always)]
        pub unsafe fn read_i16(&mut self) -> i16 {
            self.read_u16() as i16
        }
        /// # Safety
        /// Four bytes are readable.
        #[inline(always)]
        pub unsafe fn read_i32(&mut self) -> i32 {
            self.read_u32() as i32
        }
        /// # Safety
        /// Eight bytes are readable.
        #[inline(always)]
        pub unsafe fn read_i64(&mut self) -> i64 {
            self.read_u64() as i64
        }
        /// # Safety
        /// A complete schema-valid UTF-8 frame for MAX is readable.
        #[inline(always)]
        pub unsafe fn read_inline_string<const MAX: usize>(&mut self) -> crate::InlineString<MAX> {
            self.align();
            let (v, n) = crate::InlineString::<MAX>::decode_from(self.ptr);
            self.ptr = self.ptr.add(n);
            v
        }
        /// # Safety
        /// Input is a valid frame and out obeys InlineString::decode_into.
        #[inline(always)]
        pub unsafe fn read_inline_string_into<const MAX: usize>(
            &mut self,
            out: *mut crate::InlineString<MAX>,
        ) {
            self.align();
            let n = crate::InlineString::<MAX>::decode_into(self.ptr, out);
            self.ptr = self.ptr.add(n);
        }
        /// # Safety
        /// Input is a valid raw frame and out obeys InlineString::decode_raw_into.
        #[inline(always)]
        pub unsafe fn read_raw_inline_string_into<const MAX: usize>(
            &mut self,
            out: *mut crate::InlineString<MAX>,
        ) {
            self.align();
            let n = crate::InlineString::<MAX>::decode_raw_into(self.ptr, out);
            self.ptr = self.ptr.add(n);
        }
    };
}
macro_rules! byte_writer_methods {
    () => {
        /// # Safety
        /// Raw-size capacity remains for MAX and H; source/output do not overlap.
        #[inline(always)]
        pub unsafe fn write_inline_string_with<const MAX: usize, H: crate::StringHeader>(
            &mut self,
            value: &crate::InlineString<MAX>,
        ) {
            self.flush_unchecked();
            let n = value.encode_to_with::<H>(self.ptr);
            self.ptr = self.ptr.add(n);
        }
        #[inline(always)]
        pub fn as_mut_ptr(&self) -> *mut u8 {
            self.ptr
        }
        /// # Safety
        /// n exclusive writable bytes remain.
        #[inline(always)]
        pub unsafe fn claim_bytes(&mut self, n: usize) -> *mut u8 {
            self.flush_unchecked();
            let p = self.ptr;
            self.ptr = self.ptr.add(n);
            p
        }
        /// # Safety
        /// The output holds bytes.len() nonoverlapping writable bytes.
        #[inline(always)]
        pub unsafe fn write_bytes_unchecked(&mut self, bytes: &[u8]) {
            self.write_ptr_unchecked(bytes.as_ptr(), bytes.len());
        }
        /// # Safety
        /// Source/output hold n nonoverlapping readable/writable bytes.
        #[inline(always)]
        pub unsafe fn write_ptr_unchecked(&mut self, source: *const u8, n: usize) {
            let to = self.claim_bytes(n);
            ptr::copy_nonoverlapping(source, to, n);
        }
        /// # Safety
        /// n exclusive writable bytes remain.
        #[inline(always)]
        pub unsafe fn write_zeros_unchecked(&mut self, n: usize) {
            let to = self.claim_bytes(n);
            ptr::write_bytes(to, 0, n);
        }
        /// # Safety
        /// Raw-size frame capacity remains; source/output do not overlap.
        #[inline(always)]
        pub unsafe fn write_inline_string<const MAX: usize>(
            &mut self,
            value: &crate::InlineString<MAX>,
        ) {
            self.flush_unchecked();
            let n = value.encode_to(self.ptr);
            self.ptr = self.ptr.add(n);
        }
    };
}
pub struct QslByteReader {
    ptr: *const u8,
}
impl QslByteReader {
    /// # Safety
    /// ptr addresses a complete trusted record.
    #[inline(always)]
    pub unsafe fn new(ptr: *const u8) -> Self {
        Self { ptr }
    }
    #[inline(always)]
    pub fn align(&mut self) {}
    /// # Safety
    /// Only a zero-width field is permitted in a byte-only cursor.
    #[inline(always)]
    pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
        std::hint::assert_unchecked(width == 0);
        0
    }
    byte_reader_methods!();
}
pub struct QslByteWriter {
    ptr: *mut u8,
}
impl QslByteWriter {
    /// # Safety
    /// ptr addresses sufficient exclusive writable record capacity.
    #[inline(always)]
    pub unsafe fn from_raw(ptr: *mut u8) -> Self {
        Self { ptr }
    }
    #[inline(always)]
    pub fn flush_unchecked(&mut self) {}
    /// # Safety
    /// Only a zero-width field is permitted in a byte-only cursor.
    #[inline(always)]
    pub unsafe fn write_bits_unchecked(&mut self, _value: u64, width: u32) {
        std::hint::assert_unchecked(width == 0);
    }
    byte_writer_methods!();
}
macro_rules! bit_cursors {
    ($reader:ident,$writer:ident,$word:ty,$wide:ty,$load:ty,$bytes:expr) => {
        pub struct $reader {
            ptr: *const u8,
            reg: $word,
            remaining: u8,
        }
        impl $reader {
            /// # Safety
            /// ptr addresses a complete trusted record.
            #[inline(always)]
            pub unsafe fn new(ptr: *const u8) -> Self {
                Self {
                    ptr,
                    reg: 0,
                    remaining: 0,
                }
            }
            /// # Safety
            /// Padding is valid and the current input cursor remains accessible.
            #[inline(always)]
            pub unsafe fn align(&mut self) {
                self.reg = 0;
                self.remaining = 0;
            }
            /// # Safety
            /// width<=64; the buffered/new bits fit this register; bytes consumed are readable.
            #[inline(always)]
            pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
                std::hint::assert_unchecked(width <= 64 && self.remaining < 8);
                let n = ((width.saturating_sub(self.remaining as u32) + 7) >> 3) as usize;
                let total = self.remaining as u32 + n as u32 * 8;
                std::hint::assert_unchecked(total <= <$word>::BITS);
                let loaded = load_tail::<$bytes>(self.ptr, n);
                self.ptr = self.ptr.add(n);
                self.reg |= (loaded as $word) << self.remaining;
                let value = ((self.reg as $wide) & (((1 as $wide) << width) - 1)) as u64;
                self.reg = ((self.reg as $wide) >> width) as $word;
                self.remaining = (total - width) as u8;
                value
            }
            byte_reader_methods!();
        }
        pub struct $writer {
            ptr: *mut u8,
            reg: $word,
            used: u8,
        }
        impl $writer {
            /// # Safety
            /// ptr addresses sufficient exclusive writable record capacity.
            #[inline(always)]
            pub unsafe fn from_raw(ptr: *mut u8) -> Self {
                Self {
                    ptr,
                    reg: 0,
                    used: 0,
                }
            }
            /// # Safety
            /// width<=64, used+width fits this register, and output capacity is sufficient.
            #[inline(always)]
            pub unsafe fn write_bits_unchecked(&mut self, value: u64, width: u32) {
                std::hint::assert_unchecked(width <= 64 && self.used < 8);
                let total = self.used as u32 + width;
                std::hint::assert_unchecked(total <= <$word>::BITS);
                let value = ((value as $wide) & (((1 as $wide) << width) - 1)) as $word;
                self.reg |= value << self.used;
                let n = (total >> 3) as usize;
                store_tail::<$bytes>(self.ptr, self.reg as u64, n);
                self.ptr = self.ptr.add(n);
                self.reg = ((self.reg as $wide) >> (n * 8)) as $word;
                self.used = (total & 7) as u8;
            }
            /// # Safety
            /// The output includes the trailing byte if any bits remain.
            #[inline(always)]
            pub unsafe fn flush_unchecked(&mut self) {
                let n = usize::from(self.used != 0);
                let value = self.reg as u8;
                ptr::copy_nonoverlapping(&value, self.ptr, n);
                self.ptr = self.ptr.add(n);
                self.reg = 0;
                self.used = 0;
            }
            byte_writer_methods!();
        }
    };
}
bit_cursors!(QslBitReader8, QslBitWriter8, u8, u16, u8, 1);
bit_cursors!(QslBitReader16, QslBitWriter16, u16, u32, u16, 2);
bit_cursors!(QslBitReader32, QslBitWriter32, u32, u64, u32, 4);
bit_cursors!(QslBitReader64, QslBitWriter64, u64, u128, u64, 8);
bit_cursors!(QslBitReader128, QslBitWriter128, u128, u128, u64, 8);

#[cfg(test)]
mod tests {
    #[test]
    fn every_register_matches_legacy_wire_and_exact_tail() {
        macro_rules! check {
            ($r:ident,$w:ident,$bits:expr) => {
                for prefix in 0..8u32 {
                    for width in 0..=64u32 {
                        if prefix + width > $bits {
                            continue;
                        }
                        let value = if width == 64 {
                            u64::MAX
                        } else {
                            (1u64 << width) - 1
                        };
                        let mut old = [0xA7u8; 16];
                        let mut new = [0xA7u8; 16];
                        unsafe {
                            let mut before = crate::QslUncheckedWriter::from_raw(old.as_mut_ptr());
                            before.write_bits_unchecked(0, prefix);
                            before.write_bits_unchecked(value, width);
                            before.flush_unchecked();
                            let mut after = super::$w::from_raw(new.as_mut_ptr());
                            after.write_bits_unchecked(0, prefix);
                            after.write_bits_unchecked(value, width);
                            after.flush_unchecked();
                            let n = ((prefix + width + 7) >> 3) as usize;
                            assert_eq!(old, new);
                            assert_eq!(new[n], 0xA7);
                            let mut reader = super::$r::new(new.as_ptr());
                            assert_eq!(reader.read_bits(prefix), 0);
                            assert_eq!(reader.read_bits(width), value);
                        }
                    }
                }
            };
        }
        check!(QslBitReader8, QslBitWriter8, 8);
        check!(QslBitReader16, QslBitWriter16, 16);
        check!(QslBitReader32, QslBitWriter32, 32);
        check!(QslBitReader64, QslBitWriter64, 64);
        check!(QslBitReader128, QslBitWriter128, 128);
    }
}
