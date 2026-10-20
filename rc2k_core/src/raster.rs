//! Clipping and solid shapes on a surface (converted 2026-10-20 from `gfx::set_clip`
//! 0x406988, `gfx::fill_rect` 0x406869, `draw_hline` 0x406710, `draw_vline` 0x4067AF,
//! `frame_rect` 0x4066DF): the message box's fill and frame. A shape becomes a `Plan`: the
//! work variables the original leaves (`g_blitLeftSkip`, `g_blitVisibleWidth`,
//! `g_blitTopSkip`, `g_blitRowsLeft`; only those it writes before it stops) and the pixels
//! to set, as byte offsets from the surface's start.

use crate::Invalid;

/// The clip rectangle, its edges included.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Clip {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

/// (left, top, right, bottom) for a rectangle of x, y, width, height (`set_clip`).
pub fn clip_of([x, y, width, height]: [u32; 4]) -> [u32; 4] {
    [
        x,
        y,
        x.wrapping_add(width).wrapping_sub(1),
        y.wrapping_add(height).wrapping_sub(1),
    ]
}

/// What a shape needs of the surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Surface {
    pub bits: u32,
    pub bytes_per_pixel: i32,
    pub pitch: i32,
}

/// The work variables a shape leaves; `None` for one it does not write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Work {
    pub left_skip: Option<i32>,
    pub width: Option<i32>,
    pub top_skip: Option<i32>,
    pub rows: Option<i32>,
}

/// A shape's work variables and pixels (byte offsets, written at `bits`).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Plan {
    pub work: Work,
    pub bits: u32,
    pub pixels: Vec<i32>,
}

/// The step from one pixel to the next at a depth.
fn step(bits: u32) -> i32 {
    match bits {
        16 => 2,
        32 => 4,
        _ => 1,
    }
}

/// Rectangle x, y, width, height filled, clipped (`fill_rect`). As the original: on a
/// 16-bit surface; on a 32-bit one the work is done and nothing drawn (`notes/bugs.md`).
/// Refused (before anything) at other depths: on 8 bits the original fills past the rows.
pub fn fill(rect: [i32; 4], clip: &Clip, s: &Surface) -> Result<Plan, Invalid> {
    if s.bits != 16 && s.bits != 32 {
        return Err(Invalid(
            "a rectangle on a surface of neither 16 nor 32 bits",
        ));
    }
    let [x, y, w, h] = rect;
    let mut plan = Plan {
        bits: 16,
        ..Plan::default()
    };
    let left_skip = if x.wrapping_sub(clip.left) < 0 {
        clip.left.wrapping_sub(x)
    } else {
        0
    };
    plan.work.left_skip = Some(left_skip);
    let mut width = w;
    let over = x.wrapping_add(w).wrapping_sub(1).wrapping_sub(clip.right);
    if over > 0 {
        width = width.wrapping_sub(over);
    }
    width = width.wrapping_sub(left_skip);
    if width <= 0 {
        return Ok(plan);
    }
    plan.work.width = Some(width);
    let top_skip = if y.wrapping_sub(clip.top) < 0 {
        clip.top.wrapping_sub(y)
    } else {
        0
    };
    plan.work.top_skip = Some(top_skip);
    let mut rows = h;
    let over = y.wrapping_add(h).wrapping_sub(1).wrapping_sub(clip.bottom);
    if over > 0 {
        rows = rows.wrapping_sub(over);
    }
    rows = rows.wrapping_sub(top_skip);
    if rows <= 0 {
        return Ok(plan);
    }
    plan.work.rows = Some(rows);
    if s.bits == 32 {
        return Ok(plan);
    }
    let px = x.wrapping_add(left_skip).wrapping_mul(s.bytes_per_pixel);
    let mut row = y
        .wrapping_add(top_skip)
        .wrapping_mul(s.pitch)
        .wrapping_add(px);
    for _ in 0..rows {
        plan.pixels
            .extend((0..width).map(|k| row.wrapping_add(2 * k)));
        row = row.wrapping_add(s.pitch);
    }
    Ok(plan)
}

/// `length` pixels from x, y rightwards, clipped (`draw_hline`; the left clip as the
/// original: what is cut is -x, right only for a left edge of 0).
pub fn hline(x: i32, y: i32, length: i32, clip: &Clip, s: &Surface) -> Plan {
    let mut plan = Plan {
        bits: s.bits,
        ..Plan::default()
    };
    if y < clip.top || y > clip.bottom {
        return plan;
    }
    let left_skip = if x < clip.left { x.wrapping_neg() } else { 0 };
    plan.work.left_skip = Some(left_skip);
    let mut width = length;
    let over = x
        .wrapping_add(length)
        .wrapping_sub(1)
        .wrapping_sub(clip.right);
    if over > 0 {
        width = width.wrapping_sub(over);
    }
    width = width.wrapping_sub(left_skip);
    if width <= 0 {
        return plan;
    }
    plan.work.width = Some(width);
    let px = x.wrapping_add(left_skip).wrapping_mul(s.bytes_per_pixel);
    let start = y.wrapping_mul(s.pitch).wrapping_add(px);
    plan.pixels = (0..width)
        .map(|k| start.wrapping_add(step(s.bits) * k))
        .collect();
    plan
}

