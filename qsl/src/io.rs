use std::ptr;

#[cfg(feature = "diagnostics")]
#[path = "io_checked.rs"]
mod checked;
#[cfg(feature = "diagnostics")]
pub use checked::{QslCheckedBitsReader, QslCheckedBitsWriter, QslCheckedReader, QslCheckedWriter};

pub use qsl_lexer::io::{QslError, QslResult, QslSizer};

/// Minimal raw-pointer writer used by trusted, compile-time-validated schemas.
/// No bounds checks, dynamic allocation, loops, or error values exist here.
/// The caller is responsible for the complete writable allocation.
pub struct QslRawWriter {
    ptr: *mut u8,
}

impl QslRawWriter {
    /// # Safety
    /// `ptr` must be valid for the complete encoded value and exclusively writable.
    #[inline(always)]
    pub unsafe fn new(ptr: *mut u8) -> Self {
        Self { ptr }
    }

    /// Construct a pointer cursor over an existing allocation. All subsequent
    /// writes remain unsafe because this cursor intentionally stores no length.
    #[inline(always)]
    pub fn from(out: &mut [u8]) -> Self {
        Self {
            ptr: out.as_mut_ptr(),
        }
    }

    /// Store one byte and advance the cursor by one.
    /// # Safety
    /// At least one writable byte must remain.
    #[inline(always)]
    pub unsafe fn write_u8(&mut self, value: u8) {
        ptr::write(self.ptr, value);
        self.ptr = self.ptr.add(1);
    }
    /// Store an unaligned little-endian `u16` and advance by two.
    /// # Safety
    /// At least two writable bytes must remain.
    #[inline(always)]
    pub unsafe fn write_u16(&mut self, value: u16) {
        ptr::write_unaligned(self.ptr.cast::<u16>(), value.to_le());
        self.ptr = self.ptr.add(2);
    }
    /// Store an unaligned little-endian `u32` and advance by four.
    /// # Safety
    /// At least four writable bytes must remain.
    #[inline(always)]
    pub unsafe fn write_u32(&mut self, value: u32) {
        ptr::write_unaligned(self.ptr.cast::<u32>(), value.to_le());
        self.ptr = self.ptr.add(4);
    }
    /// Store an unaligned little-endian `u64` and advance by eight.
    /// # Safety
    /// At least eight writable bytes must remain.
    #[inline(always)]
    pub unsafe fn write_u64(&mut self, value: u64) {
        ptr::write_unaligned(self.ptr.cast::<u64>(), value.to_le());
        self.ptr = self.ptr.add(8);
    }
    /// Copy `len` nonoverlapping bytes into the buffer and advance once.
    /// # Safety
    /// Source and destination must be valid and nonoverlapping for `len` bytes.
    #[inline(always)]
    pub unsafe fn write(&mut self, source: *const u8, len: usize) {
        ptr::copy_nonoverlapping(source, self.ptr, len);
        self.ptr = self.ptr.add(len);
    }
}

/// Minimal raw-pointer reader. Never checks the input length or its contents.
/// Any use with a truncated or invalid allocation is the caller's fault.
pub struct QslRawReader {
    ptr: *const u8,
}

impl QslRawReader {
    /// # Safety
    /// `ptr` must address the complete schema-encoded value.
    #[inline(always)]
    pub unsafe fn new(ptr: *const u8) -> Self {
        Self { ptr }
    }

