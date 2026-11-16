//! The glyph blitter: one run-length coded glyph drawn on a surface, clipped.
//!
//! A glyph table starts with one dword offset per glyph (the first offset / 4 is the
//! count). A normal record is `[x offset][y offset][width][height]` in bytes, then per row
//! runs of `[count][gap][count pixels]`, a count of 0 ending the row; its pixels are
//! palette numbers, stored raw on an 8-bit surface, through the palette on 16 and 32 bits.
//! An extended record starts with a dword >= 0xFFFFFFFC, then the same four as words; its
//! counts, gaps and pixels are words (marker 0xFFFFFFFE) or dwords (any other), copied as
//! they are whatever the surface's depth.

use crate::Invalid;
use crate::gfx::raster::{Clip, Surface, Work};
use crate::strings::bytes::Bytes;

/// The four work variables the blitter leaves. Each is written only once the clipping has got that
/// far.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Scratch {
    pub left_skip: i32,
    pub visible_w: i32,
    pub top_skip: i32,
    /// Counted down to 0 while drawing, unless nothing is clipped horizontally.
    pub rows_left: i32,
}

/// The offset of record `glyph` of a table; `None` for a glyph past the table (the caller raises
/// a fatal error).
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
                let mut out = vec![0; bytes];
                self.record.copy_run(src, &mut out).ok_or(PAST)?;
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

/// `a - b` and whether `a <= b`.
fn sub_le(a: i32, b: i32) -> (i32, bool) {
    (a.wrapping_sub(b), a <= b)
}

/// `a + b` and whether the exact sum is 0 or less.
fn add_le(a: i32, b: i32) -> (i32, bool) {
    (a.wrapping_add(b), a as i64 + b as i64 <= 0)
}

/// Where the walk over a clipped row stands; `bool` carries the last `sub_le` / `add_le` test.
enum Step {
    Skip,
    Next,
    AfterGap(bool),
    AfterCount(bool),
    Rest,
}

/// What the clip makes of one glyph: the work variables the blit leaves and the visible part.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Plan {
    /// The work variables written before the blit stops or draws (the later ones keep their value
    /// when the glyph is clipped away), with `rows` as the clip leaves it.
    pub work: Work,
    /// Whether any pixel of the glyph is inside the clip.
    pub drawn: bool,
    /// The glyph's top-left corner, its own offsets included.
    pub x: i32,
    pub y: i32,
    /// The record's size and the part inside the clip: the columns `left_skip ..
    /// left_skip + visible_w` and the rows `top_skip .. top_skip + rows` of the record.
    pub width: i32,
    pub height: i32,
    pub left_skip: i32,
    pub visible_w: i32,
    pub top_skip: i32,
    pub rows: i32,
}

impl Plan {
    /// The work variables the blit writes, written into `scratch`.
    pub fn write_work(&self, scratch: &mut Scratch) {
        let Work {
            left_skip,
            width,
            top_skip,
            rows,
        } = self.work;
        if let Some(v) = left_skip {
            scratch.left_skip = v;
        }
        if let Some(v) = width {
            scratch.visible_w = v;
        }
        if let Some(v) = top_skip {
            scratch.top_skip = v;
        }
        if let Some(v) = rows {
            scratch.rows_left = v;
        }
    }

    /// The row counter after a drawn glyph: counted down to 0 while rows are clipped
    /// horizontally, untouched when each row is drawn whole.
    pub fn rows_left_after(&self) -> i32 {
        if self.visible_w == self.width {
            self.rows
        } else {
            0
        }
    }
}

