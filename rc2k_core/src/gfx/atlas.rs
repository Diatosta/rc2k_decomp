//! A glyph record decoded into the texels of a texture atlas, and the part of it a clip leaves as
//! a placed rectangle: what a GPU draws as one quad in place of the blitter's run-by-run writes.
//!
//! A decoded glyph is `width * height` texels of four bytes in the order a 32-bit canvas pixel
//! has them in memory (blue, green, red, alpha). A pixel of a palette glyph holds its palette
//! index in the red byte; a pixel of a direct-colour glyph holds its colour with the alpha byte
//! set; a gap holds zeroes, so the alpha byte says whether a texel is drawn. Only records whose
//! drawing is exactly the cut of this image are decoded: every row ends, no run reaches past
//! the record's width, and the whole record is held. Everything else is refused (`None`) and
//! stays with the blitter.

use crate::gfx::blit::{Plan, direct_colour};
use crate::strings::bytes::Bytes;

/// The canvas pixel that marks "not written" in a GPU's overlay: a drawn pixel of this exact
/// value would be taken for a gap there, so no glyph holding one is decoded.
pub const UNWRITTEN_PIXEL: u32 = 0xA5FF_00FF;

/// The biggest glyph decoded (a side and the area): bigger ones stay with the blitter.
const MAX_SIDE: i32 = 1024;
const MAX_AREA: i32 = 1024 * 1024;

/// A glyph record as atlas texels.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Decoded {
    pub x_offset: i32,
    pub y_offset: i32,
    pub width: i32,
    pub height: i32,
    /// Direct colour: the texels are colours. Else they are palette indices.
    pub direct: bool,
    /// The highest palette index a pixel holds (0 for a direct glyph).
    pub max_index: u8,
    /// Row by row, `width * height` texels.
    pub texels: Vec<[u8; 4]>,
}

/// Reads one field of `unit` bytes at `at`.
fn field(record: &(impl Bytes + ?Sized), at: usize, unit: usize) -> Option<u32> {
    let mut value = 0u32;
    for k in (0..unit).rev() {
        value = value << 8 | u32::from(record.byte(at.checked_add(k)?)?);
    }
    Some(value)
}

/// The glyph `record` (the bytes from its first) as texels; `None` for a record that is cut
/// short, runs past its own width, is of a kind the atlas does not hold (word-wide direct
/// colour), is too big, or holds a pixel the GPU's overlay would take for a gap.
pub fn decode(record: &(impl Bytes + ?Sized)) -> Option<Decoded> {
    let marker = record.u32_le(0)?;
    let direct = direct_colour(marker);
    let (x_offset, y_offset, width, height, mut at, unit) = if direct {
        if marker == 0xFFFF_FFFE {
            return None;
        }
        let word = |at| field(record, at, 2).map(|v| v as i32);
        (word(4)?, word(6)?, word(8)?, word(10)?, 12, 4)
    } else {
        let byte = |at| record.byte(at).map(i32::from);
        (byte(0)?, byte(1)?, byte(2)?, byte(3)?, 4, 1)
    };
    if width > MAX_SIDE || height > MAX_SIDE || width * height > MAX_AREA {
        return None;
    }
    let mut texels = vec![[0u8; 4]; (width * height) as usize];
    let mut max_index = 0u8;
    for row in 0..height {
        let mut column = 0u64;
        loop {
            let count = field(record, at, unit)?;
            at += unit;
            if count == 0 {
                break;
            }
            let gap = field(record, at, unit)?;
            at += unit;
            column += u64::from(gap);
            if column + u64::from(count) > width as u64 {
                return None;
            }
            for _ in 0..count {
                let place = (row * width) as usize + column as usize;
                texels[place] = if direct {
                    let bytes = u32::to_le_bytes(field(record, at, 4)?);
                    if u32::from_le_bytes(bytes) == UNWRITTEN_PIXEL {
                        return None;
                    }
                    [bytes[0], bytes[1], bytes[2], 0xFF]
                } else {
                    let index = record.byte(at)?;
                    max_index = max_index.max(index);
                    [0, 0, index, 0xFF]
                };
                at += unit;
                column += 1;
            }
        }
    }
    Some(Decoded {
        x_offset,
        y_offset,
        width,
        height,
        direct,
        max_index,
        texels,
    })
}

/// The part of a glyph a clip leaves, placed on the screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Placed {
    /// The screen's pixel of the rectangle's first texel.
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    /// The rectangle's first texel in the glyph.
    pub from_x: i32,
    pub from_y: i32,
}