    /// Create a raw cursor for an existing slice; reads remain unsafe.
    #[inline(always)]
    pub fn from(input: &[u8]) -> Self {
        Self {
            ptr: input.as_ptr(),
        }
    }
    /// Load exactly one byte with no conversion or temporary array.
    /// # Safety
    /// At least one readable byte must remain.
    #[inline(always)]
    pub unsafe fn read_u8(&mut self) -> u8 {
        let value = ptr::read(self.ptr);
        self.ptr = self.ptr.add(1);
        value
    }
    /// Load an unaligned little-endian `u16` and advance by two bytes.
    /// # Safety
    /// At least two readable bytes must remain.
    #[inline(always)]
    pub unsafe fn read_u16(&mut self) -> u16 {
        let value = u16::from_le(ptr::read_unaligned(self.ptr.cast::<u16>()));
        self.ptr = self.ptr.add(2);
        value
    }
    /// Load an unaligned little-endian `u32` and advance by four bytes.
    /// # Safety
    /// At least four readable bytes must remain.
    #[inline(always)]
    pub unsafe fn read_u32(&mut self) -> u32 {
        let value = u32::from_le(ptr::read_unaligned(self.ptr.cast::<u32>()));
        self.ptr = self.ptr.add(4);
        value
    }
    /// Load an unaligned little-endian `u64` and advance by eight bytes.
    /// # Safety
    /// At least eight readable bytes must remain.
    #[inline(always)]
    pub unsafe fn read_u64(&mut self) -> u64 {
        let value = u64::from_le(ptr::read_unaligned(self.ptr.cast::<u64>()));
        self.ptr = self.ptr.add(8);
        value
    }
    /// # Safety
    /// The returned source region must remain allocated for the consumer.
    #[inline(always)]
    pub unsafe fn take(&mut self, len: usize) -> *const u8 {
        let start = self.ptr;
        self.ptr = self.ptr.add(len);
        start
    }
}

/// Unchecked LSB-first bit reader for trusted records. It deliberately has no
/// length, Result, UTF-8, or padding checks. A u128 register moves
/// exactly 0..=8 bytes per operation without bit-by-bit loops or branches.
pub struct QslFastBitsReader {
    ptr: *const u8,
    reg: u128,
    remaining: u32,
}

impl QslFastBitsReader {
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

    /// Current byte cursor. Call align first after bit reads.
    #[inline(always)]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Decode a raw-only frame into final storage.
    /// # Safety
    /// The next frame is valid, raw mode, and output obeys decode_raw_into's contract.
    #[inline(always)]
    pub unsafe fn read_raw_inline_string_into<const MAX: usize>(
        &mut self,
        output: *mut crate::InlineString<MAX>,
    ) {
        self.align();
        let used = crate::InlineString::<MAX>::decode_raw_into(self.ptr, output);
        self.ptr = self.ptr.add(used);
    }
    /// Decode a frame straight into its final field storage.
    /// # Safety
    /// The next frame is valid; output satisfies InlineString::decode_into's contract.
    #[inline(always)]
    pub unsafe fn read_inline_string_into<const MAX: usize>(
        &mut self,
        output: *mut crate::InlineString<MAX>,
    ) {
        self.align();
        let used = crate::InlineString::<MAX>::decode_into(self.ptr, output);
        self.ptr = self.ptr.add(used);
    }
    /// Eagerly decode a bounded UTF-8 frame and advance by its stored length.
    /// # Safety
    /// The cursor must point to a complete valid frame for MAX.
    #[inline(always)]
    pub unsafe fn read_inline_string<const MAX: usize>(&mut self) -> crate::InlineString<MAX> {
        self.align();
        let (value, used) = crate::InlineString::<MAX>::decode_from(self.ptr);
        self.ptr = self.ptr.add(used);
        value
    }

    /// # Safety
    /// Pointer must refer to a complete valid encoded value. Methods may read
    /// arbitrary bytes from this pointer without bounds checking.
    #[inline(always)]
    pub unsafe fn new(ptr: *const u8) -> Self {
        Self {
            ptr,
            reg: 0,
            remaining: 0,
        }
    }

    /// Create an unchecked bit cursor over a slice; reading requires a valid
    /// complete record and is therefore unsafe.
    #[inline(always)]
    pub fn from(input: &[u8]) -> Self {
        Self {
            ptr: input.as_ptr(),
            reg: 0,
            remaining: 0,
        }
    }

