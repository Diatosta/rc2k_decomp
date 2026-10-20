//! Drawing text with the game's bitmap fonts (converted 2026-10-20 from `text::draw_text`
//! 0x4023D5, `text::glyph_kerning` 0x402577, `text::reset_line_state` 0x402563): the
//! layout of lines, their alignment, kerning and the control codes; the pixels are the
//! canvas's.
//!
//! Text bytes from `first_char` to `last_char` are glyphs, bytes above are ignored, bytes
//! below are control codes:
//!   0x00        end of text
//!   0x0A        new line: back to x, down by the line height
//!   0x0E n      x += n (signed byte)
//!   0x0F n      y += n (signed byte); ignored while measuring
//!   0x10.. n    extended glyph (((code - 0x10) << 8 | n) >> 1) + number of normal glyphs
//!   other       x -= code
//! Except in `MODE_AT_X` each line is measured before it is drawn. In `MODE_MEASURE`
//! nothing is drawn and the font is updated instead.
//!
//! A glyph table: a dword per glyph, the offset of its record from the table's start. A
//! record: +0 the x offset (the advance of a glyph without a bitmap), +1 blank rows on top,
//! +2 the bitmap's width (the advance), +3 its rows, then each row as runs (a count of
//! pixels, the gap before them, the pixels; a count of 0 ends the row).

use crate::Invalid;
use crate::bytes::{self, Bytes};

/// The game's 0x20-byte font object. Two fields change meaning with `mode`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Font {
    /// +0x00: the character drawn with glyph 0. Bytes below it are control codes.
    pub first_char: u8,
    /// +0x01: the last character with a glyph. Bytes above it are ignored.
    pub last_char: u8,
    /// +0x02: one of the `MODE_` values.
    pub mode: u8,
    /// +0x03: `FLAG_` bits.
    pub flags: u8,
    /// +0x04: pixels added after every normal glyph.
    pub spacing: i32,
    /// +0x08: pixels from one line to the next.
    pub line_height: i32,
    /// +0x0C: added to y once.
    pub y_offset: i32,
    /// +0x10: pixels added after every extended (two-byte) glyph.
    pub extended_spacing: i32,
    /// +0x14: the palette, for the canvas. In `MODE_MEASURE` a counter instead: one is
    /// added per line.
    pub palette: u32,
    /// +0x18: the glyph table, for the canvas.
    pub glyph_table: u32,
    /// +0x1C: the width of the box lines are aligned in, and the limit for
    /// `FLAG_FAIL_IF_WIDER`. In `MODE_MEASURE` it receives the widest line's width.
    pub box_width: u32,
}

const _: () = assert!(size_of::<Font>() == 0x20);

/// Draw each line from x as given, without measuring it first.
pub const MODE_AT_X: u8 = 0;
/// Measure each line, then draw it from x.
pub const MODE_LEFT: u8 = 1;
/// Measure each line, then draw it so that it ends at x + box width.
pub const MODE_RIGHT: u8 = 2;
/// Measure each line, then centre it in the box. Any value above 4 behaves the same.
pub const MODE_CENTRE: u8 = 3;
/// Measure only: nothing is drawn, the font's line counter and widest line are updated.
pub const MODE_MEASURE: u8 = 4;

/// Stop as soon as a measured line gets wider than the box.
pub const FLAG_FAIL_IF_WIDER: u8 = 2;
/// Pull each glyph left as far as the outline of the line so far allows.
pub const FLAG_KERNING: u8 = 4;

/// Rows of the kerning outline.
pub const KERNING_ROWS: usize = 64;
/// The kerning outline of the line so far: for each row, its trailing blank pixels and its
/// gap (the game's table at 0x93E830).
pub type Kerning = [u8; KERNING_ROWS * 2];

/// Glyph `glyph`'s record: its offset in the table.
pub fn record_at(glyphs: &(impl Bytes + ?Sized), glyph: u32) -> Result<usize, Invalid> {
    let at = (glyph as usize)
        .checked_mul(4)
        .ok_or(Invalid("a glyph past the glyph table"))?;
    glyphs
        .u32_le(at)
        .map(|offset| offset as usize)
        .ok_or(Invalid("a glyph past the glyph table"))
}