/// The visible rectangle of a clipped glyph; `None` when the clip hides all of it.
pub fn placed(plan: &Plan) -> Option<Placed> {
    plan.drawn.then(|| Placed {
        x: plan.x.wrapping_add(plan.left_skip),
        y: plan.y.wrapping_add(plan.top_skip),
        width: plan.visible_w,
        height: plan.rows,
        from_x: plan.left_skip,
        from_y: plan.top_skip,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::blit::{Scratch, blit_glyph, plan};
    use crate::gfx::raster::{Clip, Surface};

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 33) as u32
        }

        fn between(&mut self, low: i32, high: i32) -> i32 {
            low + (self.next() % (high - low + 1) as u32) as i32
        }
    }

    /// A random record, normal or dword direct colour, with runs that fit its width.
    fn random_record(lcg: &mut Lcg, direct: bool) -> Vec<u8> {
        let unit = if direct { 4 } else { 1 };
        let put = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes()[..unit]);
        let (width, height) = (lcg.between(0, 14), lcg.between(0, 9));
        let (x_offset, y_offset) = (lcg.between(0, 3), lcg.between(0, 3));
        let mut out = if direct {
            let mut v = 0xFFFF_FFFFu32.to_le_bytes().to_vec();
            for n in [x_offset, y_offset, width, height] {
                v.extend_from_slice(&(n as u16).to_le_bytes());
            }
            v
        } else {
            vec![x_offset as u8, y_offset as u8, width as u8, height as u8]
        };
        for _ in 0..height {
            let mut column = 0;
            while column < width && !lcg.next().is_multiple_of(5) {
                let gap = lcg.between(0, 3).min(width - column - 1).max(0);
                let count = lcg.between(1, 6).min(width - column - gap);
                if count <= 0 {
                    break;
                }
                put(&mut out, count as u32);
                put(&mut out, gap as u32);
                column += gap + count;
                for _ in 0..count {
                    let value = if direct {
                        lcg.next() & 0x00FF_FFFF
                    } else {
                        lcg.between(0, 255) as u32
                    };
                    put(&mut out, value);
                }
            }
            put(&mut out, 0);
        }
        out
    }

    const SCREEN: (i32, i32) = (40, 30);

    fn surface() -> Surface {
        Surface {
            bits: 32,
            bytes_per_pixel: 4,
            pitch: SCREEN.0 * 4,
        }
    }

    /// What the blitter makes of `record`, from a canvas of 0xAA bytes; `None` when it refuses.
    fn blitted(
        record: &[u8],
        x: i32,
        y: i32,
        clip: &Clip,
        palette: &dyn Fn(u8) -> u32,
    ) -> Option<Vec<u8>> {
        let mut canvas = vec![0xAAu8; (SCREEN.0 * SCREEN.1 * 4) as usize];
        let mut scratch = Scratch::default();
        let mut outside = false;
        blit_glyph(
            record,
            x,
            y,
            palette,
            &surface(),
            clip,
            &mut scratch,
            &mut |at, bytes| match canvas.get_mut(at as usize..at as usize + bytes.len()) {
                Some(place) => place.copy_from_slice(bytes),
                None => outside = true,
            },
        )
        .ok()
        .filter(|()| !outside)?;
        Some(canvas)
    }

    /// The decoded glyph drawn the way a GPU does: the plan's rectangle of texels.
    fn quad(
        decoded: &Decoded,
        x: i32,
        y: i32,
        clip: &Clip,
        palette: &dyn Fn(u8) -> u32,
    ) -> Vec<u8> {
        let mut canvas = vec![0xAAu8; (SCREEN.0 * SCREEN.1 * 4) as usize];
        let plan = plan(
            x,
            y,
            [
                decoded.x_offset,
                decoded.y_offset,
                decoded.width,
                decoded.height,
            ],
            clip,
        );
        if let Some(rect) = placed(&plan) {
            for row in 0..rect.height {
                for column in 0..rect.width {
                    let texel = decoded.texels
                        [((rect.from_y + row) * decoded.width + rect.from_x + column) as usize];
                    if texel[3] == 0 {
                        continue;
                    }
                    let colour = if decoded.direct {
                        [texel[0], texel[1], texel[2], 0]
                    } else {
                        palette(texel[2]).to_le_bytes()
                    };
                    let at = (((rect.y + row) * SCREEN.0 + rect.x + column) * 4) as usize;
                    // A direct colour's alpha byte is not drawn by the quad, whose output has
                    // alpha 1: only the three colour bytes are compared.
                    let size = if decoded.direct { 3 } else { 4 };
                    canvas[at..at + size].copy_from_slice(&colour[..size]);
                }
            }
        }
        canvas
    }

    /// A decoded glyph drawn as a quad equals the blitter's drawing through any clip, for
    /// intact and damaged records alike; a record the blitter refuses is never decoded.
    #[test]
    fn a_decoded_glyph_drawn_as_a_quad_equals_the_blit() {
        let mut lcg = Lcg(11);
        let palette =
            |index: u8| 0x0100_0000 | 0x0001_0101u32.wrapping_mul(u32::from(index)) ^ 0x40_2010;
        let (mut decoded_count, mut refused, mut clipped) = (0, 0, 0);
        for round in 0..40_000 {
            let direct = lcg.next().is_multiple_of(3);
            let mut record = random_record(&mut lcg, direct);
            if round % 4 == 0 {
                // Damage it: a few bytes changed, or the end cut off.
                for _ in 0..lcg.between(1, 3) {
                    let at = lcg.next() as usize % record.len();
                    record[at] = lcg.next() as u8;
                }
                if lcg.next().is_multiple_of(4) {
                    record.truncate(lcg.next() as usize % record.len());
                }
            }
            let (x, y) = (
                lcg.between(-18, SCREEN.0 + 4),
                lcg.between(-12, SCREEN.1 + 4),
            );
            let (left, top) = (lcg.between(-2, 30), lcg.between(-2, 22));
            let clip = Clip {
                left: left.max(0),
                right: (left + lcg.between(-1, 36)).min(SCREEN.0 - 1),
                top: top.max(0),
                bottom: (top + lcg.between(-1, 28)).min(SCREEN.1 - 1),
            };
            let Some(decoded) = decode(&record[..]) else {
                refused += 1;
                continue;
            };
            decoded_count += 1;
            let drawn = blitted(&record, x, y, &clip, &palette)
                .unwrap_or_else(|| panic!("a decoded record the blit refuses: {record:?}"));
            let by_quad = quad(&decoded, x, y, &clip, &palette);
            let differs = drawn.iter().zip(&by_quad).enumerate().any(|(n, (a, b))| {
                // The alpha byte of a direct colour is the blitter's own; the GPU's output is opaque.
                !(decoded.direct && n % 4 == 3 && (*a != 0xAA || *b != 0xAA)) && a != b
            });
            assert!(!differs, "record {record:?} at {x},{y} clip {clip:?}");
            clipped += i32::from(
                plan(
                    x,
                    y,
                    [
                        decoded.x_offset,
                        decoded.y_offset,
                        decoded.width,
                        decoded.height,
                    ],
                    &clip,
                )
                .visible_w
                    < decoded.width,
            );
        }
        assert!(decoded_count > 15_000 && refused > 1_000 && clipped > 2_000);
    }

    #[test]
    fn records_the_atlas_does_not_hold_are_refused() {
        // Words (marker 0xFFFFFFFE).
        let mut words = 0xFFFF_FFFEu32.to_le_bytes().to_vec();
        for n in [0u16, 0, 1, 1, 1, 0, 1, 7, 0] {
            words.extend_from_slice(&n.to_le_bytes());
        }
        assert!(decode(&words[..]).is_none());
        // A run past the width.
        assert!(decode(&[0u8, 0, 2, 1, 3, 0, 1, 2, 3, 0][..]).is_none());
        // A pixel that is the overlay's "unwritten" mark.
        let mut mark = 0xFFFF_FFFFu32.to_le_bytes().to_vec();
        for n in [0u16, 0, 1, 1] {
            mark.extend_from_slice(&n.to_le_bytes());
        }
        for dword in [1u32, 0, UNWRITTEN_PIXEL, 0] {
            mark.extend_from_slice(&dword.to_le_bytes());
        }
        assert!(decode(&mark[..]).is_none());
        // The record of the blit tests: a gap of one, then two pixels.
        let glyph = decode(&[0u8, 0, 3, 2, 2, 1, 7, 8, 0, 3, 0, 1, 2, 3, 0][..]).unwrap();
        assert_eq!((glyph.width, glyph.height, glyph.max_index), (3, 2, 8));
        assert_eq!(glyph.texels[0], [0; 4]);
        assert_eq!(glyph.texels[1], [0, 0, 7, 255]);
    }
}