    /// Extract `width` low-order bits from the current bitstream position.
    /// The decoded bit order is LSB-first and the requested width is static
    /// in ordinary generated QSL code.
    /// # Safety
    /// `width <= 64` and all encoded bytes consumed must be accessible.
    #[inline(always)]
    pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
        // At entry at most seven bits remain; the requested width is <= 64.
        // Read exactly 0..=8 new bytes into a zero-extended register.
        let count = ((width.saturating_sub(self.remaining) + 7) >> 3) as usize;
        let mut buf = [0u8; 8];
        ptr::copy_nonoverlapping(self.ptr, buf.as_mut_ptr(), count);
        self.ptr = self.ptr.add(count);
        self.reg |= (u64::from_le_bytes(buf) as u128) << self.remaining;
        self.remaining += (count * 8) as u32;
        let value = (self.reg & ((1u128 << width) - 1)) as u64;
        self.reg >>= width;
        self.remaining -= width;
        value
    }

    /// Discard the remainder of the current partial byte before byte I/O.
    /// # Safety
    /// This operation assumes schema-directed alignment at this position.
    #[inline(always)]
    pub unsafe fn align(&mut self) {
        self.remaining = 0;
        self.reg = 0;
    }

    /// Return a pointer to `len` consecutive bytes and advance without loading.
    /// # Safety
    /// The byte-aligned region must be readable for `len` bytes.
    #[inline(always)]
    pub unsafe fn read_exact(&mut self, len: usize) -> *const u8 {
        self.remaining = 0;
        self.reg = 0;
        let start = self.ptr;
        self.ptr = self.ptr.add(len);
        start
    }

    /// Read a byte directly; no slice construction, conversion, or loop.
    /// # Safety
    /// The cursor must have one accessible encoded byte.
    #[inline(always)]
    pub unsafe fn read_u8(&mut self) -> u8 {
        self.align();
        let value = ptr::read(self.ptr);
        self.ptr = self.ptr.add(1);
        value
    }

    /// Read an unaligned little-endian `u16` in a single pointer load.
    /// # Safety
    /// The cursor must have two accessible encoded bytes.
    #[inline(always)]
    pub unsafe fn read_u16(&mut self) -> u16 {
        self.align();
        let value = u16::from_le(ptr::read_unaligned(self.ptr.cast::<u16>()));
        self.ptr = self.ptr.add(2);
        value
    }

    /// Read an unaligned little-endian `u32` in a single pointer load.
    /// # Safety
    /// The cursor must have four accessible encoded bytes.
    #[inline(always)]
    pub unsafe fn read_u32(&mut self) -> u32 {
        self.align();
        let value = u32::from_le(ptr::read_unaligned(self.ptr.cast::<u32>()));
        self.ptr = self.ptr.add(4);
        value
    }

    /// Read an unaligned little-endian `u64` in a single pointer load.
    /// # Safety
    /// The cursor must have eight accessible encoded bytes.
    #[inline(always)]
    pub unsafe fn read_u64(&mut self) -> u64 {
        self.align();
        let value = u64::from_le(ptr::read_unaligned(self.ptr.cast::<u64>()));
        self.ptr = self.ptr.add(8);
        value
    }
    /// Read the bit pattern of one signed byte without range checks.
    /// # Safety
    /// One readable byte must remain.
    #[inline(always)]
    pub unsafe fn read_i8(&mut self) -> i8 {
        self.read_u8() as i8
    }
    /// Read a two-byte signed value in little-endian wire order.
    /// # Safety
    /// Two readable bytes must remain.
    #[inline(always)]
    pub unsafe fn read_i16(&mut self) -> i16 {
        self.read_u16() as i16
    }
    /// Read a four-byte signed value in little-endian wire order.
    /// # Safety
    /// Four readable bytes must remain.
    #[inline(always)]
    pub unsafe fn read_i32(&mut self) -> i32 {
        self.read_u32() as i32
    }
    /// Read an eight-byte signed value in little-endian wire order.
    /// # Safety
    /// Eight readable bytes must remain.
    #[inline(always)]
    pub unsafe fn read_i64(&mut self) -> i64 {
        self.read_u64() as i64
    }
}

/// Unchecked hot-path writer used after static schema analysis and exact buffer
/// allocation. Values are trusted; no runtime rank preflight is performed.
/// No capacity checks or value checks occur during the write pass.
///
/// # Safety
/// The caller must provide a writable allocation large enough for the entire
/// encoded record, and all values must satisfy the declared schema constraints.
pub struct QslUncheckedWriter {
    ptr: *mut u8,
    reg: u128,
    used: u32,
}

