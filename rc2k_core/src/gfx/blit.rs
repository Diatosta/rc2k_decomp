//! The glyph blitter (converted 2026-10-20 from `gfx::blit_glyph` 0x4069B2): one
//! run-length coded glyph drawn on a surface, clipped.
//!
//! A glyph table starts with one dword offset per glyph (the first offset / 4 is the
//! count). A normal record is `[x offset][y offset][width][height]` in bytes, then per row
//! runs of `[count][gap][count pixels]`, a count of 0 ending the row; its pixels are
//! palette numbers, stored raw on an 8-bit surface, through the palette on 16 and 32 bits.
//! An extended record starts with a dword >= 0xFFFFFFFC, then the same four as words; its
//! counts, gaps and pixels are words (marker 0xFFFFFFFE) or dwords (any other), copied as
//! they are whatever the surface's depth.

use crate::Invalid;
use crate::gfx::raster::{Clip, Surface};
use crate::strings::bytes::Bytes;

/// The four work variables the blitter leaves (`g_blitLeftSkip` ..). Each is written only
/// once the clipping has got that far.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Scratch {
    pub left_skip: i32,
    pub visible_w: i32,
    pub top_skip: i32,
    /// Counted down to 0 while drawing, unless nothing is clipped horizontally.
    pub rows_left: i32,
}

/// What the original leaves in ebx: a value, or a row's place (a byte offset from the
/// surface's start, to which the caller adds the surface's address).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ebx {
    Value(u32),
    Row(u32),
}

/// The four registers the original does not preserve, as it leaves them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Exit {
    pub eax: u32,
    pub ebx: Ebx,
    pub ecx: u32,
    pub edx: u32,
}

/// The part of the surface a glyph may touch: `rows` spans of `row_bytes` bytes, the first
/// at byte offset `first`, each `pitch` bytes after the one before.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    pub first: u32,
    pub pitch: i32,
    pub rows: u32,
    pub row_bytes: u32,
}

/// The offset of record `glyph` of a table; `None` for a glyph past the table (the
/// original's fatal error 0x80A7DF32).
pub fn record_offset(table: &(impl Bytes + ?Sized), glyph: u32) -> Result<Option<usize>, Invalid> {
    let first = table.u32_le(0).ok_or(PAST)?;
    if glyph >= first >> 2 {
        return Ok(None);
    }
    let offset = table.u32_le(glyph as usize * 4).ok_or(PAST)?;
    Ok(Some(offset as usize))
}

const PAST: Invalid = Invalid("a glyph record past its table");

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Raw8,
    Palette16,
    Palette32,
    Direct16,
    Direct32,
}

impl Format {
    fn src_unit(self) -> usize {
        match self {
            Format::Raw8 | Format::Palette16 | Format::Palette32 => 1,
            Format::Direct16 => 2,
            Format::Direct32 => 4,
        }
    }
    fn dst_unit(self) -> u32 {
        match self {
            Format::Raw8 => 1,
            Format::Palette16 | Format::Direct16 => 2,
            Format::Palette32 | Format::Direct32 => 4,
        }
    }
    fn is_direct(self) -> bool {
        matches!(self, Format::Direct16 | Format::Direct32)
    }
}

/// Reader over a record's run data; `src` an offset into the record, wrapping.
struct Runs<'a, B: Bytes + ?Sized> {
    format: Format,
    record: &'a B,
    src: usize,
    palette: &'a dyn Fn(u8) -> u32,
    put: &'a mut dyn FnMut(u32, &[u8]),
    /// The colours of the run being drawn; kept for the next run's.
    colours: Vec<u8>,
}