/// The clip of a glyph drawn at (`x`, `y`) whose record has `[x offset, y offset, width, height]`.
pub fn plan(x: i32, y: i32, [x_offset, y_offset, width, height]: [i32; 4], clip: &Clip) -> Plan {
    let x = x.wrapping_add(x_offset);
    let y = y.wrapping_add(y_offset);
    let mut work = Work::default();
    // Horizontal clip.
    let mut left_skip = 0;
    if x < clip.left {
        left_skip = clip.left.wrapping_sub(x);
    }
    work.left_skip = Some(left_skip);
    let last = x.wrapping_add(width).wrapping_sub(1);
    let over = last.wrapping_sub(clip.right);
    let mut visible = width;
    if last > clip.right {
        visible = visible.wrapping_sub(over);
    }
    let mut plan = Plan {
        work,
        drawn: false,
        x,
        y,
        width,
        height,
        left_skip,
        visible_w: 0,
        top_skip: 0,
        rows: 0,
    };
    if visible <= left_skip {
        return plan;
    }
    visible = visible.wrapping_sub(left_skip);
    plan.visible_w = visible;
    plan.work.width = Some(visible);

    // Vertical clip.
    let mut top_skip = 0;
    if y < clip.top {
        top_skip = clip.top.wrapping_sub(y);
    }
    plan.top_skip = top_skip;
    plan.work.top_skip = Some(top_skip);
    let last = y.wrapping_add(height).wrapping_sub(1);
    let over = last.wrapping_sub(clip.bottom);
    let mut rows = height;
    if last > clip.bottom {
        rows = rows.wrapping_sub(over);
    }
    if rows <= top_skip {
        return plan;
    }
    rows = rows.wrapping_sub(top_skip);
    plan.rows = rows;
    plan.work.rows = Some(rows);
    plan.drawn = true;
    plan
}