impl QslUncheckedWriter {
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

    /// Current committed byte cursor, after flushing pending bits.
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr
    }
    /// Encode directly into the final raw-size capacity, with no scratch buffer.
    /// # Safety
    /// Output must hold value.wire_capacity() exclusive, nonoverlapping bytes.
    #[inline(always)]
    pub unsafe fn write_inline_string<const MAX: usize>(
        &mut self,
        value: &crate::InlineString<MAX>,
    ) {
        self.flush_unchecked();
        self.ptr = self.ptr.add(value.encode_to(self.ptr));
    }

    /// # Safety
    /// `ptr` must point to a uniquely writable, pre-sized output allocation.
    #[inline(always)]
    pub unsafe fn from_raw(ptr: *mut u8) -> Self {
        Self {
            ptr,
            reg: 0,
            used: 0,
        }
    }

    /// Start at the beginning of the supplied buffer, without storing its
    /// capacity. Write methods remain unsafe even with this constructor.
    #[inline(always)]
    pub fn from(out: &mut [u8]) -> Self {
        Self {
            ptr: out.as_mut_ptr(),
            reg: 0,
            used: 0,
        }
    }

    /// Align to the next byte and copy a byte slice into the output cursor.
    /// # Safety
    /// The output region must hold exactly `bytes.len()` additional bytes.
    #[inline(always)]
    pub unsafe fn write_bytes_unchecked(&mut self, bytes: &[u8]) {
        self.flush_unchecked();
        ptr::copy_nonoverlapping(bytes.as_ptr(), self.ptr, bytes.len());
        self.ptr = self.ptr.add(bytes.len());
    }

    /// Align then copy `len` bytes from a raw, nonoverlapping source pointer.
    /// # Safety
    /// Both regions must be valid for `len` bytes and must not overlap.
    #[inline(always)]
    pub unsafe fn write_ptr_unchecked(&mut self, source: *const u8, len: usize) {
        self.flush_unchecked();
        ptr::copy_nonoverlapping(source, self.ptr, len);
        self.ptr = self.ptr.add(len);
    }

    /// Align then initialize exactly `count` output bytes to zero.
    /// # Safety
    /// The cursor must have `count` exclusively writable bytes remaining.
    #[inline(always)]
    pub unsafe fn write_zeros_unchecked(&mut self, count: usize) {
        self.flush_unchecked();
        ptr::write_bytes(self.ptr, 0, count);
        self.ptr = self.ptr.add(count);
    }

    /// Append `width` low bits of the rank, without per-byte iteration.
    /// # Safety
    /// Width must be at most 64; buffer capacity and range are caller-guaranteed.
    #[inline(always)]
    pub unsafe fn write_bits_unchecked(&mut self, value: u64, width: u32) {
        // At entry used<8 and width<=64, so the combined register fits u128.
        // The copy count is 0..=8 bytes; no per-bit or per-byte loop exists.
        self.reg |= ((value as u128) & ((1u128 << width) - 1)) << self.used;
        self.used += width;
        let count = (self.used / 8) as usize;
        let buf = self.reg.to_le_bytes();
        ptr::copy_nonoverlapping(buf.as_ptr(), self.ptr, count);
        self.ptr = self.ptr.add(count);
        self.reg >>= count * 8;
        self.used &= 7;
    }

    /// Flush zero or one partially occupied bytes; never inspect capacity.
    /// # Safety
    /// The destination must contain space for any trailing partial byte.
    #[inline(always)]
    pub unsafe fn flush_unchecked(&mut self) {
        let count = (self.used != 0) as usize;
        let buf = self.reg as u8;
        ptr::copy_nonoverlapping(&buf, self.ptr, count);
        self.ptr = self.ptr.add(count);
        self.reg = 0;
        self.used = 0;
    }

    /// Reserve a byte-aligned region for a custom encoder without checking
    /// the size of the output allocation.
    /// # Safety
    /// `n` bytes of writable storage must remain, and the custom codec must
    /// initialize the returned region before the buffer is observed.
    #[inline(always)]
    pub unsafe fn claim_bytes(&mut self, n: usize) -> *mut u8 {
        self.flush_unchecked();
        let ptr = self.ptr;
        self.ptr = self.ptr.add(n);
        ptr
    }

    /// Write a byte slice after aligning the bit register. No buffer checks.
    /// # Safety
    /// At least `bytes.len()` writable bytes must remain in the destination.
    #[inline(always)]
    pub unsafe fn write_bytes(&mut self, bytes: &[u8]) {
        self.write_bytes_unchecked(bytes);
    }

    /// Write zero-filled bytes with no bounds checks.
    /// # Safety
    /// At least `n` writable bytes must remain.
    #[inline(always)]
    pub unsafe fn write_zeros(&mut self, n: usize) {
        self.write_zeros_unchecked(n);
    }

    /// Append exactly `width` least significant bits of `v` without a loop.
    /// # Safety
    /// The destination must have space for the encoded bits; width <= 64.
    #[inline(always)]
    pub unsafe fn write_bits(&mut self, v: u64, width: u32) {
        self.write_bits_unchecked(v, width);
    }

    /// Commit a trailing partial byte, if any, through a zero-or-one-byte
    /// pointer copy. No `if`, `Result`, or dynamic allocation.
    /// # Safety
    /// The output allocation must include the trailing byte if required.
    #[inline(always)]
    pub unsafe fn flush(&mut self) {
        self.flush_unchecked();
    }
    /// Write one byte by a direct pointer store, without an intermediate slice.
    /// # Safety
    /// The destination must have one available byte.
    #[inline(always)]
    pub unsafe fn write_u8(&mut self, value: u8) {
        self.flush_unchecked();
        ptr::write(self.ptr, value);
        self.ptr = self.ptr.add(1);
    }

    /// Write a little-endian 16-bit value using one unaligned pointer store.
    /// # Safety
    /// The destination must have two available bytes.
    #[inline(always)]
    pub unsafe fn write_u16(&mut self, value: u16) {
        self.flush_unchecked();
        ptr::write_unaligned(self.ptr.cast::<u16>(), value.to_le());
        self.ptr = self.ptr.add(2);
    }

    /// Write a little-endian 32-bit value using one unaligned pointer store.
    /// # Safety
    /// The destination must have four available bytes.
    #[inline(always)]
    pub unsafe fn write_u32(&mut self, value: u32) {
        self.flush_unchecked();
        ptr::write_unaligned(self.ptr.cast::<u32>(), value.to_le());
        self.ptr = self.ptr.add(4);
    }

    /// Write a little-endian 64-bit value using one unaligned pointer store.
    /// # Safety
    /// The destination must have eight available bytes.
    #[inline(always)]
    pub unsafe fn write_u64(&mut self, value: u64) {
        self.flush_unchecked();
        ptr::write_unaligned(self.ptr.cast::<u64>(), value.to_le());
        self.ptr = self.ptr.add(8);
    }
    /// Write one signed byte with its two's-complement bit pattern.
    /// # Safety
    /// One writable byte must remain in the buffer.
    #[inline(always)]
    pub unsafe fn write_i8(&mut self, v: i8) {
        self.write_u8(v as u8);
    }
    /// Write a little-endian signed 16-bit value.
    /// # Safety
    /// Two writable bytes must remain.
    #[inline(always)]
    pub unsafe fn write_i16(&mut self, v: i16) {
        self.write_u16(v as u16);
    }
    /// Write a little-endian signed 32-bit value.
    /// # Safety
    /// Four writable bytes must remain.
    #[inline(always)]
    pub unsafe fn write_i32(&mut self, v: i32) {
        self.write_u32(v as u32);
    }
    /// Write a little-endian signed 64-bit value.
    /// # Safety
    /// Eight writable bytes must remain.
    #[inline(always)]
    pub unsafe fn write_i64(&mut self, v: i64) {
        self.write_u64(v as u64);
    }
}