fn byte(b: &(impl Bytes + ?Sized), at: usize) -> Result<u8, Invalid> {
    b.byte(at)
        .ok_or(Invalid("a glyph record past the glyph table"))
}

/// Pixels to the next glyph: the bitmap width, or the x offset for a glyph without one.
pub fn advance(glyphs: &(impl Bytes + ?Sized), glyph: u32) -> Result<i32, Invalid> {
    let at = record_at(glyphs, glyph)?;
    let width = byte(glyphs, at + 2)?;
    Ok(if width != 0 { width } else { byte(glyphs, at)? } as i32)
}

fn put(kerning: &mut Kerning, row: usize, trailing: u8, gap: u8) {
    if row < KERNING_ROWS {
        kerning[row * 2] = trailing;
        kerning[row * 2 + 1] = gap;
    }
}

fn trailing_of(kerning: &Kerning, row: usize) -> u8 {
    if row < KERNING_ROWS {
        kerning[row * 2]
    } else {
        0
    }
}

/// The outline updated for one glyph `record`; how far the glyph moves left. Rows past the
/// 64th are left out (the original writes past its table).
pub fn add_record(kerning: &mut Kerning, record: &(impl Bytes + ?Sized)) -> Result<u32, Invalid> {
    let adv = byte(record, 2)?;
    if adv == 0 {
        return Ok(0); // no outline: the table untouched
    }
    let mut row = 0usize;
    for _ in 0..byte(record, 1)? {
        let old = trailing_of(kerning, row);
        put(kerning, row, 0, old.wrapping_add(adv));
        row += 1;
    }
    let mut p = 4usize;
    for _ in 0..byte(record, 3)? {
        let (mut lead_gap, mut trail) = (adv, adv);
        let mut count = byte(record, p)?;
        p += 1;
        if count != 0 {
            lead_gap = byte(record, p)?;
            p += 1;
            trail = adv.wrapping_sub(count).wrapping_sub(lead_gap);
            p += count as usize;
            loop {
                count = byte(record, p)?;
                p += 1;
                if count == 0 {
                    break;
                }
                let gap = byte(record, p)?;
                p += 1;
                trail = trail.wrapping_sub(count).wrapping_sub(gap);
                p += count as usize;
            }
        }
        let old = trailing_of(kerning, row);
        put(kerning, row, trail, lead_gap.wrapping_add(old));
        row += 1;
    }
    let half = adv >> 1;
    while row < KERNING_ROWS {
        let old = trailing_of(kerning, row);
        put(kerning, row, half, old);
        row += 1;
    }
    Ok((0..KERNING_ROWS)
        .map(|r| kerning[r * 2 + 1])
        .min()
        .unwrap_or(0xFF) as u32)
}

/// `add_record` for glyph `glyph` of a table.
pub fn add_glyph(
    kerning: &mut Kerning,
    glyphs: &(impl Bytes + ?Sized),
    glyph: u32,
) -> Result<u32, Invalid> {
    let start = record_at(glyphs, glyph)?;
    add_record(
        kerning,
        &bytes::Offset {
            bytes: glyphs,
            start,
        },
    )
}

/// Where text is drawn.
pub trait Canvas {
    type Error: From<Invalid>;
    /// The kerning outline the canvas keeps.
    fn kerning(&mut self) -> &mut Kerning;
    /// Glyph `glyph` of `font`'s table at x, y.
    fn blit(&mut self, font: &Font, glyph: u32, x: i32, y: i32) -> Result<(), Self::Error>;
}

/// How drawing ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Outcome {
    /// A line was wider than the box and the font has `FLAG_FAIL_IF_WIDER`.
    pub too_wide: bool,
    /// Where the text goes on: after the NUL, or the start of the line that was too wide.
    pub next: usize,
    /// The x given.
    pub x: i32,
    /// The y of the last line, the font's offset included.
    pub y: i32,
}

fn extended_glyph(font: &Font, code: u8, next: u8) -> u32 {
    ((((code - 0x10) as u32) << 8 | next as u32) >> 1)
        + (font.last_char.wrapping_sub(font.first_char) as u32 + 1)
}

