//! Experimental fixed-region kernels. These never choose a wire format.
//! Inputs are canonical nonnegative ranks in logical order; the lowest rank is
//! stored first in the LSB-first compact stream. The caller checks ranges once.
//! ISA entry points require an outer feature gate; there is no per-field dispatch.
//! No kernel loads padding, an uninitialized suffix, or bytes beyond its array.
use crate::{QslError, QslResult};
#[inline]
pub const fn mask(bits: usize) -> u64 {
    assert!(bits <= 64);
    if bits == 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}
#[inline]
pub fn pack4<const A: usize, const B: usize, const C: usize, const D: usize>(v: [u32; 4]) -> u128 {
    const {
        assert!(A <= 32 && B <= 32 && C <= 32 && D <= 32);
    }
    (v[0] as u128)
        | ((v[1] as u128) << A)
        | ((v[2] as u128) << (A + B))
        | ((v[3] as u128) << (A + B + C))
}
#[inline]
pub fn unpack4<const A: usize, const B: usize, const C: usize, const D: usize>(
    v: u128,
) -> [u32; 4] {
    const {
        assert!(A <= 32 && B <= 32 && C <= 32 && D <= 32);
    }
    [
        (v as u64 & mask(A)) as u32,
        ((v >> A) as u64 & mask(B)) as u32,
        ((v >> (A + B)) as u64 & mask(C)) as u32,
        ((v >> (A + B + C)) as u64 & mask(D)) as u32,
    ]
}
/// Checks the rank domain once before entering a trusted packer.
pub fn validate4(v: [u32; 4], widths: [usize; 4]) -> QslResult<()> {
    for (x, w) in v.into_iter().zip(widths) {
        if w > 32 || u64::from(x) > mask(w) {
            return Err(QslError::InvalidValue);
        }
    }
    Ok(())
}
/// Two independently loaded GPR words, followed by canonical shift/OR packing.
#[inline]
pub fn pack4_gpr<const A: usize, const B: usize, const C: usize, const D: usize>(
    v: &[u32; 4],
) -> u128 {
    const {
        assert!(A <= 32 && B <= 32 && C <= 32 && D <= 32);
    }
    // All 16 bytes are initialized u32 elements. The result is endian-independent.
    let lo = u64::from(v[0]) | (u64::from(v[1]) << 32);
    let hi = u64::from(v[2]) | (u64::from(v[3]) << 32);
    pack4::<A, B, C, D>([lo as u32, (lo >> 32) as u32, hi as u32, (hi >> 32) as u32])
}
#[cfg(target_arch = "x86_64")]
pub mod x86 {
    use super::*;
    use core::arch::x86_64::*;
    /// # Safety
    /// CPU supports BMI2; each lane fits its compile-time width (0..=32).
    #[target_feature(enable = "bmi2")]
    pub unsafe fn pack4_bmi2<const A: usize, const B: usize, const C: usize, const D: usize>(
        v: &[u32; 4],
    ) -> u128 {
        const {
            assert!(A <= 32 && B <= 32 && C <= 32 && D <= 32);
        }
        let lo = core::ptr::read_unaligned(v.as_ptr().cast::<u64>());
        let hi = core::ptr::read_unaligned(v.as_ptr().add(2).cast::<u64>());
        u128::from(_pext_u64(lo, mask(A) | (mask(B) << 32)))
            | (u128::from(_pext_u64(hi, mask(C) | (mask(D) << 32))) << (A + B))
    }
    /// # Safety
    /// CPU supports BMI2; bits above the combined canonical width are zero.
    #[target_feature(enable = "bmi2")]
    pub unsafe fn unpack4_bmi2<const A: usize, const B: usize, const C: usize, const D: usize>(
        v: u128,
    ) -> [u32; 4] {
        const {
            assert!(A <= 32 && B <= 32 && C <= 32 && D <= 32);
        }
        let lo = _pdep_u64(v as u64, mask(A) | (mask(B) << 32));
        let hi = _pdep_u64((v >> (A + B)) as u64, mask(C) | (mask(D) << 32));
        [lo as u32, (lo >> 32) as u32, hi as u32, (hi >> 32) as u32]
    }
    /// # Safety
    /// CPU supports SSSE3; each lane is at most 65535.
    #[target_feature(enable = "ssse3")]
    pub unsafe fn pack4_shuffle16(v: &[u32; 4]) -> u64 {
        let x = _mm_loadu_si128(v.as_ptr().cast());
        _mm_cvtsi128_si64(_mm_shuffle_epi8(
            x,
            _mm_setr_epi8(0, 1, 4, 5, 8, 9, 12, 13, -1, -1, -1, -1, -1, -1, -1, -1),
        )) as u64
    }
    /// # Safety
    /// CPU supports SSSE3. All 64 input bits are valid canonical payload.
    #[target_feature(enable = "ssse3")]
    pub unsafe fn unpack4_shuffle16(v: u64) -> [u32; 4] {
        // MOVQ loads exactly 8 input bytes and zeroes the high half of the XMM value.
        let x = _mm_shuffle_epi8(
            _mm_cvtsi64_si128(v as i64),
            _mm_setr_epi8(0, 1, -1, -1, 2, 3, -1, -1, 4, 5, -1, -1, 6, 7, -1, -1),
        );
        let mut out = [0u32; 4];
        _mm_storeu_si128(out.as_mut_ptr().cast(), x);
        out
    }
    /// # Safety
    /// CPU supports SSE4.1; each lane is in 0..=65535, so saturation is impossible.
    #[target_feature(enable = "sse4.1")]
    pub unsafe fn pack4_packus16(v: &[u32; 4]) -> u64 {
        _mm_cvtsi128_si64(_mm_packus_epi32(
            _mm_loadu_si128(v.as_ptr().cast()),
            _mm_setzero_si128(),
        )) as u64
    }
    /// # Safety
    /// CPU supports SSE4.1. All 64 input bits are valid canonical payload.
    #[target_feature(enable = "sse4.1")]
    pub unsafe fn unpack4_packus16(v: u64) -> [u32; 4] {
        let x = _mm_cvtepu16_epi32(_mm_cvtsi64_si128(v as i64));
        let mut out = [0u32; 4];
        _mm_storeu_si128(out.as_mut_ptr().cast(), x);
        out
    }
    /// # Safety
    /// CPU supports SSE4.1; all lanes are in 0..=4095. Signed PMADDWD
    /// products and sums fit i32, and PACKUSDW cannot saturate.
    #[target_feature(enable = "sse4.1")]
    pub unsafe fn pack4_madd12(v: &[u32; 4]) -> u64 {
        let narrow = _mm_packus_epi32(_mm_loadu_si128(v.as_ptr().cast()), _mm_setzero_si128());
        let pairs = _mm_madd_epi16(narrow, _mm_setr_epi16(1, 4096, 1, 4096, 0, 0, 0, 0));
        let pairs = _mm_cvtsi128_si64(pairs) as u64;
        (pairs & 0xffffff) | ((pairs >> 32) << 24)
    }
    /// # Safety
    /// CPU supports AVX2; every lane is in 0..=65535.
    #[target_feature(enable = "avx2")]
    pub unsafe fn pack8_avx2(v: &[u32; 8]) -> [u8; 16] {
        let x = _mm256_loadu_si256(v.as_ptr().cast());
        let packed = _mm256_packus_epi32(x, _mm256_setzero_si256());
        // PACKUSDW is lane-local; lanes 0 and 2 contain the eight wanted u16s.
        let ordered = _mm256_permute4x64_epi64::<0xd8>(packed);
        let mut out = [0u8; 16];
        _mm_storeu_si128(out.as_mut_ptr().cast(), _mm256_castsi256_si128(ordered));
        out
    }
    /// # Safety
    /// CPU supports AVX2. The complete 16-byte array is initialized.
    #[target_feature(enable = "avx2")]
    pub unsafe fn unpack8_avx2(v: &[u8; 16]) -> [u32; 8] {
        let x = _mm256_cvtepu16_epi32(_mm_loadu_si128(v.as_ptr().cast()));
        let mut out = [0u32; 8];
        _mm256_storeu_si256(out.as_mut_ptr().cast(), x);
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use std::{prelude::v1::*, vec};
    #[test]
    fn feature_gated_kernels_match_scalar_extremes_and_random_ranks() {
        let mut seed = 0x12345678abcdefu64;
        for i in 0..10000 {
            let mut v = [0u32; 8];
            for n in &mut v {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                *n = if i == 0 {
                    0
                } else if i == 1 {
                    65535
                } else {
                    seed as u32 & 65535
                };
            }
            let a = [v[0], v[1], v[2], v[3]];
            let bits = pack4::<16, 16, 16, 16>(a) as u64;
            assert_eq!(unpack4::<16, 16, 16, 16>(bits as u128), a);
            #[cfg(target_arch = "x86_64")]
            unsafe {
                if std::is_x86_feature_detected!("bmi2") {
                    let v = [a[0] & 31, a[1] & 2047, a[2] & 8191, a[3] & 127];
                    let p = pack4_bmi2_test(v);
                    assert_eq!(p, pack4::<5, 11, 13, 7>(v));
                    assert_eq!(x86::unpack4_bmi2::<5, 11, 13, 7>(p), v);
                    assert_eq!(
                        x86::pack4_bmi2::<32, 32, 32, 32>(&a),
                        pack4::<32, 32, 32, 32>(a)
                    );
                    assert_eq!(
                        x86::unpack4_bmi2::<32, 32, 32, 32>(pack4::<32, 32, 32, 32>(a)),
                        a
                    );
                }
                if std::is_x86_feature_detected!("ssse3") {
                    assert_eq!(x86::pack4_shuffle16(&a), bits);
                    assert_eq!(x86::unpack4_shuffle16(bits), a);
                }
                if std::is_x86_feature_detected!("sse4.1") {
                    assert_eq!(x86::pack4_packus16(&a), bits);
                    assert_eq!(x86::unpack4_packus16(bits), a);
                    let v = a.map(|x| x & 4095);
                    assert_eq!(x86::pack4_madd12(&v) as u128, pack4::<12, 12, 12, 12>(v));
                }
                if std::is_x86_feature_detected!("avx2") {
                    let wire = x86::pack8_avx2(&v);
                    let expected: Vec<_> =
                        v.iter().flat_map(|x| (*x as u16).to_le_bytes()).collect();
                    assert_eq!(wire.as_slice(), expected);
                    assert_eq!(x86::unpack8_avx2(&wire), v);
                }
            }
        }
        assert!(validate4([65536, 0, 0, 0], [16; 4]).is_err());
        assert!(validate4([1, 0, 0, 0], [0; 4]).is_err());
        assert!(validate4([0; 4], [33; 4]).is_err());
        assert!(validate4([0; 4], [0; 4]).is_ok());
    }
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "bmi2")]
    unsafe fn pack4_bmi2_test(v: [u32; 4]) -> u128 {
        x86::pack4_bmi2::<5, 11, 13, 7>(&v)
    }
}
