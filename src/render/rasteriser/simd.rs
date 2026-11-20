//! The four-lane row path: a row's pixels four at a time on the processor's vector unit (SSE2,
//! or NEON through `rasteriser_neon`), with the same results as the scalar row.

use super::*;

/// A row's values: each (at its first place, its change a pixel).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub(super) struct Lines {
    pub(super) z: (f32, f32),
    pub(super) colour: [(f32, f32); 4],
    pub(super) fog: (f32, f32),
    pub(super) w: (f32, f32),
    pub(super) uw: (f32, f32),
    pub(super) vw: (f32, f32),
    pub(super) u: (f32, f32),
    pub(super) v: (f32, f32),
}

/// The proof the four-lane path's `unsafe` calls rest on: the build's processor has SSE2 (both
/// x86 targets' baseline, i686 and x86_64), so a function that asks for it runs on any machine
/// the build runs on.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(super) const SSE2_IN_THE_BUILD: () =
    assert!(cfg!(target_feature = "sse2"), "an x86 build without SSE2");

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _: () = SSE2_IN_THE_BUILD;

// A vector's lanes as an array and back, lane by lane (the compilers make these the one
// move they were as transmutes).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
#[target_feature(enable = "sse2")]
pub(super) fn i32s(v: __m128i) -> [i32; 4] {
    [
        _mm_cvtsi128_si32(v),
        _mm_cvtsi128_si32(_mm_srli_si128::<4>(v)),
        _mm_cvtsi128_si32(_mm_srli_si128::<8>(v)),
        _mm_cvtsi128_si32(_mm_srli_si128::<12>(v)),
    ]
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
#[target_feature(enable = "sse2")]
pub(super) fn from_u32s(a: [u32; 4]) -> __m128i {
    _mm_set_epi32(a[3] as i32, a[2] as i32, a[1] as i32, a[0] as i32)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
#[target_feature(enable = "sse2")]
pub(super) fn f32s(v: __m128) -> [f32; 4] {
    [
        _mm_cvtss_f32(v),
        _mm_cvtss_f32(_mm_shuffle_ps::<0x55>(v, v)),
        _mm_cvtss_f32(_mm_shuffle_ps::<0xAA>(v, v)),
        _mm_cvtss_f32(_mm_shuffle_ps::<0xFF>(v, v)),
    ]
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
#[target_feature(enable = "sse2")]
pub(super) fn from_f32s(a: [f32; 4]) -> __m128 {
    _mm_set_ps(a[3], a[2], a[1], a[0])
}

#[cfg(target_arch = "aarch64")]
#[inline]
pub(super) fn i32s(v: __m128i) -> [i32; 4] {
    lanes_i32(v)
}

#[cfg(target_arch = "aarch64")]
#[inline]
pub(super) fn from_u32s(a: [u32; 4]) -> __m128i {
    from_lanes_i32(a.map(|x| x as i32))
}

#[cfg(target_arch = "aarch64")]
#[inline]
pub(super) fn f32s(v: __m128) -> [f32; 4] {
    lanes_f32(v)
}

#[cfg(target_arch = "aarch64")]
#[inline]
pub(super) fn from_f32s(a: [f32; 4]) -> __m128 {
    from_lanes_f32(a)
}

/// `compare` on four lanes: their mask (all ones where it holds).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
pub(super) fn compare4(function: u32, value: __m128, reference: __m128) -> __m128 {
    match function {
        1 => _mm_setzero_ps(),
        2 => _mm_cmplt_ps(value, reference),
        3 => _mm_cmpeq_ps(value, reference),
        4 => _mm_cmple_ps(value, reference),
        5 => _mm_cmpgt_ps(value, reference),
        6 => _mm_cmpneq_ps(value, reference),
        7 => _mm_cmpge_ps(value, reference),
        _ => _mm_castsi128_ps(_mm_set1_epi32(-1)),
    }
}

/// `floor` on four lanes, as the scalar one: truncated, one less where that is above.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
pub(super) fn floor4(x: __m128) -> __m128i {
    let i = _mm_cvttps_epi32(x);
    let above = _mm_castps_si128(_mm_cmpgt_ps(_mm_cvtepi32_ps(i), x));
    // above is -1 where the truncation is above: add it
    _mm_add_epi32(i, above)
}

/// Four lanes' 0xAARRGGBB as four channels (blue, green, red, alpha) of 0 .. 1, each
/// n x (1 / 255) as `UNIT` has it.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
pub(super) fn channels4(argb: __m128i) -> [__m128; 4] {
    let byte = _mm_set1_epi32(0xFF);
    let most = _mm_set1_ps(INV_255);
    [
        _mm_mul_ps(_mm_cvtepi32_ps(_mm_and_si128(argb, byte)), most),
        _mm_mul_ps(
            _mm_cvtepi32_ps(_mm_and_si128(_mm_srli_epi32(argb, 8), byte)),
            most,
        ),
        _mm_mul_ps(
            _mm_cvtepi32_ps(_mm_and_si128(_mm_srli_epi32(argb, 16), byte)),
            most,
        ),
        _mm_mul_ps(_mm_cvtepi32_ps(_mm_srli_epi32(argb, 24)), most),
    ]
}

/// The scalar `lerp` of `Texels::bilinear` on four lanes: two channels at a time in 16-bit
/// halves (each product at most 255 x 256, so the same numbers).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
pub(super) fn lerp4(a: __m128i, b: __m128i, w: __m128i) -> __m128i {
    let mask = _mm_set1_epi32(0x00FF_00FF);
    // w in both halves of each lane, and 256 - w
    let w2 = _mm_or_si128(w, _mm_slli_epi32(w, 16));
    let rest = _mm_sub_epi16(_mm_set1_epi16(256), w2);
    let blend =
        |x: __m128i, y: __m128i| _mm_add_epi16(_mm_mullo_epi16(x, rest), _mm_mullo_epi16(y, w2));
    let rb = _mm_and_si128(
        _mm_srli_epi16(blend(_mm_and_si128(a, mask), _mm_and_si128(b, mask)), 8),
        mask,
    );
    let ag = _mm_andnot_si128(
        mask,
        blend(
            _mm_and_si128(_mm_srli_epi32(a, 8), mask),
            _mm_and_si128(_mm_srli_epi32(b, 8), mask),
        ),
    );
    _mm_or_si128(rb, ag)
}

/// `factor` on four lanes: each channel's factor (blue, green, red, alpha).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
pub(super) fn factor4(blend: u32, source: &[__m128; 4], target: &[__m128; 4]) -> [__m128; 4] {
    let one = _mm_set1_ps(1.0);
    let less = |x: __m128| _mm_sub_ps(one, x);
    match blend {
        1 => [_mm_setzero_ps(); 4],
        3 => *source,
        4 => source.map(less),
        5 => [source[3]; 4],
        6 => [less(source[3]); 4],
        7 => [target[3]; 4],
        8 => [less(target[3]); 4],
        9 => *target,
        10 => target.map(less),
        11 => {
            let f = _mm_min_ps(source[3], less(target[3]));
            [f, f, f, one]
        }
        _ => [one; 4],
    }
}

/// `read` on four places from x (lanes past `n` read as 0): the channels as 0 .. 1,
/// alpha 1, each by the same multiplication as the scalar one.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
impl Cells<'_> {
    #[inline]
    #[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
    pub(super) fn read4(&self, x: usize, y: usize, n: usize) -> [__m128; 4] {
        let mut p = [0u32; 4];
        for (lane, p) in p.iter_mut().enumerate().take(n) {
            *p = self.pixel(x + lane, y);
        }
        let p = from_u32s(p);
        let part = |mask: i32, shift: i32, most: f32| {
            let v = _mm_and_si128(p, _mm_set1_epi32(mask));
            let v = match shift {
                0 => v,
                8 => _mm_srli_epi32(v, 8),
                _ => _mm_srli_epi32(v, 16),
            };
            _mm_mul_ps(_mm_cvtepi32_ps(v), _mm_set1_ps(1.0 / most))
        };
        let one = _mm_set1_ps(1.0);
        [
            part(0xFF, 0, 255.0),
            part(0xFF00, 8, 255.0),
            part(0xFF_0000, 16, 255.0),
            one,
        ]
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
impl Triangle<'_> {
    /// `row`'s places `first ..= last` four at a time with SSE2: the same
    /// numbers as the scalar loop, but the texels' reads, which stay one by one.
    #[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
    pub(super) fn row4(&self, y: usize, first: usize, last: usize, l: &Lines) -> u64 {
        let tile = self.tile;
        let mut written = 0u64;
        let lanes = _mm_set_ps(3.0, 2.0, 1.0, 0.0);
        let mut x0 = first;
        while x0 <= last {
            let n = (last + 1 - x0).min(4);
            let k = _mm_add_ps(_mm_set1_ps((x0 - first) as f32), lanes);
            let at =
                |(v0, d): (f32, f32)| _mm_add_ps(_mm_set1_ps(v0), _mm_mul_ps(_mm_set1_ps(d), k));
            let mut live: i32 = (1 << n) - 1;
            let z = at(l.z);
            let mut old = [1.0f32; 4];
            for (lane, old) in old.iter_mut().enumerate().take(n) {
                *old = tile.depth(x0 + lane, y);
            }
            if self.depth_test {
                live &= _mm_movemask_ps(compare4(self.depth_function, z, from_f32s(old)));
            }
            if live == 0 {
                x0 += 4;
                continue;
            }
            let colour = l.colour.map(at);
            let mut out = colour;
            if let Some(texels) = &self.texture {
                let texture = texels.texture;
                let w = at(l.w);
                let zero_w = _mm_cmpeq_ps(w, _mm_setzero_ps());
                let over = _mm_div_ps(_mm_set1_ps(1.0), w);
                let pick = |perspective: __m128, plain: __m128| {
                    _mm_or_ps(
                        _mm_and_ps(zero_w, plain),
                        _mm_andnot_ps(zero_w, perspective),
                    )
                };
                let u = pick(_mm_mul_ps(at(l.uw), over), at(l.u));
                let v = pick(_mm_mul_ps(at(l.vw), over), at(l.v));
                let (tw, th) = (
                    _mm_set1_ps(texture.width as f32),
                    _mm_set1_ps(texture.height as f32),
                );
                let argb = if self.linear {
                    let half = _mm_set1_ps(0.5);
                    let tx = _mm_sub_ps(_mm_mul_ps(u, tw), half);
                    let ty = _mm_sub_ps(_mm_mul_ps(v, th), half);
                    let (ix, iy) = (floor4(tx), floor4(ty));
                    let scale = _mm_set1_ps(256.0);
                    let wx =
                        _mm_cvttps_epi32(_mm_mul_ps(_mm_sub_ps(tx, _mm_cvtepi32_ps(ix)), scale));
                    let wy =
                        _mm_cvttps_epi32(_mm_mul_ps(_mm_sub_ps(ty, _mm_cvtepi32_ps(iy)), scale));
                    let (xs, ys) = (i32s(ix), i32s(iy));
                    let [t00, t10, t01, t11] = match texels.quad4(ix, iy) {
                        Some(quad) => quad,
                        None => {
                            let (xs1, ys1) = (xs.map(|x| x + 1), ys.map(|y| y + 1));
                            [
                                texels.fetch4(&xs, &ys, live),
                                texels.fetch4(&xs1, &ys, live),
                                texels.fetch4(&xs, &ys1, live),
                                texels.fetch4(&xs1, &ys1, live),
                            ]
                        }
                    };
                    let load = |t: &[u32; 4]| from_u32s(*t);
                    let top = lerp4(load(&t00), load(&t10), wx);
                    let bottom = lerp4(load(&t01), load(&t11), wx);
                    lerp4(top, bottom, wy)
                } else {
                    let (ix, iy) = (floor4(_mm_mul_ps(u, tw)), floor4(_mm_mul_ps(v, th)));
                    let (xs, ys) = (i32s(ix), i32s(iy));
                    let t = texels.fetch4(&xs, &ys, live);
                    from_u32s(t)
                };
                let texel = channels4(argb);
                out = match self.mixing {
                    1 | 7 => texel,
                    4 => [0, 1, 2, 3].map(|c| _mm_mul_ps(texel[c], colour[c])),
                    _ => [
                        _mm_mul_ps(texel[0], colour[0]),
                        _mm_mul_ps(texel[1], colour[1]),
                        _mm_mul_ps(texel[2], colour[2]),
                        if texture.has_alpha() {
                            texel[3]
                        } else {
                            colour[3]
                        },
                    ],
                };
            }
            if self.alpha_test {
                live &= _mm_movemask_ps(compare4(
                    self.alpha_function,
                    out[3],
                    _mm_set1_ps(self.alpha_reference),
                ));
                if live == 0 {
                    x0 += 4;
                    continue;
                }
            }
            if self.fog {
                let kept = _mm_min_ps(_mm_max_ps(at(l.fog), _mm_setzero_ps()), _mm_set1_ps(1.0));
                let rest = _mm_sub_ps(_mm_set1_ps(1.0), kept);
                for (o, &f) in out.iter_mut().zip(&self.fog_colour).take(3) {
                    *o = _mm_add_ps(_mm_mul_ps(*o, kept), _mm_mul_ps(_mm_set1_ps(f), rest));
                }
            }
            if self.blending {
                let under = tile.read4(x0, y, n);
                let (sf, df) = (
                    factor4(self.source_blend, &out, &under),
                    factor4(self.target_blend, &out, &under),
                );
                out = [0, 1, 2, 3]
                    .map(|c| _mm_add_ps(_mm_mul_ps(out[c], sf[c]), _mm_mul_ps(under[c], df[c])));
            }
            // write_target's numbers: (clamped * most + 0.5) truncated
            let to = |c: __m128, most: f32| {
                let c = _mm_min_ps(_mm_max_ps(c, _mm_setzero_ps()), _mm_set1_ps(1.0));
                _mm_cvttps_epi32(_mm_add_ps(
                    _mm_mul_ps(c, _mm_set1_ps(most)),
                    _mm_set1_ps(0.5),
                ))
            };
            let packed = _mm_or_si128(
                _mm_or_si128(
                    _mm_slli_epi32(to(out[2], 255.0), 16),
                    _mm_slli_epi32(to(out[1], 255.0), 8),
                ),
                to(out[0], 255.0),
            );
            let pixels = i32s(packed).map(|p| p as u32);
            let zs = f32s(z);
            for lane in 0..n {
                if live & (1 << lane) == 0 {
                    continue;
                }
                let x = x0 + lane;
                tile.put_pixel(x, y, pixels[lane]);
                if self.depth_write {
                    tile.set_depth(x, y, zs[lane]);
                }
                written += 1;
            }
            x0 += 4;
        }
        written
    }
}

/// SSE2's lane-wise `min` and `max` of signed dwords (SSE4.1 has them).
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
#[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
pub(super) fn clamp4(v: __m128i, most: i32) -> __m128i {
    // not below 0: zero where the sign is set
    let v = _mm_andnot_si128(_mm_srai_epi32(v, 31), v);
    let most = _mm_set1_epi32(most);
    let over = _mm_cmpgt_epi32(v, most);
    _mm_or_si128(_mm_and_si128(over, most), _mm_andnot_si128(over, v))
}