impl<B: Bytes + ?Sized> Runs<'_, B> {
    fn byte(&self, at: usize) -> Result<u8, Invalid> {
        self.record.byte(at).ok_or(PAST)
    }

    fn field(&mut self) -> Result<u32, Invalid> {
        let unit = self.format.src_unit();
        let mut v = 0u32;
        for k in (0..unit).rev() {
            v = v << 8 | self.byte(self.src.wrapping_add(k))? as u32;
        }
        self.src = self.src.wrapping_add(unit);
        Ok(v)
    }

    fn skip_pixels(&mut self, pixels: i32) {
        let bytes = (pixels as isize).wrapping_mul(self.format.src_unit() as isize);
        self.src = self.src.wrapping_add_signed(bytes);
    }

    fn draw(&mut self, count: u32, dst: u32) -> Result<(), Invalid> {
        if count == 0 {
            return Ok(());
        }
        let n = count as usize;
        match self.format {
            Format::Raw8 | Format::Direct16 | Format::Direct32 => {
                let bytes = n * self.format.src_unit();
                let src = self.src;
                self.src = self.src.wrapping_add(bytes);
                if let Some(run) = self.record.run(src, bytes) {
                    (self.put)(dst, run);
                    return Ok(());
                }
                let out = (0..bytes)
                    .map(|i| self.byte(src.wrapping_add(i)))
                    .collect::<Result<Vec<u8>, _>>()?;
                (self.put)(dst, &out);
            }
            Format::Palette16 | Format::Palette32 => {
                let d = self.format.dst_unit() as usize;
                let mut out = std::mem::take(&mut self.colours);
                out.clear();
                out.resize(n * d, 0);
                if let Some(run) = self.record.run(self.src, n) {
                    for (place, &index) in out.chunks_exact_mut(d).zip(run) {
                        place.copy_from_slice(&(self.palette)(index).to_le_bytes()[..d]);
                    }
                } else {
                    for i in 0..n {
                        let colour = (self.palette)(self.byte(self.src.wrapping_add(i))?);
                        out[i * d..(i + 1) * d].copy_from_slice(&colour.to_le_bytes()[..d]);
                    }
                }
                self.src = self.src.wrapping_add(n);
                (self.put)(dst, &out);
                self.colours = out;
            }
        }
        Ok(())
    }
}

/// `a - b` and whether a `jle` after that `sub` is taken.
fn sub_le(a: i32, b: i32) -> (i32, bool) {
    (a.wrapping_sub(b), a <= b)
}

/// `a + b` and whether a `jle` after that `add` is taken.
fn add_le(a: i32, b: i32) -> (i32, bool) {
    (a.wrapping_add(b), a as i64 + b as i64 <= 0)
}

/// Where the walk over a clipped row stands; `bool` carries the original's `jle`.
enum Step {
    Skip,
    Next,
    AfterGap(bool),
    AfterCount(bool),
    Rest,
}

