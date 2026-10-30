//! The debug font (IDA 2026-10-30): `sub_420313` (a text), `sub_4205F1` (a character's
//! glyph), `sub_420791` / `sub_4208DB` (a glyph at 16 / 32 bits), `sub_420A6A` (the step to
//! the next character).
//!
//! 40 glyphs of 4 x 5 pixels (`unk_5C6614`: five rows of 40 words, a pixel a nibble from the
//! top, lit when not 0; the nibbles are written as the digits 0 and 1), each pixel a square
//! of `scale` pixels (`g_debugScale`). A character is 4 `scale` wide; past the screen's width
//! the text goes on 6 `scale` lower at the width less. A print at (0, 0) goes on where the last
//! one ended (`g_debugCursorX` / `Y`).

/// The glyphs (`unk_5C6614`), row by row: the digits, A .. Z, then '-', '.', '+' and a
/// blank.
pub const GLYPHS: [[u16; GLYPH_COUNT]; ROWS] = [
    [
        0x0100, 0x0100, 0x1100, 0x1100, 0x1010, 0x1110, 0x0100, 0x1110, 0x0100, 0x0100, 0x0100,
        0x1100, 0x0100, 0x1100, 0x1110, 0x1110, 0x1110, 0x1010, 0x1110, 0x1110, 0x1010, 0x1000,
        0x1010, 0x1100, 0x1110, 0x1100, 0x1110, 0x1100, 0x0110, 0x1110, 0x1010, 0x1010, 0x1010,
        0x1010, 0x1010, 0x1110, 0x0000, 0x0000, 0x0000, 0x0000,
    ],
    [
        0x1010, 0x1100, 0x0010, 0x0010, 0x1010, 0x1000, 0x1000, 0x0010, 0x1010, 0x1010, 0x1010,
        0x1010, 0x1010, 0x1010, 0x1000, 0x1000, 0x1000, 0x1010, 0x0100, 0x0010, 0x1010, 0x1000,
        0x1110, 0x1010, 0x1010, 0x1010, 0x1010, 0x1010, 0x1000, 0x0100, 0x1010, 0x1010, 0x1010,
        0x1010, 0x1010, 0x0010, 0x0000, 0x0000, 0x0100, 0x0000,
    ],
    [
        0x1010, 0x0100, 0x0100, 0x1100, 0x1110, 0x1100, 0x1100, 0x0100, 0x0100, 0x0110, 0x1110,
        0x1100, 0x1000, 0x1010, 0x1100, 0x1100, 0x1010, 0x1110, 0x0100, 0x0010, 0x1100, 0x1000,
        0x1010, 0x1010, 0x1010, 0x1100, 0x1010, 0x1100, 0x0100, 0x0100, 0x1010, 0x1010, 0x1010,
        0x0100, 0x1110, 0x0100, 0x1110, 0x0000, 0x1110, 0x0000,
    ],
    [
        0x1010, 0x0100, 0x1000, 0x0010, 0x0010, 0x0010, 0x1010, 0x0100, 0x1010, 0x0010, 0x1010,
        0x1010, 0x1010, 0x1010, 0x1000, 0x1000, 0x1010, 0x1010, 0x0100, 0x1010, 0x1010, 0x1000,
        0x1010, 0x1010, 0x1010, 0x1000, 0x1100, 0x1100, 0x0010, 0x0100, 0x1010, 0x0100, 0x1110,
        0x1010, 0x0100, 0x1000, 0x0000, 0x0110, 0x0100, 0x0000,
    ],
    [
        0x0100, 0x1110, 0x1110, 0x1100, 0x0010, 0x1100, 0x0100, 0x0100, 0x0100, 0x1100, 0x1010,
        0x1100, 0x0100, 0x1100, 0x1110, 0x1000, 0x1110, 0x1010, 0x1110, 0x0100, 0x1010, 0x1110,
        0x1010, 0x1010, 0x1110, 0x1000, 0x1010, 0x1010, 0x1100, 0x0100, 0x1110, 0x0100, 0x1010,
        0x1010, 0x0100, 0x1110, 0x0000, 0x0110, 0x0000, 0x0000,
    ],
];

