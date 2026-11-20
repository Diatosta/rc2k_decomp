//! Reading a texture's texels for the rasteriser: the colour-mask tables, the formats, and the
//! nearest and bilinear fetches, four lanes at a time where the processor has them.

use super::*;

/// The 16-bit formats' tables met so far: each pixel's 0xAARRGGBB, by the four masks.
pub(super) type Table = std::sync::Arc<Vec<u32>>;

pub(super) static TABLES: std::sync::Mutex<Vec<([u32; 4], Table)>> =
    std::sync::Mutex::new(Vec::new());

/// A pixel of `masks` as 0xAARRGGBB (a channel without a mask: 0xFF).
pub(crate) fn to_argb(pixel: u32, masks: &[u32; 4]) -> u32 {
    let mut out = 0;
    // masks: blue, green, red, alpha; out's bytes: blue, green, red, alpha
    for (n, &mask) in masks.iter().enumerate() {
        let c = if mask == 0 {
            255
        } else {
            let shift = mask.trailing_zeros();
            let most = mask >> shift;
            (((pixel & mask) >> shift) * 255 + most / 2) / most
        };
        out |= c << (8 * n);
    }
    out
}

pub(crate) fn table_for(masks: &[u32; 4]) -> Table {
    let mut tables = TABLES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, t)) = tables.iter().find(|(m, _)| m == masks) {
        return t.clone();
    }
    let t: Table = std::sync::Arc::new((0..0x1_0000u32).map(|p| to_argb(p, masks)).collect());
    tables.push((*masks, t.clone()));
    t
}

/// How a texture's raw pixel becomes 0xAARRGGBB.
pub(super) enum Kind {
    /// A 16-bit format: its table.
    Table(Table),
    /// A 32-bit A8R8G8B8 (or X8R8G8B8: the alpha made full by this).
    Usual(u32),
    /// Any other: channel by channel.
    Other,
}

/// A texture read as 0xAARRGGBB, everything a texel's read needs worked out once for the
/// triangle: its place, wrapped (by masks for sizes of powers of two) or clamped, read
/// without a check when the texture's bytes hold every place (else checked).
pub(super) struct Texels<'a> {
    pub(super) texture: &'a Texture<'a>,
    kind: Kind,
    clamp: bool,
    /// The masks of the wrap, for sizes of powers of two.
    masks: Option<(i32, i32)>,
    /// Every place is within the bytes.
    whole: bool,
    /// The pitch's and a texel's bytes as shifts, when both are powers of two.
    #[cfg_attr(
        not(any(target_arch = "x86", target_arch = "x86_64")),
        allow(dead_code)
    )] // the SSE path's
    shifts: Option<(i32, i32)>,
}

impl<'a> Texels<'a> {
    pub(super) fn of(texture: &'a Texture<'a>, clamp: bool) -> Self {
        let m = texture.masks;
        let kind = if texture.bytes == 2 {
            Kind::Table(table_for(&m))
        } else if texture.bytes == 4
            && m[0] == 0xFF
            && m[1] == 0xFF00
            && m[2] == 0xFF_0000
            && (m[3] == 0 || m[3] == 0xFF00_0000)
        {
            Kind::Usual(if m[3] == 0 { 0xFF00_0000 } else { 0 })
        } else {
            Kind::Other
        };
        let (w, h) = (texture.width.max(1), texture.height.max(1));
        let needed = texture.pitch * (h - 1) + w * texture.bytes.max(1);
        Self {
            texture,
            kind,
            clamp,
            masks: (w.is_power_of_two() && h.is_power_of_two())
                .then_some((w as i32 - 1, h as i32 - 1)),
            whole: matches!(texture.bytes, 2 | 4) && texture.pixels.len() >= needed,
            shifts: (texture.pitch.is_power_of_two() && texture.bytes.is_power_of_two()).then(
                || {
                    (
                        texture.pitch.trailing_zeros() as i32,
                        texture.bytes.trailing_zeros() as i32,
                    )
                },
            ),
        }
    }