/// Draws `record` at x, y. `put(offset, bytes)` gets every write, at byte offsets from the
/// surface's start (wrapping). Returns the registers the original leaves and, unless the
/// glyph is clipped away, the part of the surface it may have touched.
#[allow(clippy::too_many_arguments)] // the original's inputs, the surface, the clip and the sink
pub fn blit_glyph(
    record: &(impl Bytes + ?Sized),
    x: i32,
    y: i32,
    palette: &dyn Fn(u8) -> u32,
    surface: &Surface,
    clip: &Clip,
    scratch: &mut Scratch,
    put: &mut dyn FnMut(u32, &[u8]),
) -> Result<(Exit, Option<Area>), Invalid> {
    let marker = record.u32_le(0).ok_or(PAST)?;
    let byte = |at: usize| record.byte(at).map(i32::from).ok_or(PAST);
    let word = |at: usize| Ok::<_, Invalid>(byte(at)? | byte(at + 1)? << 8);
    let (x_offset, y_offset, width, height, data, format) = if !direct_colour(marker) {
        let format = match surface.bits {
            16 => Format::Palette16,
            32 => Format::Palette32,
            _ => Format::Raw8,
        };
        (byte(0)?, byte(1)?, byte(2)?, byte(3)?, 4, format)
    } else {
        let format = if marker == 0xFFFF_FFFE {
            Format::Direct16
        } else {
            Format::Direct32
        };
        (word(4)?, word(6)?, word(8)?, word(10)?, 12, format)
    };
    let x = x.wrapping_add(x_offset);
    let y = y.wrapping_add(y_offset);
    let clipped_away = |eax: i32, ebx: i32| {
        let exit = Exit {
            eax: eax as u32,
            ebx: Ebx::Value(ebx as u32),
            ecx: x as u32,
            edx: y as u32,
        };
        Ok((exit, None))
    };

    // Horizontal clip.
    scratch.left_skip = 0;
    if x < clip.left {
        scratch.left_skip = clip.left.wrapping_sub(x);
    }
    let last = x.wrapping_add(width).wrapping_sub(1);
    let over = last.wrapping_sub(clip.right);
    let mut visible = width;
    if last > clip.right {
        visible = visible.wrapping_sub(over);
    }
    if visible <= scratch.left_skip {
        return clipped_away(visible.wrapping_sub(scratch.left_skip), over);
    }
    visible = visible.wrapping_sub(scratch.left_skip);
    scratch.visible_w = visible;

    // Vertical clip.
    scratch.top_skip = 0;
    if y < clip.top {
        scratch.top_skip = clip.top.wrapping_sub(y);
    }
    let last = y.wrapping_add(height).wrapping_sub(1);
    let over = last.wrapping_sub(clip.bottom);
    let mut rows = height;
    if last > clip.bottom {
        rows = rows.wrapping_sub(over);
    }
    if rows <= scratch.top_skip {
        return clipped_away(rows.wrapping_sub(scratch.top_skip), over);
    }
    rows = rows.wrapping_sub(scratch.top_skip);
    scratch.rows_left = rows;

    // The glyph's column 0 in the first visible row. The x term uses the surface's bytes
    // per pixel, while the runs advance by the format's own unit.
    let d = format.dst_unit();
    let pitch = surface.pitch;
    let mut row = y
        .wrapping_add(scratch.top_skip)
        .wrapping_mul(pitch)
        .wrapping_add(x.wrapping_mul(surface.bytes_per_pixel)) as u32;
    let area = Area {
        first: row.wrapping_add((scratch.left_skip as u32).wrapping_mul(d)),
        pitch,
        rows: rows as u32,
        row_bytes: (visible as u32).wrapping_mul(d),
    };
    let done = |eax: u32, row: u32, edx: u32| {
        let exit = Exit {
            eax,
            ebx: Ebx::Row(row),
            ecx: 0,
            edx,
        };
        Ok((exit, Some(area)))
    };

    let mut runs = Runs {
        format,
        record,
        src: data,
        palette,
        put,
        colours: Vec::new(),
    };

    // Step over the rows clipped off at the top.
    let mut skip = scratch.top_skip;
    if skip != 0 {
        loop {
            let count = runs.field()?;
            if count == 0 {
                skip -= 1;
                if skip == 0 {
                    break;
                }
            } else {
                runs.field()?;
                runs.skip_pixels(count as i32);
            }
        }
    }

    // The gap of the last run read: what the original leaves in eax.
    let mut gap = 0u32;

    if visible == width {
        // Nothing clipped horizontally: every run is drawn whole.
        let mut rows_left = rows;
        let mut dst = row;
        loop {
            let count = runs.field()?;
            if count == 0 {
                rows_left -= 1;
                if rows_left == 0 {
                    return done(gap, row, 0);
                }
                row = row.wrapping_add(pitch as u32);
                dst = row;
                continue;
            }
            gap = runs.field()?;
            dst = dst.wrapping_add(gap.wrapping_mul(d));
            runs.draw(count, dst)?;
            dst = dst.wrapping_add(count.wrapping_mul(d));
        }
    }

    loop {
        // One clipped row. `left` is the original's edx: pixels still to skip while left of
        // the clip edge, then pixels of room inside it.
        let mut dst = row;
        let mut left = scratch.left_skip;
        let mut count = 0u32;
        let mut step = if left != 0 {
            Step::Skip
        } else {
            left = scratch.visible_w;
            Step::Next
        };
        loop {
            match step {
                Step::Skip => {
                    count = runs.field()?;
                    if count == 0 {
                        break;
                    }
                    gap = runs.field()?;
                    dst = dst.wrapping_add(gap.wrapping_mul(d));
                    let (rest, le) = sub_le(left, gap as i32);
                    left = rest;
                    if le {
                        // The gap reached the clip edge.
                        let (room, le) = add_le(left, scratch.visible_w);
                        left = room;
                        step = Step::AfterGap(le);
                        continue;
                    }
                    runs.skip_pixels(count as i32);
                    dst = dst.wrapping_add(count.wrapping_mul(d));
                    let (rest, le) = sub_le(left, count as i32);
                    left = rest;
                    if !le {
                        continue; // the whole run is left of the clip edge
                    }
                    // The run crosses the clip edge: back to its first visible pixel.
                    count = left.wrapping_neg() as u32;
                    runs.skip_pixels(left);
                    dst = dst.wrapping_add((left as u32).wrapping_mul(d));
                    let (room, le) = add_le(left, scratch.visible_w);
                    left = room;
                    step = Step::AfterCount(le);
                }
                Step::Next => {
                    count = runs.field()?;
                    if count == 0 {
                        break;
                    }
                    gap = runs.field()?;
                    dst = dst.wrapping_add(gap.wrapping_mul(d));
                    let (room, le) = sub_le(left, gap as i32);
                    left = room;
                    step = Step::AfterGap(le);
                }
                Step::AfterGap(no_room) => {
                    if no_room {
                        runs.skip_pixels(count as i32);
                        step = Step::Rest;
                    } else {
                        let (room, le) = sub_le(left, count as i32);
                        left = room;
                        step = Step::AfterCount(le);
                    }
                }
                Step::AfterCount(last_run) => {
                    if last_run {
                        // Only the part up to the right clip edge.
                        count = count.wrapping_add(left as u32);
                        runs.draw(count, dst)?;
                        runs.skip_pixels(left.wrapping_neg());
                        if format.is_direct() {
                            left = left.wrapping_neg(); // these two paths `neg edx`
                        }
                        step = Step::Rest;
                    } else {
                        runs.draw(count, dst)?;
                        dst = dst.wrapping_add(count.wrapping_mul(d));
                        step = Step::Next;
                    }
                }
                Step::Rest => {
                    count = runs.field()?;
                    if count == 0 {
                        break;
                    }
                    runs.field()?;
                    runs.skip_pixels(count as i32);
                }
            }
        }
        scratch.rows_left = scratch.rows_left.wrapping_sub(1);
        if scratch.rows_left == 0 {
            return done(gap, row, left as u32);
        }
        row = row.wrapping_add(pitch as u32);
    }
}