/// Glyphs in the font.
pub const GLYPH_COUNT: usize = 40;
/// A glyph's rows and columns.
pub const ROWS: usize = 5;
pub const COLUMNS: usize = 4;

/// The glyph of '-'.
pub const MINUS: u8 = 0x24;
/// Of '.'.
pub const DOT: u8 = 0x25;
/// Of '+'.
pub const PLUS: u8 = 0x26;
/// The blank one.
pub const BLANK: u8 = 0x27;

/// `sub_4205F1`: a character's glyph. A digit, a letter of either case, '-', '+', '.'; the
/// blank for a space and below '0', between '9' and 'A', 'Z' and 'a'; past 'z' the minus
/// (the original's fall-through into the '-' case).
pub fn glyph_of(c: u8) -> u8 {
    match c {
        b'-' => MINUS,
        b'+' => PLUS,
        b'.' => DOT,
        b'0'..=b'9' => c - b'0',
        b'A'..=b'Z' => c - 0x37,
        b'a'..=b'z' => c - 0x57,
        0x7B.. => MINUS,
        _ => BLANK,
    }
}

/// Whether the glyph's pixel at (`column`, `row`) is lit.
pub fn lit(glyph: u8, column: usize, row: usize) -> bool {
    let word = GLYPHS[row][usize::from(glyph) % GLYPH_COUNT];
    (word >> (12 - 4 * column)) & 0xF != 0
}

/// `sub_420A6A`: the place of the next character: 4 `scale` to the right; at or past the
/// screen's `width` the width less, 6 `scale` lower.
pub fn advance(x: u32, y: u32, scale: u32, width: u32) -> (u32, u32) {
    let x = x.wrapping_add(scale.wrapping_mul(4));
    if x < width {
        (x, y)
    } else {
        (x.wrapping_sub(width), y.wrapping_add(scale.wrapping_mul(6)))
    }
}

/// The frame a glyph goes into: its row pitch, bytes a pixel (2 or 4) and length (the pitch
/// times the screen's height).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub pitch: u32,
    pub wide: u32,
    pub len: u32,
}

/// A run of pixels to fill: its offset in the frame and its pixels.
pub type Run = (u32, u32);

/// `sub_420791` / `sub_4208DB`: the runs of a glyph at (`x`, `y`) (their low 16 bits), each
/// lit pixel `scale` rows of `scale` pixels. A row is filled when it starts before the
/// frame's end; the original then fills all of it, running up to `scale - 1` pixels past the
/// end: here a run ends at the frame's end.
pub fn glyph_runs(glyph: u8, x: u32, y: u32, scale: u32, frame: Frame, runs: &mut Vec<Run>) {
    let at = (y & 0xFFFF)
        .wrapping_mul(frame.pitch)
        .wrapping_add((x & 0xFFFF).wrapping_mul(frame.wide));
    let step = scale.wrapping_mul(frame.wide);
    for row in 0..ROWS {
        let row_at = at.wrapping_add((row as u32).wrapping_mul(scale).wrapping_mul(frame.pitch));
        for column in 0..COLUMNS {
            if !lit(glyph, column, row) {
                continue;
            }
            let mut start = row_at.wrapping_add((column as u32).wrapping_mul(step));
            for _ in 0..scale {
                if start < frame.len {
                    let pixels = scale.min((frame.len - start) / frame.wide);
                    runs.push((start, pixels));
                }
                start = start.wrapping_add(frame.pitch);
            }
        }
    }
}

/// `sub_420313`: the runs of `text` (to its NUL or end) from (`x`, `y`), or from `cursor`
/// when both are 0; where the next print would go on (the new cursor).
pub fn print_runs(
    text: &[u8],
    (x, y): (u32, u32),
    cursor: (u32, u32),
    scale: u32,
    width: u32,
    frame: Frame,
    runs: &mut Vec<Run>,
) -> (u32, u32) {
    let (mut x, mut y) = if x == 0 && y == 0 { cursor } else { (x, y) };
    for &c in text.iter().take_while(|&&c| c != 0) {
        glyph_runs(glyph_of(c), x, y, scale, frame, runs);
        (x, y) = advance(x, y, scale, width);
    }
    (x, y)
}