/// `text` drawn with `font` (whose glyph table is `glyphs`) from x, y. `font` changes only
/// in `MODE_MEASURE`. On an error, glyphs before it are drawn.
pub fn draw<C: Canvas>(
    font: &mut Font,
    text: &(impl Bytes + ?Sized),
    glyphs: &(impl Bytes + ?Sized),
    x: i32,
    y: i32,
    canvas: &mut C,
) -> Result<Outcome, C::Error> {
    let kerning = font.flags & FLAG_KERNING != 0;
    let at = |i: usize| -> Result<u8, Invalid> {
        text.byte(i).ok_or(Invalid("a text that does not end"))
    };
    let line_x = x;
    let mut place = 0usize;
    let mut y = y.wrapping_add(font.y_offset);
    // The spacing of the glyph before (the original's ebp). It survives from line to line,
    // but an aligned line clears it before it is drawn.
    let mut spacing = 0i32;

    'line: loop {
        let mut x = line_x;
        if kerning {
            *canvas.kerning() = [0; KERNING_ROWS * 2];
        }

        if font.mode != MODE_AT_X {
            let line_start = place;
            let mut width = 0i32;
            let end = loop {
                let c = at(place)?;
                place += 1;
                let glyph = if c < font.first_char {
                    match c {
                        0 | 0x0A => break c,
                        0x0E => {
                            width = width.wrapping_add(at(place)? as i8 as i32);
                            place += 1;
                            continue;
                        }
                        0x0F => {
                            place += 1;
                            continue;
                        }
                        0x10.. => {
                            let next = at(place)?;
                            place += 1;
                            spacing = font.extended_spacing;
                            extended_glyph(font, c, next)
                        }
                        _ => {
                            width = width.wrapping_sub(c as i32);
                            continue;
                        }
                    }
                } else if c > font.last_char {
                    continue;
                } else {
                    spacing = font.spacing;
                    (c - font.first_char) as u32
                };
                if kerning {
                    width = width.wrapping_sub(add_glyph(canvas.kerning(), glyphs, glyph)? as i32);
                }
                width = width
                    .wrapping_add(advance(glyphs, glyph)?)
                    .wrapping_add(spacing);
                if font.flags & FLAG_FAIL_IF_WIDER != 0 && width as u32 > font.box_width {
                    return Ok(Outcome {
                        too_wide: true,
                        next: line_start,
                        x: line_x,
                        y,
                    });
                }
            };
            width = width.wrapping_sub(spacing);

            if font.mode == MODE_MEASURE {
                font.palette = font.palette.wrapping_add(1);
                if font.box_width < width as u32 {
                    font.box_width = width as u32;
                }
                if end == 0 {
                    return Ok(Outcome {
                        too_wide: false,
                        next: place,
                        x: line_x,
                        y,
                    });
                }
                y = y.wrapping_add(font.line_height);
                continue 'line;
            }

            let slack = (font.box_width as i32).wrapping_sub(width);
            match font.mode {
                MODE_LEFT => {}
                MODE_RIGHT => x = x.wrapping_add(slack),
                _ => x = x.wrapping_add(slack >> 1),
            }
            place = line_start;
            spacing = 0;
            if kerning {
                *canvas.kerning() = [0; KERNING_ROWS * 2];
            }
        }

        loop {
            let c = at(place)?;
            place += 1;
            let glyph = if c < font.first_char {
                match c {
                    0 => {
                        return Ok(Outcome {
                            too_wide: false,
                            next: place,
                            x: line_x,
                            y,
                        });
                    }
                    0x0A => {
                        y = y.wrapping_add(font.line_height);
                        continue 'line;
                    }
                    0x0E => {
                        x = x.wrapping_add(at(place)? as i8 as i32);
                        place += 1;
                        continue;
                    }
                    0x0F => {
                        y = y.wrapping_add(at(place)? as i8 as i32);
                        place += 1;
                        continue;
                    }
                    0x10.. => {
                        let next = at(place)?;
                        place += 1;
                        spacing = font.extended_spacing;
                        extended_glyph(font, c, next)
                    }
                    _ => {
                        x = x.wrapping_sub(c as i32);
                        continue;
                    }
                }
            } else if c > font.last_char {
                continue;
            } else {
                spacing = font.spacing;
                (c - font.first_char) as u32
            };
            if kerning {
                x = x.wrapping_sub(add_glyph(canvas.kerning(), glyphs, glyph)? as i32);
            }
            canvas.blit(font, glyph, x, y)?;
            x = x
                .wrapping_add(advance(glyphs, glyph)?)
                .wrapping_add(spacing);
        }
    }
}