/// Whether a record starting with `marker` holds its pixels as they are (an extended
/// record), not as palette indices.
pub fn direct_colour(marker: u32) -> bool {
    marker >= 0xFFFF_FFFC
}

#[cfg(test)]
mod tests {
    use super::*;

    const S8: Surface = Surface {
        bits: 8,
        bytes_per_pixel: 1,
        pitch: 16,
    };

    /// A 3 x 2 glyph: row 1 a gap of 1 and pixels 7, 8; row 2 pixels 1, 2, 3.
    const GLYPH: [u8; 15] = [0, 0, 3, 2, 2, 1, 7, 8, 0, 3, 0, 1, 2, 3, 0];

    type Drawn = (Vec<u8>, Result<(Exit, Option<Area>), Invalid>);

    fn draw(clip: Clip) -> Drawn {
        let mut screen = vec![0u8; 16 * 4];
        let mut scratch = Scratch::default();
        let r = blit_glyph(
            &GLYPH,
            1,
            1,
            &|p| p as u32,
            &S8,
            &clip,
            &mut scratch,
            &mut |at, b| screen[at as usize..][..b.len()].copy_from_slice(b),
        );
        (screen, r)
    }

    #[test]
    fn whole_and_clipped() {
        let all = Clip {
            left: 0,
            right: 15,
            top: 0,
            bottom: 3,
        };
        let (screen, r) = draw(all);
        assert_eq!(&screen[16..20], &[0, 0, 7, 8]);
        assert_eq!(&screen[32..36], &[0, 1, 2, 3]);
        assert_eq!(r.unwrap().0.ebx, Ebx::Row(33));
        let (screen, _) = draw(Clip { left: 2, ..all });
        assert_eq!(&screen[32..36], &[0, 0, 2, 3]);
        let (screen, _) = draw(Clip { right: 2, ..all });
        assert_eq!(&screen[32..36], &[0, 1, 2, 0]);
        let (screen, _) = draw(Clip { top: 2, ..all });
        assert_eq!(&screen[16..20], &[0; 4]);
        assert_eq!(&screen[32..36], &[0, 1, 2, 3]);
        let (_, r) = draw(Clip { left: 9, ..all });
        assert!(r.unwrap().1.is_none());
        // A record cut short is refused, not read past.
        let mut scratch = Scratch::default();
        assert!(
            blit_glyph(
                &GLYPH[..9],
                0,
                0,
                &|p| p as u32,
                &S8,
                &all,
                &mut scratch,
                &mut |_, _| ()
            )
            .is_err()
        );
    }

    #[test]
    fn tables() {
        let table = [8u8, 0, 0, 0, 9, 0, 0, 0];
        assert_eq!(record_offset(&table, 1), Ok(Some(9)));
        assert_eq!(record_offset(&table, 2), Ok(None));
    }
}