/// Draws `record` at x, y. `put(offset, bytes)` gets every write, at byte offsets from the
/// surface's start (wrapping).
#[allow(clippy::too_many_arguments)] // the record, its place, palette, surface, clip and sink
pub fn blit_glyph(
    record: &(impl Bytes + ?Sized),
    x: i32,
    y: i32,
    palette: &dyn Fn(u8) -> u32,
    surface: &Surface,
    clip: &Clip,
    scratch: &mut Scratch,
    put: &mut dyn FnMut(u32, &[u8]),
) -> Result<(), Invalid> {
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
    let plan = plan(x, y, [x_offset, y_offset, width, height], clip);
    plan.write_work(scratch);
    if !plan.drawn {
        return Ok(());
    }
    let (x, y, visible, rows) = (plan.x, plan.y, plan.visible_w, plan.rows);

    // The glyph's column 0 in the first visible row. The x term uses the surface's bytes
    // per pixel, while the runs advance by the format's own unit.
    let d = format.dst_unit();
    let pitch = surface.pitch;
    let mut row = y
        .wrapping_add(scratch.top_skip)
        .wrapping_mul(pitch)
        .wrapping_add(x.wrapping_mul(surface.bytes_per_pixel)) as u32;
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

    if visible == width {
        // Nothing clipped horizontally: every run is drawn whole.
        let mut rows_left = rows;
        let mut dst = row;
        loop {
            let count = runs.field()?;
            if count == 0 {
                rows_left -= 1;
                if rows_left == 0 {
                    return Ok(());
                }
                row = row.wrapping_add(pitch as u32);
                dst = row;
                continue;
            }
            let gap = runs.field()?;
            dst = dst.wrapping_add(gap.wrapping_mul(d));
            runs.draw(count, dst)?;
            dst = dst.wrapping_add(count.wrapping_mul(d));
        }
    }

    loop {
        // One clipped row. `left`: pixels still to skip while left of the clip edge, then pixels
        // of room inside it.
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
                    let gap = runs.field()?;
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
                    let gap = runs.field()?;
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
                            left = left.wrapping_neg(); // these two paths negate it
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
            return Ok(());
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

    type Drawn = (Vec<u8>, Result<(), Invalid>);

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
        assert!(r.is_ok());
        let (screen, _) = draw(Clip { left: 2, ..all });
        assert_eq!(&screen[32..36], &[0, 0, 2, 3]);
        let (screen, _) = draw(Clip { right: 2, ..all });
        assert_eq!(&screen[32..36], &[0, 1, 2, 0]);
        let (screen, _) = draw(Clip { top: 2, ..all });
        assert_eq!(&screen[16..20], &[0; 4]);
        assert_eq!(&screen[32..36], &[0, 1, 2, 3]);
        let (screen, r) = draw(Clip { left: 9, ..all });
        assert!(r.is_ok());
        assert_eq!(screen, [0; 64]);
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

    /// A small deterministic generator for the random tests.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 33) as u32
        }

        /// A number in `low ..= high`.
        fn between(&mut self, low: i32, high: i32) -> i32 {
            low + (self.next() % (high - low + 1) as u32) as i32
        }
    }

    /// A glyph as its pixels: (column, row, pixel as drawn: an index or a direct colour).
    type Pixels = Vec<(i32, i32, u32)>;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Kind {
        /// Palette indices, one byte each.
        Normal,
        /// Words (marker 0xFFFFFFFE), or dwords (any other marker).
        Words,
        Dwords,
    }

    /// A random record of `kind` with its pixels: rows of runs (a gap, then pixels), the last
    /// rows possibly empty.
    fn random_record(lcg: &mut Lcg, kind: Kind) -> (Vec<u8>, [i32; 4], Pixels) {
        let unit = match kind {
            Kind::Normal => 1,
            Kind::Words => 2,
            Kind::Dwords => 4,
        };
        let put = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes()[..unit]);
        let width = lcg.between(1, 14);
        let height = lcg.between(1, 9);
        let (x_offset, y_offset) = (lcg.between(0, 3), lcg.between(0, 3));
        let mut out = match kind {
            Kind::Normal => vec![x_offset as u8, y_offset as u8, width as u8, height as u8],
            Kind::Words | Kind::Dwords => {
                let marker: u32 = if kind == Kind::Words {
                    0xFFFF_FFFE
                } else {
                    0xFFFF_FFFF
                };
                let mut v = marker.to_le_bytes().to_vec();
                for n in [x_offset, y_offset, width, height] {
                    v.extend_from_slice(&(n as u16).to_le_bytes());
                }
                v
            }
        };
        let mut pixels = Pixels::new();
        for row in 0..height {
            let mut column = 0;
            while column < width && !lcg.next().is_multiple_of(5) {
                let gap = lcg.between(0, 3).min(width - column - 1).max(0);
                let count = lcg.between(1, 6).min(width - column - gap);
                if count <= 0 {
                    break;
                }
                put(&mut out, count as u32);
                put(&mut out, gap as u32);
                column += gap;
                for _ in 0..count {
                    let value = match kind {
                        Kind::Normal => lcg.between(1, 255) as u32,
                        Kind::Words => lcg.between(1, 0xFFFF) as u32,
                        Kind::Dwords => lcg.next() | 1,
                    };
                    put(&mut out, value);
                    pixels.push((column, row, value));
                    column += 1;
                }
            }
            put(&mut out, 0);
        }
        (out, [x_offset, y_offset, width, height], pixels)
    }

    /// Every glyph drawn through the clip equals the pixels the clip leaves, the work variables
    /// are the plan's, and the plan's visible part holds every pixel.
    #[test]
    fn the_plan_and_the_blit_agree_on_random_glyphs() {
        let mut lcg = Lcg(7);
        let palette = |index: u8| 0x0101_0101u32.wrapping_mul(index as u32) ^ 0x0040_2010;
        let (screen_w, screen_h) = (40, 30);
        let mut clipped_rows = 0;
        let mut clipped_columns = 0;
        let mut hidden = 0;
        for _ in 0..30_000 {
            let (kind, bits) = match lcg.next() % 5 {
                0 => (Kind::Normal, 8),
                1 => (Kind::Normal, 16),
                2 => (Kind::Normal, 32),
                3 => (Kind::Words, 16),
                _ => (Kind::Dwords, 32),
            };
            let bpp = bits / 8;
            let surface = Surface {
                bits,
                bytes_per_pixel: bpp as i32,
                pitch: (screen_w * bpp) as i32 + 4 * lcg.between(0, 2),
            };
            let (record, header, pixels) = random_record(&mut lcg, kind);
            let (x, y) = (lcg.between(-18, screen_w as i32 + 4), lcg.between(-12, 34));
            let (left, top) = (lcg.between(-2, 30), lcg.between(-2, 22));
            // Inside the surface, as the game's clip rectangle is.
            let clip = Clip {
                left: left.max(0),
                right: (left + lcg.between(-1, 36)).min(screen_w as i32 - 1),
                top: top.max(0),
                bottom: (top + lcg.between(-1, 28)).min(screen_h as i32 - 1),
            };
            let size = surface.pitch as usize * screen_h;
            let mut canvas = vec![0xAAu8; size + 64];
            let before = canvas.clone();
            let mut scratch = Scratch {
                left_skip: -1,
                visible_w: -2,
                top_skip: -3,
                rows_left: -4,
            };
            let kept = scratch;
            let mut writes_outside = false;
            let drawn = blit_glyph(
                &record[..],
                x,
                y,
                &palette,
                &surface,
                &clip,
                &mut scratch,
                &mut |at, bytes| {
                    let at = at as usize;
                    if at + bytes.len() > canvas.len() {
                        writes_outside = true;
                    } else {
                        canvas[at..at + bytes.len()].copy_from_slice(bytes);
                    }
                },
            );
            assert!(
                drawn.is_ok() && !writes_outside,
                "{kind:?} {header:?} at {x},{y}"
            );
            let plan = plan(x, y, header, &clip);
            // The work variables.
            let mut expected = kept;
            plan.write_work(&mut expected);
            if plan.drawn {
                expected.rows_left = plan.rows_left_after();
            }
            assert_eq!(
                scratch, expected,
                "{kind:?} {header:?} at {x},{y} clip {clip:?}"
            );
            // The plan's own arithmetic, from the clip rectangle's edges.
            let (gx, gy) = (x + header[0], y + header[1]);
            let (first, last) = (gx.max(clip.left), (gx + header[2] - 1).min(clip.right));
            let (top_row, bottom_row) = (gy.max(clip.top), (gy + header[3] - 1).min(clip.bottom));
            assert_eq!(plan.drawn, first <= last && top_row <= bottom_row);
            if plan.drawn {
                assert_eq!((plan.x, plan.y), (gx, gy));
                assert_eq!(plan.left_skip, first - gx);
                assert_eq!(plan.visible_w, last - first + 1);
                assert_eq!(plan.top_skip, top_row - gy);
                assert_eq!(plan.rows, bottom_row - top_row + 1);
                clipped_rows += i32::from(plan.rows < plan.height);
                clipped_columns += i32::from(plan.visible_w < plan.width);
            } else {
                hidden += 1;
            }
            // The pixels the clip leaves, drawn where they belong and nowhere else.
            let mut reference = before.clone();
            for &(column, row, value) in &pixels {
                let (px, py) = (gx + column, gy + row);
                if px < clip.left || px > clip.right || py < clip.top || py > clip.bottom {
                    continue;
                }
                assert!(plan.drawn);
                assert!(
                    (plan.left_skip..plan.left_skip + plan.visible_w).contains(&(px - gx))
                        && (plan.top_skip..plan.top_skip + plan.rows).contains(&(py - gy))
                );
                let at = py as usize * surface.pitch as usize + px as usize * bpp as usize;
                let colour = if kind == Kind::Normal && bits != 8 {
                    palette(value as u8)
                } else {
                    value
                };
                reference[at..at + bpp as usize]
                    .copy_from_slice(&colour.to_le_bytes()[..bpp as usize]);
            }
            assert_eq!(
                canvas, reference,
                "{kind:?} {header:?} at {x},{y} clip {clip:?} bits {bits}"
            );
        }
        // The cases the loop is for were reached.
        assert!(clipped_rows > 1_000 && clipped_columns > 1_000 && hidden > 1_000);
    }
}