/// What `measure` finds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measured {
    pub outcome: Outcome,
    /// The number of lines (the font's +0x14 afterwards).
    pub lines: u32,
    /// The widest line (the font's +0x1C afterwards).
    pub width: u32,
}

/// `text` measured (converted from `text::measure_text` 0x40230B): `draw` from x and the
/// font's line height with the line count and widest line cleared first; the font is left
/// with them, as the original leaves it.
pub fn measure<C: Canvas>(
    font: &mut Font,
    text: &(impl Bytes + ?Sized),
    glyphs: &(impl Bytes + ?Sized),
    x: i32,
    canvas: &mut C,
) -> Result<Measured, C::Error> {
    font.box_width = 0;
    font.palette = 0;
    let outcome = draw(font, text, glyphs, x, font.line_height, canvas)?;
    Ok(Measured {
        outcome,
        lines: font.palette,
        width: font.box_width,
    })
}

/// A race time's eight characters ("01:23.45" and one more) in fixed columns: the step
/// from each column to the next (digit to digit 13, to or from punctuation 10; from
/// `text::draw_time_fixed_pitch` 0x465EC4).
pub const TIME_STEPS: [i32; 8] = [13, 10, 10, 13, 10, 10, 13, 0];
/// A time of day's five ("09:05"; `text::draw_clock_fixed_pitch` 0x465F42). The last step
/// is added after the fifth character too.
pub const CLOCK_STEPS: [i32; 5] = [13, 10, 10, 13, 10];

/// The x of each fixed column for text ending at `right`: the first at `right` - 100,
/// then on by `steps`.
pub fn columns(right: i32, steps: &[i32]) -> Vec<i32> {
    let mut x = right.wrapping_sub(100);
    steps
        .iter()
        .map(|step| {
            let at = x;
            x = x.wrapping_add(*step);
            at
        })
        .collect()
}

