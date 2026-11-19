//! Clipping and solid shapes on a surface: the message box's fill and frame. A shape becomes a
//! `Plan`: the blitter's work variables it leaves (only those written before it stops) and the
//! pixels to set, as byte offsets from the surface's start.

use crate::Invalid;

/// The clip rectangle, its edges included.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Clip {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

/// The clip's (left, top, right, bottom) for a rectangle of x, y, width, height.
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

/// Rectangle x, y, width, height filled, clipped, at the surface's depth.
pub fn fill(rect: [i32; 4], clip: &Clip, s: &Surface) -> Result<Plan, Invalid> {
    let (mut plan, rows) = fill_runs(rect, clip, s)?;
    plan.pixels = expand(&rows, s);
    Ok(plan)
}

/// `fill` as a plan without pixels and a run of pixels for each row: (first offset, pixels).
pub fn fill_runs(
    rect: [i32; 4],
    clip: &Clip,
    s: &Surface,
) -> Result<(Plan, Vec<(i32, usize)>), Invalid> {
    if !matches!(s.bits, 8 | 16 | 32) {
        return Err(Invalid("a rectangle on a surface of an unknown depth"));
    }
    let [x, y, w, h] = rect;
    let mut plan = Plan {
        bits: s.bits,
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
        return Ok((plan, Vec::new()));
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
        return Ok((plan, Vec::new()));
    }
    plan.work.rows = Some(rows);
    let px = x.wrapping_add(left_skip).wrapping_mul(s.bytes_per_pixel);
    let mut row = y
        .wrapping_add(top_skip)
        .wrapping_mul(s.pitch)
        .wrapping_add(px);
    let mut runs = Vec::with_capacity(rows as usize);
    for _ in 0..rows {
        runs.push((row, width as usize));
        row = row.wrapping_add(s.pitch);
    }
    Ok((plan, runs))
}

/// The pixels' byte offsets of runs (first offset, pixels) at the surface's depth.
fn expand(runs: &[(i32, usize)], s: &Surface) -> Vec<i32> {
    runs.iter()
        .flat_map(|&(first, count)| {
            (0..count as i32).map(move |k| first.wrapping_add(step(s.bits) * k))
        })
        .collect()
}

/// `length` pixels from x, y rightwards, clipped. Kept: at the left clip what is cut is -x,
/// which is right only for a left edge of 0.
pub fn hline(x: i32, y: i32, length: i32, clip: &Clip, s: &Surface) -> Plan {
    let (mut plan, runs) = hline_runs(x, y, length, clip, s);
    plan.pixels = expand(&runs, s);
    plan
}

/// `hline` as a plan without pixels and its one run of pixels (none when nothing is drawn).
pub fn hline_runs(
    x: i32,
    y: i32,
    length: i32,
    clip: &Clip,
    s: &Surface,
) -> (Plan, Vec<(i32, usize)>) {
    let mut plan = Plan {
        bits: s.bits,
        ..Plan::default()
    };
    if y < clip.top || y > clip.bottom {
        return (plan, Vec::new());
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
        return (plan, Vec::new());
    }
    plan.work.width = Some(width);
    let px = x.wrapping_add(left_skip).wrapping_mul(s.bytes_per_pixel);
    let start = y.wrapping_mul(s.pitch).wrapping_add(px);
    (plan, vec![(start, width as usize)])
}

/// `length` pixels from x, y downwards, clipped; at the top clip what is cut is -y.
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

/// A rectangle's outline moved by dx, dy: the top and bottom rows, then the
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

/// A plan's pixel offsets as runs of (first offset, pixels): neighbours `step` bytes apart, in the
/// order listed, join into one run, so a solid shape is written a row at a time instead of a pixel
/// at a time. Writing each run's pixels in order is writing each offset's pixel in order.
pub fn runs(offsets: &[i32], step: i32) -> Vec<(i32, usize)> {
    let mut runs: Vec<(i32, usize)> = Vec::new();
    for &offset in offsets {
        match runs.last_mut() {
            Some((first, count))
                if first.wrapping_add(step.wrapping_mul(*count as i32)) == offset =>
            {
                *count += 1;
            }
            _ => runs.push((offset, 1)),
        }
    }
    runs
}

