//! General lossless QPS byte groups. This codec does not predict field presence.
//! Q selects 2/4/8/16 byte lanes. Each block is one radix-2^B operand;
//! the shared minimum and all exact ranks are retained. Length is never quantized.
use crate::{BorrowSerializable, QslError, QslResult, Serializable};
use std::mem::MaybeUninit;

pub const BYTE_FRAME_HEADER: usize = 16;
pub const BYTE_QPS_CODEC_REVISION: u32 = 3;
const MAGIC: [u8; 4] = *b"QPB3";
// Full groups are loaded as one integer. The shared minimum guarantees that
// subtracting its repeated-byte operand cannot borrow between byte lanes.
type Pack = unsafe fn(u128, *mut u8);
type Unpack = unsafe fn(*const u8, u8, *mut u8) -> u8;
const fn lanes<const Q: usize>() -> u128 {
    match Q {
        2 => 0x0101,
        4 => 0x01010101,
        8 => 0x0101010101010101,
        16 => u128::MAX / 255,
        _ => 0,
    }
}
#[inline(always)]
unsafe fn read_lanes<const Q: usize>(p: *const u8) -> u128 {
    match Q {
        2 => u16::from_le(unsafe { p.cast::<u16>().read_unaligned() }) as u128,
        4 => u32::from_le(unsafe { p.cast::<u32>().read_unaligned() }) as u128,
        8 => u64::from_le(unsafe { p.cast::<u64>().read_unaligned() }) as u128,
        16 => u128::from_le(unsafe { p.cast::<u128>().read_unaligned() }),
        _ => unreachable!(),
    }
}
#[inline(always)]
fn word_width<const Q: usize>(mut w: u128) -> usize {
    if Q > 8 {
        w |= w >> 64
    }
    if Q > 4 {
        w |= w >> 32
    }
    if Q > 2 {
        w |= w >> 16
    }
    w |= w >> 8;
    (u8::BITS - (w as u8).leading_zeros()) as usize
}
#[inline(always)]
unsafe fn read_packed<const Q: usize, const B: usize>(p: *const u8) -> u128 {
    let n = (Q * B + 7) >> 3;
    let mut at = 0;
    let mut w = 0u128;
    if n >= 8 {
        w = u64::from_le(unsafe { p.cast::<u64>().read_unaligned() }) as u128;
        at = 8;
    }
    if n == 16 {
        w |= (u64::from_le(unsafe { p.add(8).cast::<u64>().read_unaligned() }) as u128) << 64;
        at = 16;
    }
    if n - at >= 4 {
        w |=
            (u32::from_le(unsafe { p.add(at).cast::<u32>().read_unaligned() }) as u128) << (at * 8);
        at += 4;
    }
    if n - at >= 2 {
        w |=
            (u16::from_le(unsafe { p.add(at).cast::<u16>().read_unaligned() }) as u128) << (at * 8);
        at += 2;
    }
    if n > at {
        w |= (unsafe { *p.add(at) } as u128) << (at * 8);
    }
    w
}
#[inline(always)]
unsafe fn write_packed<const Q: usize, const B: usize>(w: u128, p: *mut u8) {
    let n = (Q * B + 7) >> 3;
    let mut at = 0;
    if n >= 8 {
        unsafe { p.cast::<u64>().write_unaligned((w as u64).to_le()) };
        at = 8;
    }
    if n == 16 {
        unsafe {
            p.add(8)
                .cast::<u64>()
                .write_unaligned(((w >> 64) as u64).to_le())
        };
        at = 16;
    }
    if n - at >= 4 {
        unsafe {
            p.add(at)
                .cast::<u32>()
                .write_unaligned(((w >> (at * 8)) as u32).to_le())
        };
        at += 4;
    }
    if n - at >= 2 {
        unsafe {
            p.add(at)
                .cast::<u16>()
                .write_unaligned(((w >> (at * 8)) as u16).to_le())
        };
        at += 2;
    }
    if n > at {
        unsafe { p.add(at).write((w >> (at * 8)) as u8) }
    }
}
#[inline(always)]
unsafe fn restore_lanes<const Q: usize, const B: usize>(
    r: u128,
    packed: u128,
    base: u8,
    out: *mut u8,
) -> u8 {
    let a = lanes::<Q>() * base as u128;
    let sum = r.wrapping_add(a);
    // Carry-out at each byte's high bit detects every overflowing lane, including
    // the last one. This also works when a lower lane polluted a higher lane.
    let carry = ((r & a) | ((r | a) & !sum)) & (lanes::<Q>() << 7);
    let used = Q * B;
    let padding = u8::from((packed >> (used & 127)) != 0) * u8::from(used < 128) * 2;
    match Q {
        2 => unsafe { out.cast::<u16>().write_unaligned((sum as u16).to_le()) },
        4 => unsafe { out.cast::<u32>().write_unaligned((sum as u32).to_le()) },
        8 => unsafe { out.cast::<u64>().write_unaligned((sum as u64).to_le()) },
        16 => unsafe { out.cast::<u128>().write_unaligned(sum.to_le()) },
        _ => unreachable!(),
    };
    u8::from(carry != 0) | padding
}
#[inline]
unsafe fn pack_portable<const Q: usize, const B: usize>(r: u128, out: *mut u8) {
    let mut w = 0u128;
    for i in 0..Q {
        w |= ((r >> (i * 8)) & 255) << (i * B)
    }
    unsafe { write_packed::<Q, B>(w, out) }
}
#[inline]
unsafe fn unpack_portable<const Q: usize, const B: usize>(
    input: *const u8,
    base: u8,
    out: *mut u8,
) -> u8 {
    let w = unsafe { read_packed::<Q, B>(input) };
    let mask = (1u128 << B) - 1;
    let mut r = 0u128;
    for i in 0..Q {
        r |= ((w >> (i * B)) & mask) << (i * 8)
    }
    unsafe { restore_lanes::<Q, B>(r, w, base, out) }
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
unsafe fn pack_bmi2<const Q: usize, const B: usize>(r: u128, out: *mut u8) {
    let mask = lanes::<Q>() * ((1u128 << B) - 1);
    let mut w = std::arch::x86_64::_pext_u64(r as u64, mask as u64) as u128;
    if Q == 16 {
        w |=
            (std::arch::x86_64::_pext_u64((r >> 64) as u64, (mask >> 64) as u64) as u128) << (8 * B)
    }
    unsafe { write_packed::<Q, B>(w, out) }
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
unsafe fn unpack_bmi2<const Q: usize, const B: usize>(
    input: *const u8,
    base: u8,
    out: *mut u8,
) -> u8 {
    let w = unsafe { read_packed::<Q, B>(input) };
    let mask = lanes::<Q>() * ((1u128 << B) - 1);
    let mut r = std::arch::x86_64::_pdep_u64(w as u64, mask as u64) as u128;
    if Q == 16 {
        r |= (std::arch::x86_64::_pdep_u64((w >> (8 * B)) as u64, (mask >> 64) as u64) as u128)
            << 64;
    }
    unsafe { restore_lanes::<Q, B>(r, w, base, out) }
}
fn packers<const Q: usize>() -> [Pack; 9] {
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("bmi2") {
        return [
            pack_bmi2::<Q, 0>,
            pack_bmi2::<Q, 1>,
            pack_bmi2::<Q, 2>,
            pack_bmi2::<Q, 3>,
            pack_bmi2::<Q, 4>,
            pack_bmi2::<Q, 5>,
            pack_bmi2::<Q, 6>,
            pack_bmi2::<Q, 7>,
            pack_bmi2::<Q, 8>,
        ];
    }
    [
        pack_portable::<Q, 0>,
        pack_portable::<Q, 1>,
        pack_portable::<Q, 2>,
        pack_portable::<Q, 3>,
        pack_portable::<Q, 4>,
        pack_portable::<Q, 5>,
        pack_portable::<Q, 6>,
        pack_portable::<Q, 7>,
        pack_portable::<Q, 8>,
    ]
}
fn unpackers<const Q: usize>() -> [Unpack; 9] {
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("bmi2") {
        return [
            unpack_bmi2::<Q, 0>,
            unpack_bmi2::<Q, 1>,
            unpack_bmi2::<Q, 2>,
            unpack_bmi2::<Q, 3>,
            unpack_bmi2::<Q, 4>,
            unpack_bmi2::<Q, 5>,
            unpack_bmi2::<Q, 6>,
            unpack_bmi2::<Q, 7>,
            unpack_bmi2::<Q, 8>,
        ];
    }
    [
        unpack_portable::<Q, 0>,
        unpack_portable::<Q, 1>,
        unpack_portable::<Q, 2>,
        unpack_portable::<Q, 3>,
        unpack_portable::<Q, 4>,
        unpack_portable::<Q, 5>,
        unpack_portable::<Q, 6>,
        unpack_portable::<Q, 7>,
        unpack_portable::<Q, 8>,
    ]
}
fn quantum(q: usize) -> QslResult<()> {
    if matches!(q, 2 | 4 | 8 | 16) {
        Ok(())
    } else {
        Err(QslError::InvalidWidth)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteMode {
    Raw,
    Packed,
}
/// General framed codec. Inner QSL/string/array bytes restore exactly;
/// this is separate from the statically specialized scalar QPS representation.
pub struct ByteQps<const Q: usize>;
pub struct ByteFrame<'a, const Q: usize> {
    bytes: &'a [u8],
    original: usize,
    mode: ByteMode,
}
impl<const Q: usize> ByteFrame<'_, Q> {
    pub fn as_slice(&self) -> &[u8] {
        self.bytes
    }
    pub fn original_len(&self) -> usize {
        self.original
    }
    pub fn mode(&self) -> ByteMode {
        self.mode
    }
    pub fn decode_to<'out>(&self, out: &'out mut [u8]) -> QslResult<&'out mut [u8]> {
        ByteQps::<Q>::decode_to(self.bytes, out)
    }
    pub fn decode_record_into<'out, T: CheckedRecord>(
        &self,
        restoration: &mut [u8],
        out: &'out mut MaybeUninit<T>,
    ) -> QslResult<&'out mut T> {
        ByteQps::<Q>::decode_record_into(self.bytes, restoration, out)
    }
}
/// # Safety
/// Implementations must validate exactly the representation consumed by
/// Serializable, reject invalid values before encoding, and obey that trait's
/// complete size, initialization and non-overlap contracts. Generated safety
/// schemas implement this trait; arbitrary validators cannot authorize unsafe reads.
pub unsafe trait CheckedRecord: Serializable {
    const MAX_DATA_SIZE: usize = crate::validation::DEFAULT_MAX_DATA_SIZE;
    fn qps_validate_values(&self) -> QslResult<()>;
    fn qps_validate_wire(input: &[u8]) -> QslResult<()>;
}
/// # Safety
/// Validation must prove the exact borrowed representation and source lifetime
/// required by BorrowSerializable. The restored region must outlive the value.
pub unsafe trait CheckedBorrowedRecord<'a>: BorrowSerializable<'a> {
    const MAX_DATA_SIZE: usize = crate::validation::DEFAULT_MAX_DATA_SIZE;
    fn qps_validate_values(&self) -> QslResult<()>;
    fn qps_validate_wire(input: &[u8]) -> QslResult<()>;
}
impl<const Q: usize> ByteQps<Q> {
    pub fn max_encoded_len(len: usize) -> QslResult<usize> {
        quantum(Q)?;
        len.checked_add(BYTE_FRAME_HEADER)
            .ok_or(QslError::LengthOverflow)
    }
    pub fn inspect(input: &[u8]) -> QslResult<(ByteMode, usize)> {
        quantum(Q)?;
        let h = input
            .get(..BYTE_FRAME_HEADER)
            .ok_or(QslError::UnexpectedEof)?;
        if h[..4] != MAGIC || h[4] as usize != Q || h[7] != 0 {
            return Err(QslError::InvalidSchema);
        }
        let mode = match h[5] {
            0 => ByteMode::Raw,
            1 => ByteMode::Packed,
            _ => return Err(QslError::InvalidValue),
        };
        let original = usize::try_from(u64::from_le_bytes(h[8..16].try_into().unwrap()))
            .map_err(|_| QslError::LengthOverflow)?;
        if mode == ByteMode::Raw && (h[6] != 0 || input.len() - BYTE_FRAME_HEADER != original) {
            return Err(QslError::InvalidValue);
        }
        if mode == ByteMode::Packed && (original == 0 || input.len() >= original) {
            return Err(QslError::InvalidValue);
        }
        Ok((mode, original))
    }
    /// Encode any bytes without allocating. Output needs header+input.len().
    /// Packed is selected only if its COMPLETE frame is smaller than input;
    /// raw carries a visible 16-byte envelope rather than hiding framing cost.
    // Keep invalid Q=0 a recoverable InvalidWidth, rather than instantiating
    // as_chunks::<0>'s compile-time assertion. Valid Q is checked once above.
    #[allow(clippy::chunks_exact_to_as_chunks)]
    pub fn encode_to<'out>(input: &[u8], out: &'out mut [u8]) -> QslResult<ByteFrame<'out, Q>> {
        let capacity = Self::max_encoded_len(input.len())?;
        if out.len() < capacity {
            return Err(QslError::BufferTooSmall);
        }
        let base = input.iter().copied().min().unwrap_or(0);
        let a = lanes::<Q>() * base as u128;
        let mut candidate = BYTE_FRAME_HEADER;
        let full = input.len() / Q;
        let tail = &input[full * Q..];
        let mut staged = [base; 16];
        staged[..tail.len()].copy_from_slice(tail);
        for block in input.chunks_exact(Q) {
            let ranks = unsafe { read_lanes::<Q>(block.as_ptr()) } - a;
            let b = word_width::<Q>(ranks);
            candidate = candidate
                .checked_add(1 + ((Q * b + 7) >> 3))
                .ok_or(QslError::LengthOverflow)?;
        }
        let tail_ranks = unsafe { read_lanes::<Q>(staged.as_ptr()) } - a;
        let tail_bits = word_width::<Q>(tail_ranks);
        if !tail.is_empty() {
            candidate = candidate
                .checked_add(1 + ((tail.len() * tail_bits + 7) >> 3))
                .ok_or(QslError::LengthOverflow)?;
        }
        let packed = candidate < input.len();
        out[..4].copy_from_slice(&MAGIC);
        out[4] = Q as u8;
        out[5] = u8::from(packed);
        out[6] = if packed { base } else { 0 };
        out[7] = 0;
        out[8..16].copy_from_slice(&(input.len() as u64).to_le_bytes());
        let len = if packed {
            let table = packers::<Q>();
            let mut cursor = BYTE_FRAME_HEADER;
            for block in input.chunks_exact(Q) {
                let ranks = unsafe { read_lanes::<Q>(block.as_ptr()) } - a;
                let b = word_width::<Q>(ranks);
                out[cursor] = b as u8;
                cursor += 1;
                unsafe { table[b](ranks, out.as_mut_ptr().add(cursor)) };
                cursor += (Q * b + 7) >> 3;
            }
            if !tail.is_empty() {
                out[cursor] = tail_bits as u8;
                cursor += 1;
                // capacity=input.len()+16 and candidate<input.len() guarantee
                // room for this final full-word store. Only actual tail bytes transmit.
                unsafe { table[tail_bits](tail_ranks, out.as_mut_ptr().add(cursor)) };
                cursor += (tail.len() * tail_bits + 7) >> 3;
            }
            debug_assert_eq!(cursor, candidate);
            cursor
        } else {
            out[16..capacity].copy_from_slice(input);
            capacity
        };
        Ok(ByteFrame {
            bytes: &out[..len],
            original: input.len(),
            mode: if packed {
                ByteMode::Packed
            } else {
                ByteMode::Raw
            },
        })
    }
    /// Safe exact-slice restoration: no input padding or allocation is needed.
    /// Output may change before a malformed-input error is returned.
    #[allow(clippy::chunks_exact_to_as_chunks)]
    pub fn decode_to<'out>(input: &[u8], out: &'out mut [u8]) -> QslResult<&'out mut [u8]> {
        Self::decode_to_with_limit(input, out, usize::MAX)
    }
    /// Bound both the envelope and restored frame before touching output.
    #[allow(clippy::chunks_exact_to_as_chunks)]
    pub fn decode_to_with_limit<'out>(
        input: &[u8],
        out: &'out mut [u8],
        maximum: usize,
    ) -> QslResult<&'out mut [u8]> {
        crate::validation::check_size(input.len(), maximum)?;
        let (mode, original) = Self::inspect(input)?;
        crate::validation::check_size(original, maximum)?;
        let out = out.get_mut(..original).ok_or(QslError::BufferTooSmall)?;
        if mode == ByteMode::Raw {
            out.copy_from_slice(&input[16..]);
            return Ok(out);
        }
        let base = input[6];
        let table = unpackers::<Q>();
        let mut cursor = 16;
        let full = original / Q;
        for block in out[..full * Q].chunks_exact_mut(Q) {
            let b = *input.get(cursor).ok_or(QslError::UnexpectedEof)? as usize;
            cursor += 1;
            if b > 8 {
                return Err(QslError::InvalidWidth);
            }
            let n = (Q * b + 7) >> 3;
            let end = cursor.checked_add(n).ok_or(QslError::LengthOverflow)?;
            let encoded = input.get(cursor..end).ok_or(QslError::UnexpectedEof)?;
            let flags = unsafe { table[b](encoded.as_ptr(), base, block.as_mut_ptr()) };
            if flags & 2 != 0 {
                return Err(QslError::InvalidPadding);
            }
            if flags & 1 != 0 {
                return Err(QslError::InvalidValue);
            }
            cursor = end;
        }
        let tail = &mut out[full * Q..];
        if !tail.is_empty() {
            let b = *input.get(cursor).ok_or(QslError::UnexpectedEof)? as usize;
            cursor += 1;
            if b > 8 {
                return Err(QslError::InvalidWidth);
            }
            let n = (tail.len() * b + 7) >> 3;
            let end = cursor.checked_add(n).ok_or(QslError::LengthOverflow)?;
            let encoded = input.get(cursor..end).ok_or(QslError::UnexpectedEof)?;
            let mut staging = [0u8; 16];
            staging[..n].copy_from_slice(encoded);
            let word = u128::from_le_bytes(staging);
            if word >> (tail.len() * b) != 0 {
                return Err(QslError::InvalidPadding);
            }
            let mut decoded = [0u8; 16];
            let flags = unsafe { table[b](staging.as_ptr(), base, decoded.as_mut_ptr()) };
            if flags & 1 != 0 {
                return Err(QslError::InvalidValue);
            }
            tail.copy_from_slice(&decoded[..tail.len()]);
            cursor = end;
        }
        if cursor != input.len() {
            return Err(QslError::InvalidValue);
        }
        Ok(out)
    }
    pub fn encode_borrowed_to<'out, 'a, T: CheckedBorrowedRecord<'a>>(
        value: &T,
        scratch: &mut [u8],
        out: &'out mut [u8],
    ) -> QslResult<ByteFrame<'out, Q>> {
        value.qps_validate_values()?;
        let cap = value.serialized_size();
        crate::validation::check_size(cap, T::MAX_DATA_SIZE)?;
        let scratch = scratch.get_mut(..cap).ok_or(QslError::BufferTooSmall)?;
        let n = unsafe { value.serialize_into_unsafe(scratch.as_mut_ptr()) };
        let frame = Self::encode_to(&scratch[..n], out)?;
        crate::validation::check_size(frame.as_slice().len(), T::MAX_DATA_SIZE)?;
        Ok(frame)
    }
    pub fn decode_borrowed<'a, T: CheckedBorrowedRecord<'a>>(
        input: &[u8],
        restoration: &'a mut [u8],
    ) -> QslResult<T> {
        let raw = Self::decode_to_with_limit(input, restoration, T::MAX_DATA_SIZE)?;
        T::qps_validate_wire(raw)?;
        Ok(unsafe { T::deserialize_borrowed_from_raw_parts(raw.as_ptr(), raw.len()) })
    }
    pub fn encode_record_to<'out, T: CheckedRecord>(
        value: &T,
        scratch: &mut [u8],
        out: &'out mut [u8],
    ) -> QslResult<ByteFrame<'out, Q>> {
        value.qps_validate_values()?;
        let cap = value.serialized_size();
        crate::validation::check_size(cap, T::MAX_DATA_SIZE)?;
        let scratch = scratch.get_mut(..cap).ok_or(QslError::BufferTooSmall)?;
        let n = unsafe { value.serialize_into_unsafe(scratch.as_mut_ptr()) };
        let frame = Self::encode_to(&scratch[..n], out)?;
        crate::validation::check_size(frame.as_slice().len(), T::MAX_DATA_SIZE)?;
        Ok(frame)
    }
    pub fn decode_record_into<'out, T: CheckedRecord>(
        input: &[u8],
        restoration: &mut [u8],
        out: &'out mut MaybeUninit<T>,
    ) -> QslResult<&'out mut T> {
        let raw = Self::decode_to_with_limit(input, restoration, T::MAX_DATA_SIZE)?;
        T::qps_validate_wire(raw)?;
        unsafe {
            T::deserialize_into(raw.as_ptr(), raw.len(), out.as_mut_ptr());
            Ok(out.assume_init_mut())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn roundtrip<const Q: usize>(value: &[u8]) {
        let mut encoded = vec![0; value.len() + BYTE_FRAME_HEADER];
        let frame = ByteQps::<Q>::encode_to(value, &mut encoded).unwrap();
        if frame.mode() == ByteMode::Packed {
            assert!(frame.as_slice().len() < value.len());
        }
        let mut output = vec![0; value.len()];
        assert_eq!(frame.decode_to(&mut output).unwrap(), value);
        let mut again = vec![0; value.len() + BYTE_FRAME_HEADER];
        let next = ByteQps::<Q>::encode_to(value, &mut again).unwrap();
        assert_eq!(frame.as_slice(), next.as_slice());
        for len in 0..frame.as_slice().len() {
            assert!(ByteQps::<Q>::decode_to(&frame.as_slice()[..len], &mut output).is_err());
        }
    }
    #[test]
    fn portable_and_cpu_kernels_are_identical() {
        fn check<const Q: usize, const B: usize>() {
            for base in [0u8, 1, 127, 200, 255] {
                let max = ((1u16 << B) - 1).min(255 - base as u16) as u8;
                let mut input = [base; 16];
                for (i, x) in input[..Q].iter_mut().enumerate() {
                    *x = base + (i as u8 & max);
                }
                let ranks =
                    unsafe { read_lanes::<Q>(input.as_ptr()) } - lanes::<Q>() * base as u128;
                let mut a = [0u8; 16];
                let mut b = [0u8; 16];
                unsafe { pack_portable::<Q, B>(ranks, a.as_mut_ptr()) };
                let mut out = [0u8; 16];
                assert_eq!(
                    unsafe { unpack_portable::<Q, B>(a.as_ptr(), base, out.as_mut_ptr()) },
                    0
                );
                assert_eq!(&out[..Q], &input[..Q]);
                #[cfg(target_arch = "x86_64")]
                if std::is_x86_feature_detected!("bmi2") {
                    unsafe { pack_bmi2::<Q, B>(ranks, b.as_mut_ptr()) };
                    assert_eq!(a, b);
                    out.fill(0);
                    assert_eq!(
                        unsafe { unpack_bmi2::<Q, B>(b.as_ptr(), base, out.as_mut_ptr()) },
                        0
                    );
                    assert_eq!(&out[..Q], &input[..Q]);
                }
            }
        }
        macro_rules! widths {
            ($q:literal) => {
                check::<$q, 0>();
                check::<$q, 1>();
                check::<$q, 2>();
                check::<$q, 3>();
                check::<$q, 4>();
                check::<$q, 5>();
                check::<$q, 6>();
                check::<$q, 7>();
                check::<$q, 8>();
            };
        }
        widths!(2);
        widths!(4);
        widths!(8);
        widths!(16);
    }
    #[test]
    fn all_quantums_restore_all_byte_values_and_tails() {
        for n in [0, 1, 2, 3, 4, 7, 8, 15, 16, 17, 63, 255, 256, 1024] {
            for kind in 0..4 {
                let value = (0..n)
                    .map(|i| match kind {
                        0 => 0,
                        1 => (i % 4) as u8,
                        2 => 128 + (i % 32) as u8,
                        _ => (i * 73 + 19) as u8,
                    })
                    .collect::<Vec<_>>();
                roundtrip::<2>(&value);
                roundtrip::<4>(&value);
                roundtrip::<8>(&value);
                roundtrip::<16>(&value);
            }
        }
    }
    #[test]
    fn strings_arrays_and_high_bases_are_exact() {
        let text = "Рад, так Рад. UTF-8 🦀\0".repeat(40);
        roundtrip::<16>(text.as_bytes());
        roundtrip::<4>(&[255; 512]);
        for byte in 0..=255 {
            roundtrip::<8>(&[byte; 64]);
        }
    }
    #[test]
    fn mutated_frames_never_panic_and_never_return_truncated_data() {
        let value = (0..1024).map(|i| (i % 16) as u8).collect::<Vec<_>>();
        let mut encoded = vec![0; 1040];
        let n = ByteQps::<16>::encode_to(&value, &mut encoded)
            .unwrap()
            .as_slice()
            .len();
        encoded.truncate(n);
        for index in 0..encoded.len() {
            for byte in [0, 1, 8, 9, 127, 255] {
                let mut bad = encoded.clone();
                bad[index] = byte;
                let mut restored = vec![0; 2048];
                if let Ok(bytes) = ByteQps::<16>::decode_to(&bad, &mut restored) {
                    assert_eq!(bytes.len(), value.len());
                }
            }
        }
        let mut corrupt = encoded;
        corrupt[6] = 255;
        let mut restored = [0; 1024];
        assert!(ByteQps::<16>::decode_to(&corrupt, &mut restored).is_err());
    }
    #[test]
    fn malformed_headers_widths_lengths_and_values_are_rejected() {
        let mut encoded = vec![0; 272];
        let n = ByteQps::<16>::encode_to(&[200; 256], &mut encoded)
            .unwrap()
            .as_slice()
            .len();
        encoded.truncate(n);
        let mut out = [0; 256];
        let mut bad = encoded.clone();
        bad[16] = 9;
        assert_eq!(
            ByteQps::<16>::decode_to(&bad, &mut out),
            Err(QslError::InvalidWidth)
        );
        let mut bad = encoded.clone();
        bad[5] = 2;
        assert_eq!(
            ByteQps::<16>::decode_to(&bad, &mut out),
            Err(QslError::InvalidValue)
        );
        let mut bad = encoded.clone();
        bad[7] = 1;
        assert_eq!(
            ByteQps::<16>::decode_to(&bad, &mut out),
            Err(QslError::InvalidSchema)
        );
        let mut bad = encoded.clone();
        bad.push(0);
        assert!(ByteQps::<16>::decode_to(&bad, &mut out).is_err());
        assert!(ByteQps::<3>::encode_to(&[0; 32], &mut [0; 48]).is_err());
        assert!(ByteQps::<16>::decode_to(&encoded, &mut [0; 255]).is_err());
    }
}