/// Primary QSL I/O names intentionally resolve to the minimal unchecked
/// cursors. The `QslChecked*` variants are for diagnostics and input validation.
pub type QslWriter = QslRawWriter;
pub type QslReader = QslRawReader;
pub type QslBitsWriter = QslUncheckedWriter;
pub type QslBitsReader = QslFastBitsReader;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(feature = "diagnostics")]
    fn bit_roundtrip_crosses_bytes_and_zero_width() {
        let mut bytes = [0u8; 17];
        let mut w = QslCheckedBitsWriter::from(&mut bytes);
        w.write_bits(5, 3).unwrap();
        w.write_bits(0, 0).unwrap();
        w.write_bits(0x1234_5678_9abc_def0, 64).unwrap();
        w.write_bits(3, 2).unwrap();
        w.flush().unwrap();
        let len = w.position();
        let mut r = QslCheckedBitsReader::new(&bytes[..len]);
        assert_eq!(r.read_bits(3).unwrap(), 5);
        assert_eq!(r.read_bits(0).unwrap(), 0);
        assert_eq!(r.read_bits(64).unwrap(), 0x1234_5678_9abc_def0);
        assert_eq!(r.read_bits(2).unwrap(), 3);
        r.align().unwrap();
        assert_eq!(r.position(), len);
    }
    #[test]
    fn bitmap_msb_in_u32_le() {
        let mask = (1u32 << 31) | (1u32 << 30);
        assert_eq!(mask.to_le_bytes(), [0, 0, 0, 0xc0]);
    }

    #[test]
    fn primary_io_is_pointer_only_and_branchless_at_source_level() {
        assert_eq!(
            std::mem::size_of::<QslWriter>(),
            std::mem::size_of::<*mut u8>()
        );
        assert_eq!(
            std::mem::size_of::<QslReader>(),
            std::mem::size_of::<*const u8>()
        );
        let mut bytes = [0u8; 4];
        unsafe {
            let mut writer = QslBitsWriter::from(&mut bytes);
            writer.write_bits(5, 3);
            writer.write_bits(31, 5);
            writer.write_u16(0xABCD);
            writer.flush();
            let mut reader = QslBitsReader::from(&bytes);
            assert_eq!(reader.read_bits(3), 5);
            assert_eq!(reader.read_bits(5), 31);
            assert_eq!(reader.read_u16(), 0xABCD);
        }
        assert_eq!(&bytes[..3], &[0xFD, 0xCD, 0xAB]);
    }

    #[test]
    #[cfg(feature = "diagnostics")]
    fn unchecked_u128_register_matches_checked_codec_across_boundaries() {
        let values = [
            (0u64, 0u32),
            (5, 3),
            (0x1234_5678_9abc_def0, 64),
            (0, 0),
            (1, 1),
            (127, 7),
            (u64::MAX, 64),
            (0, 0),
        ];
        let mut checked = [0u8; 32];
        let mut unchecked = [0u8; 32];
        let mut reference = QslCheckedBitsWriter::from(&mut checked);
        for &(value, width) in &values {
            reference.write_bits(value, width).unwrap();
        }
        reference.flush().unwrap();
        unsafe {
            let mut writer = QslUncheckedWriter::from_raw(unchecked.as_mut_ptr());
            for &(value, width) in &values {
                writer.write_bits_unchecked(value, width);
            }
            writer.flush_unchecked();
        }
        assert_eq!(unchecked, checked);
        unsafe {
            let mut reader = QslFastBitsReader::new(unchecked.as_ptr());
            for &(value, width) in &values {
                assert_eq!(reader.read_bits(width), value);
            }
            reader.align();
        }
    }

    #[test]
    fn raw_pointer_primitives_work_from_unaligned_addresses() {
        let mut bytes = [0u8; 16];
        unsafe {
            let mut writer = QslRawWriter::new(bytes.as_mut_ptr().add(1));
            writer.write_u8(0xFA);
            writer.write_u16(0x1234);
            writer.write_u32(0xDEAD_BEEF);
            writer.write_u64(0x0123_4567_89AB_CDEF);
            let mut reader = QslRawReader::new(bytes.as_ptr().add(1));
            assert_eq!(reader.read_u8(), 0xFA);
            assert_eq!(reader.read_u16(), 0x1234);
            assert_eq!(reader.read_u32(), 0xDEAD_BEEF);
            assert_eq!(reader.read_u64(), 0x0123_4567_89AB_CDEF);
        }
    }
}
