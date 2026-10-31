//! The SSE2 instructions the rasteriser's four-at-a-time path uses (`row4`), on ARM's NEON, so
//! both processors run one path (the handheld, 2026-10-31). Each does what the x86 instruction
//! does, by its Intel name; `tiles_and_lanes_draw_the_same` checks the path against the plain
//! one. The one known difference: a float too big for an integer, or NaN, converts to the
//! nearest integer (0 for NaN) and not to `i32::MIN`, as Rust's own `as i32` does on every
//! processor.

use core::arch::aarch64::*;

#[allow(non_camel_case_types)]
pub type __m128 = float32x4_t;
#[allow(non_camel_case_types)]
pub type __m128i = int32x4_t;

macro_rules! neon {
    ($(fn $name:ident($($a:ident: $t:ty),*) -> $r:ty $body:block)*) => {
        $(
            #[inline(always)]
            pub fn $name($($a: $t),*) -> $r {
                // SAFETY: NEON is part of every AArch64 processor; these only compute.
                #[allow(unused_unsafe)]
                unsafe { $body }
            }
        )*
    };
}

neon! {
    fn _mm_castps_si128(a: __m128) -> __m128i { vreinterpretq_s32_f32(a) }
    fn _mm_castsi128_ps(a: __m128i) -> __m128 { vreinterpretq_f32_s32(a) }
    fn _mm_setzero_ps() -> __m128 { vdupq_n_f32(0.0) }
    fn _mm_set1_ps(a: f32) -> __m128 { vdupq_n_f32(a) }
    fn _mm_set_ps(d: f32, c: f32, b: f32, a: f32) -> __m128 {
        core::mem::transmute::<[f32; 4], __m128>([a, b, c, d])
    }
    fn _mm_set1_epi32(a: i32) -> __m128i { vdupq_n_s32(a) }
    fn _mm_set1_epi16(a: i16) -> __m128i { vreinterpretq_s32_s16(vdupq_n_s16(a)) }
    fn _mm_cvtsi32_si128(a: i32) -> __m128i {
        core::mem::transmute::<[i32; 4], __m128i>([a, 0, 0, 0])
    }

    fn _mm_add_ps(a: __m128, b: __m128) -> __m128 { vaddq_f32(a, b) }
    fn _mm_sub_ps(a: __m128, b: __m128) -> __m128 { vsubq_f32(a, b) }
    fn _mm_mul_ps(a: __m128, b: __m128) -> __m128 { vmulq_f32(a, b) }
    fn _mm_div_ps(a: __m128, b: __m128) -> __m128 { vdivq_f32(a, b) }
    // As x86's: the second when the comparison fails (NaN too).
    fn _mm_min_ps(a: __m128, b: __m128) -> __m128 { vbslq_f32(vcltq_f32(a, b), a, b) }
    fn _mm_max_ps(a: __m128, b: __m128) -> __m128 { vbslq_f32(vcgtq_f32(a, b), a, b) }

    fn _mm_and_ps(a: __m128, b: __m128) -> __m128 {
        _mm_castsi128_ps(vandq_s32(_mm_castps_si128(a), _mm_castps_si128(b)))
    }
    fn _mm_or_ps(a: __m128, b: __m128) -> __m128 {
        _mm_castsi128_ps(vorrq_s32(_mm_castps_si128(a), _mm_castps_si128(b)))
    }
    fn _mm_andnot_ps(a: __m128, b: __m128) -> __m128 {
        _mm_castsi128_ps(vbicq_s32(_mm_castps_si128(b), _mm_castps_si128(a)))
    }

    fn _mm_cmpeq_ps(a: __m128, b: __m128) -> __m128 { vreinterpretq_f32_u32(vceqq_f32(a, b)) }
    fn _mm_cmpneq_ps(a: __m128, b: __m128) -> __m128 {
        vreinterpretq_f32_u32(vmvnq_u32(vceqq_f32(a, b)))
    }
    fn _mm_cmplt_ps(a: __m128, b: __m128) -> __m128 { vreinterpretq_f32_u32(vcltq_f32(a, b)) }
    fn _mm_cmple_ps(a: __m128, b: __m128) -> __m128 { vreinterpretq_f32_u32(vcleq_f32(a, b)) }
    fn _mm_cmpgt_ps(a: __m128, b: __m128) -> __m128 { vreinterpretq_f32_u32(vcgtq_f32(a, b)) }
    fn _mm_cmpge_ps(a: __m128, b: __m128) -> __m128 { vreinterpretq_f32_u32(vcgeq_f32(a, b)) }
    fn _mm_cmpgt_epi32(a: __m128i, b: __m128i) -> __m128i {
        vreinterpretq_s32_u32(vcgtq_s32(a, b))
    }
    fn _mm_movemask_ps(a: __m128) -> i32 {
        let signs = vshrq_n_u32::<31>(vreinterpretq_u32_f32(a));
        let weights = core::mem::transmute::<[u32; 4], uint32x4_t>([1, 2, 4, 8]);
        vaddvq_u32(vmulq_u32(signs, weights)) as i32
    }

    fn _mm_cvttps_epi32(a: __m128) -> __m128i { vcvtq_s32_f32(a) }
    fn _mm_cvtepi32_ps(a: __m128i) -> __m128 { vcvtq_f32_s32(a) }

    fn _mm_add_epi32(a: __m128i, b: __m128i) -> __m128i { vaddq_s32(a, b) }
    fn _mm_and_si128(a: __m128i, b: __m128i) -> __m128i { vandq_s32(a, b) }
    fn _mm_or_si128(a: __m128i, b: __m128i) -> __m128i { vorrq_s32(a, b) }
    fn _mm_andnot_si128(a: __m128i, b: __m128i) -> __m128i { vbicq_s32(b, a) }
    fn _mm_slli_epi32(a: __m128i, n: i32) -> __m128i { vshlq_s32(a, vdupq_n_s32(n)) }
    fn _mm_srli_epi32(a: __m128i, n: i32) -> __m128i {
        vreinterpretq_s32_u32(vshlq_u32(vreinterpretq_u32_s32(a), vdupq_n_s32(-n)))
    }
    fn _mm_srai_epi32(a: __m128i, n: i32) -> __m128i { vshlq_s32(a, vdupq_n_s32(-n)) }
    // The count is the second's low lane; NEON gives 0 for a count of 32 or more, as x86.
    fn _mm_sll_epi32(a: __m128i, count: __m128i) -> __m128i {
        vshlq_s32(a, vdupq_n_s32(vgetq_lane_s32::<0>(count)))
    }

    fn _mm_add_epi16(a: __m128i, b: __m128i) -> __m128i {
        vreinterpretq_s32_s16(vaddq_s16(vreinterpretq_s16_s32(a), vreinterpretq_s16_s32(b)))
    }
    fn _mm_sub_epi16(a: __m128i, b: __m128i) -> __m128i {
        vreinterpretq_s32_s16(vsubq_s16(vreinterpretq_s16_s32(a), vreinterpretq_s16_s32(b)))
    }
    fn _mm_mullo_epi16(a: __m128i, b: __m128i) -> __m128i {
        vreinterpretq_s32_s16(vmulq_s16(vreinterpretq_s16_s32(a), vreinterpretq_s16_s32(b)))
    }
    fn _mm_srli_epi16(a: __m128i, n: i32) -> __m128i {
        vreinterpretq_s32_u16(vshlq_u16(vreinterpretq_u16_s32(a), vdupq_n_s16(-(n as i16))))
    }
}