/// `sub_4202AB`'s glyph of a character: '=' the dot, '+', '-', a digit, a lower-case letter;
/// anything else the blank (upper case too).
pub fn narrow_glyph_of(c: u8) -> u8 {
    match c {
        b'=' => DOT,
        b'+' => PLUS,
        b'-' => MINUS,
        b'0'..=b'9' => c - b'0',
        b'a'..=b'z' => c - b'a' + 10,
        _ => BLANK,
    }
}

/// `sub_4202AB`: the runs of at most `count` characters of `text` (to its NUL) from
/// (`x`, `y`) at `scale`, each glyph `sub_420ABB`'s; a character 4 `scale` wide, a blank 2
/// (the original's scale is 1: it steps 2 and 2 more); no wrapping.
pub fn narrow_runs(
    text: &[u8],
    count: usize,
    (mut x, y): (u32, u32),
    scale: u32,
    frame: Frame,
    runs: &mut Vec<Run>,
) {
    for &c in text.iter().take(count).take_while(|&&c| c != 0) {
        let glyph = narrow_glyph_of(c);
        glyph_runs(glyph, x, y, scale, frame, runs);
        let step = if glyph == BLANK { 2 } else { 4 };
        x = x.wrapping_add(step * scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn characters_have_their_glyphs() {
        assert_eq!(glyph_of(b'7'), 7);
        assert_eq!(glyph_of(b'A'), 10);
        assert_eq!(glyph_of(b'z'), 35);
        assert_eq!(glyph_of(b'-'), MINUS);
        assert_eq!(glyph_of(b' '), BLANK);
        assert_eq!(glyph_of(b'@'), BLANK);
        assert_eq!(glyph_of(b'~'), MINUS);
        // The minus is one bar in the middle row.
        let lit_at = |r| (0..COLUMNS).filter(|&c| lit(MINUS, c, r)).count();
        assert_eq!([0, 1, 2, 3, 4].map(lit_at), [0, 0, 3, 0, 0]);
    }

    #[test]
    fn the_narrow_print_maps_and_steps_its_own_way() {
        assert_eq!(narrow_glyph_of(b'='), DOT);
        assert_eq!(narrow_glyph_of(b'A'), BLANK);
        assert_eq!(narrow_glyph_of(b'b'), 11);
        let frame = Frame {
            pitch: 400,
            wide: 4,
            len: 400 * 10,
        };
        let mut runs = Vec::new();
        // "1 1": the second '1' 4 + 2 pixels on; the count stops before the third.
        narrow_runs(b"1 11", 3, (0, 0), 1, frame, &mut runs);
        assert!(runs.contains(&(4, 1)) && runs.contains(&(4 + 6 * 4, 1)));
        assert!(!runs.contains(&(4 + 10 * 4, 1)));
    }

    #[test]
    fn the_text_wraps_at_the_width() {
        assert_eq!(advance(8, 2, 2, 100), (16, 2));
        assert_eq!(advance(96, 2, 2, 100), (4, 14));
    }

    #[test]
    fn a_print_goes_on_from_the_cursor_and_ends_at_the_frame() {
        let frame = Frame {
            pitch: 64,
            wide: 2,
            len: 64 * 4,
        };
        let mut runs = Vec::new();
        let end = print_runs(b"1\0x", (0, 0), (3, 1), 1, 32, frame, &mut runs);
        assert_eq!(end, (7, 1));
        // '1': rows 0x0100, 0x1100, 0x0100 (its fourth and fifth past the frame).
        let at = |c: u32, r: u32| (1 + r) * 64 + (3 + c) * 2;
        assert_eq!(
            runs,
            vec![(at(1, 0), 1), (at(0, 1), 1), (at(1, 1), 1), (at(1, 2), 1)]
        );
        // A scaled pixel's run cut at the frame's end.
        runs.clear();
        glyph_runs(BLANK, 0, 0, 3, frame, &mut runs);
        assert!(runs.is_empty());
        glyph_runs(1, 30, 0, 3, frame, &mut runs);
        assert!(runs.iter().all(|&(at, n)| at + 2 * n <= frame.len));
    }
}