/// Where a highlighted menu item's pulse stands (converted from `text::blend_font`
/// 0x405E92): whether it goes from font A to font B now (else back), and how far, 0 ..
/// `half_pulse` - 1. `eased` (blend mode 2) makes it t * t >> 8 and the same from the other
/// end.
pub fn pulse(time: u32, half_pulse: u32, eased: bool) -> (bool, u32) {
    let mut t = time & half_pulse.wrapping_sub(1);
    if eased {
        t = t.wrapping_mul(t) >> 8;
        t ^= 0xFF;
        t = t.wrapping_mul(t) >> 8;
        t ^= 0xFF;
    }
    (half_pulse & time != 0, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Glyph records without bitmaps rows: the advance is the width byte.
    fn table(widths: &[u8]) -> Vec<u8> {
        let base = widths.len() * 4;
        let mut out = Vec::new();
        for i in 0..widths.len() {
            out.extend_from_slice(&((base + i * 4) as u32).to_le_bytes());
        }
        for &w in widths {
            out.extend_from_slice(&[0, 0, w, 0]);
        }
        out
    }

    struct Recording {
        kerning: Kerning,
        blits: Vec<(u32, i32, i32)>,
    }

    impl Recording {
        fn new() -> Self {
            Recording {
                kerning: [0; KERNING_ROWS * 2],
                blits: Vec::new(),
            }
        }
    }

    impl Canvas for Recording {
        type Error = Invalid;
        fn kerning(&mut self) -> &mut Kerning {
            &mut self.kerning
        }
        fn blit(&mut self, _: &Font, glyph: u32, x: i32, y: i32) -> Result<(), Invalid> {
            self.blits.push((glyph, x, y));
            Ok(())
        }
    }

    fn font(mode: u8) -> Font {
        Font {
            first_char: b'A',
            last_char: b'C',
            mode,
            spacing: 1,
            line_height: 10,
            y_offset: 2,
            box_width: 40,
            ..Font::default()
        }
    }

    #[test]
    fn lines_at_x() {
        let glyphs = table(&[5, 6, 7]);
        let mut f = font(MODE_AT_X);
        let mut c = Recording::new();
        let out = draw(&mut f, b"AB\nC\0", &glyphs, 100, 50, &mut c).unwrap();
        assert_eq!(c.blits, [(0, 100, 52), (1, 106, 52), (2, 100, 62)]);
        assert_eq!(
            out,
            Outcome {
                too_wide: false,
                next: 5,
                x: 100,
                y: 62
            }
        );
    }

    #[test]
    fn aligned_and_controls() {
        let glyphs = table(&[5, 6, 7]);
        // "AB" is 5 + 1 + 6 = 12 wide (the last spacing taken off): centred in 40, x + 14.
        let mut f = font(MODE_CENTRE);
        let mut c = Recording::new();
        draw(&mut f, b"AB\0", &glyphs, 0, 0, &mut c).unwrap();
        assert_eq!(c.blits, [(0, 14, 2), (1, 20, 2)]);
        // Right: x + 28. A 0x0E moves x on, a 0x0F moves y, other codes move x back.
        let mut f = font(MODE_RIGHT);
        let mut c = Recording::new();
        draw(&mut f, b"A\x0E\x04\x0F\x03B\x02C\0", &glyphs, 0, 0, &mut c).unwrap();
        // width: 5+1 +4 +6+1 -2 +7+1, less 1 = 22; x = 18
        assert_eq!(c.blits, [(0, 18, 2), (1, 28, 5), (2, 33, 5)]);
    }

    #[test]
    fn measuring_and_too_wide() {
        let glyphs = table(&[5, 6, 7]);
        let mut f = font(MODE_MEASURE);
        let mut c = Recording::new();
        f.box_width = 0;
        let out = draw(&mut f, b"ABC\nA\0", &glyphs, 0, 0, &mut c).unwrap();
        assert!(c.blits.is_empty());
        assert_eq!((f.palette, f.box_width), (2, 20));
        assert_eq!(out.next, 6);
        let mut f = font(MODE_LEFT);
        f.flags = FLAG_FAIL_IF_WIDER;
        f.box_width = 10;
        let out = draw(&mut f, b"A\nABC\0", &glyphs, 0, 0, &mut c).unwrap();
        assert!(out.too_wide);
        assert_eq!(out.next, 2);
    }

    #[test]
    fn measures_and_columns() {
        let glyphs = table(&[5, 6, 7]);
        let mut f = font(MODE_MEASURE);
        f.box_width = 99;
        f.palette = 7;
        let mut c = Recording::new();
        let m = measure(&mut f, b"AB\nABC\0", &glyphs, 0, &mut c).unwrap();
        assert_eq!((m.lines, m.width), (2, 20));
        assert_eq!(m.outcome.y, 10 + 2 + 10);
        assert_eq!(columns(200, &CLOCK_STEPS), [100, 113, 123, 133, 146]);
        assert_eq!(columns(200, &TIME_STEPS)[7], 179);
        assert_eq!(pulse(0x180, 0x100, false), (true, 0x80));
        assert_eq!(pulse(0x80, 0x100, false), (false, 0x80));
        assert_eq!(pulse(0x80, 0x100, true).1, 0x71); // 0x80 -> 0x40 -> 0xBF -> 0x8E -> 0x71
    }

    #[test]
    fn bad_input_is_refused() {
        let glyphs = table(&[5]);
        let mut f = font(MODE_AT_X);
        let mut c = Recording::new();
        assert!(draw(&mut f, b"AB", &glyphs, 0, 0, &mut c).is_err()); // no NUL
        assert!(draw(&mut f, b"B\0", &glyphs, 0, 0, &mut c).is_err()); // no glyph 1
    }

    #[test]
    fn kerning_outline() {
        // A glyph 4 wide with one row: 1 gap pixel, 2 pixels.
        let record = [4u8, 0, 4, 1, 2, 1, 9, 9, 0];
        let mut k: Kerning = [0; KERNING_ROWS * 2];
        let moved = add_record(&mut k, &record).unwrap();
        // Row 0: trailing 4 - 2 - 1 = 1, gap 1; the rest: trailing 2, gap 0.
        assert_eq!(&k[..4], &[1, 1, 2, 0]);
        assert_eq!(moved, 0);
        assert_eq!(add_record(&mut k, &[0u8, 0, 0, 0]).unwrap(), 0);
        assert!(add_record(&mut k, &[4u8, 0, 4, 1, 2]).is_err());
    }
}