/// `length` pixels from x, y downwards, clipped (`draw_vline`; the top clip as the
/// original: what is cut is -y).
pub fn vline(x: i32, y: i32, length: i32, clip: &Clip, s: &Surface) -> Plan {
    let mut plan = Plan {
        bits: s.bits,
        ..Plan::default()
    };
    if x < clip.left || x > clip.right {
        return plan;
    }
    let top_skip = if y < clip.top { y.wrapping_neg() } else { 0 };
    plan.work.top_skip = Some(top_skip);
    let mut rows = length;
    let over = y
        .wrapping_add(length)
        .wrapping_sub(1)
        .wrapping_sub(clip.bottom);
    if over > 0 {
        rows = rows.wrapping_sub(over);
    }
    rows = rows.wrapping_sub(top_skip);
    if rows <= 0 {
        return plan;
    }
    plan.work.rows = Some(rows);
    let mut p = y
        .wrapping_add(top_skip)
        .wrapping_mul(s.pitch)
        .wrapping_add(x.wrapping_mul(s.bytes_per_pixel));
    for _ in 0..rows {
        plan.pixels.push(p);
        p = p.wrapping_add(s.pitch);
    }
    plan
}

/// A rectangle's outline moved by dx, dy (`frame_rect`): the top and bottom rows, then the
/// left and right columns between them, in that order.
pub fn frame(rect: [i32; 4], dx: i32, dy: i32, clip: &Clip, s: &Surface) -> [Plan; 4] {
    let [x, y, w, h] = rect;
    let (x, y) = (dx.wrapping_add(x), dy.wrapping_add(y));
    let bottom = y.wrapping_add(h).wrapping_sub(1);
    let side = h.wrapping_sub(2);
    let top = bottom.wrapping_sub(side);
    [
        hline(x, y, w, clip, s),
        hline(x, bottom, w, clip, s),
        vline(x, top, side, clip, s),
        vline(x.wrapping_add(w).wrapping_sub(1), top, side, clip, s),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIP: Clip = Clip {
        left: 0,
        right: 63,
        top: 0,
        bottom: 31,
    };
    const S16: Surface = Surface {
        bits: 16,
        bytes_per_pixel: 2,
        pitch: 136,
    };

    #[test]
    fn rectangles() {
        assert_eq!(clip_of([10, 20, 5, 6]), [10, 20, 14, 25]);
        let p = fill([3, 2, 2, 2], &CLIP, &S16).unwrap();
        assert_eq!(
            p.pixels,
            [2 * 136 + 6, 2 * 136 + 8, 3 * 136 + 6, 3 * 136 + 8]
        );
        assert_eq!(
            p.work,
            Work {
                left_skip: Some(0),
                width: Some(2),
                top_skip: Some(0),
                rows: Some(2)
            }
        );
        let p = fill([-2, 0, 4, 1], &CLIP, &S16).unwrap();
        assert_eq!(p.pixels, [0, 2]);
        let p = fill([70, 0, 4, 1], &CLIP, &S16).unwrap();
        assert_eq!(
            p.work,
            Work {
                left_skip: Some(0),
                ..Work::default()
            }
        );
        let s32 = Surface {
            bits: 32,
            bytes_per_pixel: 4,
            pitch: 264,
        };
        assert!(fill([0, 0, 4, 4], &CLIP, &s32).unwrap().pixels.is_empty());
        let s8 = Surface {
            bits: 8,
            bytes_per_pixel: 1,
            pitch: 64,
        };
        assert!(fill([0, 0, 4, 4], &CLIP, &s8).is_err());
    }

    #[test]
    fn lines_and_frames() {
        assert_eq!(hline(62, 1, 4, &CLIP, &S16).pixels, [136 + 124, 136 + 126]);
        assert!(hline(0, 40, 4, &CLIP, &S16).pixels.is_empty());
        assert_eq!(
            vline(1, 30, 4, &CLIP, &S16).pixels,
            [30 * 136 + 2, 31 * 136 + 2]
        );
        let f = frame([2, 2, 4, 3], 0, 0, &CLIP, &S16);
        assert_eq!(f[0].pixels.len(), 4);
        assert_eq!(f[2].pixels, [3 * 136 + 4]);
        assert_eq!(f[3].pixels, [3 * 136 + 10]);
    }
}