    /// Texel (x, y), wrapped or clamped, as 0xAARRGGBB.
    #[inline(always)]
    pub(super) fn argb(&self, x: i32, y: i32) -> u32 {
        let t = self.texture;
        let (x, y) = match (self.clamp, self.masks) {
            (true, _) => (
                x.clamp(0, t.width as i32 - 1) as usize,
                y.clamp(0, t.height as i32 - 1) as usize,
            ),
            (false, Some((mx, my))) => ((x & mx) as usize, (y & my) as usize),
            (false, None) => (
                x.rem_euclid(t.width.max(1) as i32) as usize,
                y.rem_euclid(t.height.max(1) as i32) as usize,
            ),
        };
        let place = y * t.pitch + x * t.bytes;
        let raw = if self.whole {
            // `whole`: the bytes hold every place (checked in `of`), so the reads are inside.
            if t.bytes == 2 {
                texel16(t.pixels, place)
            } else {
                texel32(t.pixels, place)
            }
        } else {
            t.texel(x as i32, y as i32, true)
        };
        match &self.kind {
            Kind::Table(table) => table[raw as usize & 0xFFFF],
            Kind::Usual(full) => raw | full,
            Kind::Other => to_argb(raw, &t.masks),
        }
    }

    /// Four texels' 0xAARRGGBB (the lanes of `live`; the others 0): `argb` four times, but
    /// the usual cases (a whole texture, wrapped by powers of two or clamped, 32-bit
    /// A8R8G8B8 or a 16-bit table) chosen once for the four.
    #[inline(always)]
    #[cfg_attr(
        not(any(target_arch = "x86", target_arch = "x86_64")),
        allow(dead_code)
    )] // the SSE path's
    pub(super) fn fetch4(&self, xs: &[i32; 4], ys: &[i32; 4], live: i32) -> [u32; 4] {
        let t = self.texture;
        let mut out = [0u32; 4];
        let place = |x: i32, y: i32| -> usize {
            match (self.clamp, self.masks) {
                (true, _) => {
                    y.clamp(0, t.height as i32 - 1) as usize * t.pitch
                        + x.clamp(0, t.width as i32 - 1) as usize * t.bytes
                }
                (false, Some((mx, my))) => {
                    (y & my) as usize * t.pitch + (x & mx) as usize * t.bytes
                }
                (false, None) => 0,
            }
        };
        let fast = self.whole && (self.clamp || self.masks.is_some());
        match (&self.kind, fast) {
            (Kind::Usual(full), true) => {
                for lane in 0..4 {
                    if live >> lane & 1 != 0 {
                        // `whole`, and the place wrapped or clamped into the texture.
                        out[lane] = texel32(t.pixels, place(xs[lane], ys[lane])) | full;
                    }
                }
            }
            (Kind::Table(table), true) => {
                for lane in 0..4 {
                    if live >> lane & 1 != 0 {
                        out[lane] = table[texel16(t.pixels, place(xs[lane], ys[lane])) as usize];
                    }
                }
            }
            _ => {
                for lane in 0..4 {
                    if live >> lane & 1 != 0 {
                        out[lane] = self.argb(xs[lane], ys[lane]);
                    }
                }
            }
        }
        out
    }

    pub(super) fn colour(argb: u32) -> Colour {
        [
            UNIT[(argb & 0xFF) as usize],
            UNIT[(argb >> 8 & 0xFF) as usize],
            UNIT[(argb >> 16 & 0xFF) as usize],
            UNIT[(argb >> 24) as usize],
        ]
    }

    /// The texel at (u, v), the nearest one.
    pub(super) fn nearest(&self, u: f32, v: f32) -> Colour {
        let t = self.texture;
        let x = floor(u * t.width as f32);
        let y = floor(v * t.height as f32);
        Self::colour(self.argb(x, y))
    }

    /// The colour at (u, v) filtered from the four texels around it (Direct3D's linear
    /// filter: texel centres at half places), with 8-bit weights, two channels at a time.
    pub(super) fn bilinear(&self, u: f32, v: f32) -> Colour {
        let t = self.texture;
        let tx = u * t.width as f32 - 0.5;
        let ty = v * t.height as f32 - 0.5;
        let (x, y) = (floor(tx), floor(ty));
        let wx = ((tx - x as f32) * 256.0) as i32 as u32;
        let wy = ((ty - y as f32) * 256.0) as i32 as u32;
        let lerp = |a: u32, b: u32, w: u32| -> u32 {
            let rb = ((a & 0x00FF_00FF) * (256 - w) + (b & 0x00FF_00FF) * w) >> 8 & 0x00FF_00FF;
            let ag =
                ((a >> 8 & 0x00FF_00FF) * (256 - w) + (b >> 8 & 0x00FF_00FF) * w) & 0xFF00_FF00;
            rb | ag
        };
        let top = lerp(self.argb(x, y), self.argb(x + 1, y), wx);
        let bottom = lerp(self.argb(x, y + 1), self.argb(x + 1, y + 1), wx);
        Self::colour(lerp(top, bottom, wy))
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
impl Texels<'_> {
    /// The four corners' texels (x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1) for four
    /// lanes, as `argb` gives them, with the places worked out four at a time: for a whole
    /// texture of a pitch and texel of powers of two, wrapped by masks or clamped (every lane
    /// read: wrapping and clamping keep every place inside); else None.
    #[inline]
    #[cfg_attr(not(target_arch = "aarch64"), target_feature(enable = "sse2"))]
    pub(super) fn quad4(&self, x: __m128i, y: __m128i) -> Option<[[u32; 4]; 4]> {
        let (pitch_shift, byte_shift) = self.shifts?;
        if !self.whole || matches!(self.kind, Kind::Other) {
            return None;
        }
        let one = _mm_set1_epi32(1);
        let (x1, y1) = (_mm_add_epi32(x, one), _mm_add_epi32(y, one));
        let t = self.texture;
        let (x0, x1, y0, y1) = if self.clamp {
            let (w, h) = (t.width as i32 - 1, t.height as i32 - 1);
            (clamp4(x, w), clamp4(x1, w), clamp4(y, h), clamp4(y1, h))
        } else {
            let (mx, my) = self.masks?;
            let (mx, my) = (_mm_set1_epi32(mx), _mm_set1_epi32(my));
            (
                _mm_and_si128(x, mx),
                _mm_and_si128(x1, mx),
                _mm_and_si128(y, my),
                _mm_and_si128(y1, my),
            )
        };
        let shift = |v: __m128i, by: i32| _mm_sll_epi32(v, _mm_cvtsi32_si128(by));
        let (r0, r1) = (shift(y0, pitch_shift), shift(y1, pitch_shift));
        let (c0, c1) = (shift(x0, byte_shift), shift(x1, byte_shift));
        let places = [
            _mm_add_epi32(r0, c0),
            _mm_add_epi32(r0, c1),
            _mm_add_epi32(r1, c0),
            _mm_add_epi32(r1, c1),
        ];
        let mut out = [[0u32; 4]; 4];
        for (corner, p) in places.iter().enumerate() {
            let at = i32s(*p);
            for lane in 0..4 {
                // `whole` (every place of the texture within its bytes, `of`), and the places
                // wrapped by the masks or clamped into the texture above.
                let at = at[lane] as u32 as usize;
                out[corner][lane] = match &self.kind {
                    Kind::Usual(full) => texel32(t.pixels, at) | full,
                    Kind::Table(table) => table[texel16(t.pixels, at) as usize],
                    Kind::Other => 0,
                };
            }
        }
        Some(out)
    }
}