/// The pixels at byte `offsets` of a surface (`bytes_per_pixel` each, rows `pitch` bytes apart,
/// `rows` rows) as rectangles (x, y, width, height) that cover exactly those pixels when the
/// offsets come as a plan lists them: a row's neighbours join into a run, and runs of one width
/// on consecutive rows join into a taller rectangle. `None` when an offset is not a pixel of the
/// surface.
pub fn pixel_rectangles(
    offsets: &[i32],
    bytes_per_pixel: i32,
    pitch: i32,
    rows: i32,
) -> Option<Vec<[i32; 4]>> {
    if bytes_per_pixel <= 0 || pitch < bytes_per_pixel || pitch % bytes_per_pixel != 0 {
        return None;
    }
    let width = pitch / bytes_per_pixel;
    let mut rectangles: Vec<[i32; 4]> = Vec::new();
    // The open run: its first pixel's column and row, and its length.
    let mut run: Option<(i32, i32, i32)> = None;
    let close = |run: Option<(i32, i32, i32)>, rectangles: &mut Vec<[i32; 4]>| {
        if let Some((x, y, length)) = run {
            match rectangles.last_mut() {
                Some([lx, ly, lw, lh]) if *lx == x && *lw == length && *ly + *lh == y => *lh += 1,
                _ => rectangles.push([x, y, length, 1]),
            }
        }
    };
    for &offset in offsets {
        if offset < 0 || offset % bytes_per_pixel != 0 {
            return None;
        }
        let (y, x) = ((offset / pitch), (offset % pitch) / bytes_per_pixel);
        if y >= rows || x >= width {
            return None;
        }
        run = match run {
            Some((rx, ry, length)) if ry == y && rx + length == x => Some((rx, ry, length + 1)),
            other => {
                close(other, &mut rectangles);
                Some((x, y, 1))
            }
        };
    }
    close(run, &mut rectangles);
    Some(rectangles)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny generator for the tests: the same numbers each run.
    fn next(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state >> 33
    }

    /// Every pixel written alone (what the canvas did before) and every run written whole come out
    /// the same on random rectangles, lines and clip windows at the three depths.
    #[test]
    fn runs_paint_what_pixels_paint() {
        let mut rng = 7u64;
        for bits in [8u32, 16, 32] {
            let bpp = (bits / 8) as i32;
            let s = Surface {
                bits,
                bytes_per_pixel: bpp,
                // Some rows with a gap after the pixels, one packed.
                pitch: if bits == 16 { 64 * 2 } else { 64 * bpp + 8 },
            };
            let size = (s.pitch * 40) as usize;
            for _ in 0..400 {
                let mut r = |n: u64| next(&mut rng) as i32 % n as i32;
                let clip = Clip {
                    left: r(20),
                    right: 30 + r(34),
                    top: r(10),
                    bottom: 15 + r(25),
                };
                let rect = [r(90) - 20, r(60) - 10, r(80) - 5, r(50) - 5];
                let mut plans = vec![fill(rect, &clip, &s).unwrap()];
                plans.push(hline(rect[0], rect[1], rect[2], &clip, &s));
                plans.push(vline(rect[0], rect[1], rect[3], &clip, &s));
                plans.extend(frame(rect, 1, 2, &clip, &s));
                let pixel: Vec<u8> = (0..bpp as u8).map(|k| 0xA0 + k).collect();
                for plan in plans {
                    let (mut alone, mut whole) = (vec![0x11u8; size], vec![0x11u8; size]);
                    for &offset in &plan.pixels {
                        alone[offset as usize..][..pixel.len()].copy_from_slice(&pixel);
                    }
                    for (offset, count) in runs(&plan.pixels, bpp) {
                        whole[offset as usize..][..pixel.len() * count]
                            .copy_from_slice(&pixel.repeat(count));
                    }
                    assert_eq!(alone, whole, "{bits} bits {rect:?} {clip:?}");
                }
                // The shapes made as runs are the shapes made as pixels.
                let (plan, rows) = fill_runs(rect, &clip, &s).unwrap();
                let whole = fill(rect, &clip, &s).unwrap();
                assert_eq!((plan.work, plan.bits), (whole.work, whole.bits));
                assert_eq!(expand(&rows, &s), whole.pixels);
                let (plan, rows) = hline_runs(rect[0], rect[1], rect[2], &clip, &s);
                let whole = hline(rect[0], rect[1], rect[2], &clip, &s);
                assert_eq!((plan.work, plan.bits), (whole.work, whole.bits));
                assert_eq!(expand(&rows, &s), whole.pixels);
                {}
            }
        }
    }

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
    fn pixels_become_rectangles() {
        let s32 = Surface {
            bits: 32,
            bytes_per_pixel: 4,
            pitch: 64,
        };
        let (rows, pitch) = (8, s32.pitch);
        let covered = |rects: &[[i32; 4]]| {
            let mut set = std::collections::BTreeSet::new();
            for &[x, y, w, h] in rects {
                for j in y..y + h {
                    for i in x..x + w {
                        set.insert((i, j));
                    }
                }
            }
            set
        };
        let clip = Clip {
            left: 0,
            right: 15,
            top: 0,
            bottom: 7,
        };
        let line = hline(2, 3, 5, &clip, &s32);
        let r = pixel_rectangles(&line.pixels, 4, pitch, rows).unwrap();
        assert_eq!(r, [[2, 3, 5, 1]]);
        let column = vline(4, 1, 4, &clip, &s32);
        let r = pixel_rectangles(&column.pixels, 4, pitch, rows).unwrap();
        assert_eq!(r, [[4, 1, 1, 4]]);
        // Every shape's pixels are covered exactly, in the clip's odd cases too.
        for plan in frame([-3, 1, 30, 6], 1, 0, &clip, &s32)
            .into_iter()
            .chain(frame([3, 1, 6, 6], 0, 1, &clip, &s32))
        {
            let r = pixel_rectangles(&plan.pixels, 4, pitch, rows).unwrap();
            let wanted: std::collections::BTreeSet<_> = plan
                .pixels
                .iter()
                .map(|o| ((o % pitch) / 4, o / pitch))
                .collect();
            assert_eq!(covered(&r), wanted);
        }
        // A pixel past the rows, or off the pixel grid, is no rectangle.
        assert!(pixel_rectangles(&[8 * 64], 4, pitch, rows).is_none());
        assert!(pixel_rectangles(&[2], 4, pitch, rows).is_none());
        assert!(pixel_rectangles(&[-4], 4, pitch, rows).is_none());
    }

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
        let p = fill([1, 1, 2, 2], &CLIP, &s32).unwrap();
        assert_eq!((p.bits, p.pixels), (32, vec![268, 272, 532, 536]));
        let s8 = Surface {
            bits: 8,
            bytes_per_pixel: 1,
            pitch: 64,
        };
        let p = fill([1, 1, 2, 2], &CLIP, &s8).unwrap();
        assert_eq!((p.bits, p.pixels), (8, vec![65, 66, 129, 130]));
        let odd = Surface {
            bits: 24,
            bytes_per_pixel: 3,
            pitch: 192,
        };
        assert!(fill([0, 0, 4, 4], &CLIP, &odd).is_err());
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
