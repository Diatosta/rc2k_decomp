//! The 0x20-byte font object that `draw_text` takes.

/// `repr(C)` with the game's layout. Two fields change meaning with `mode`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    /// +0x14: palette address passed to the blitter. In `MODE_MEASURE` it is used as a
    /// counter instead: one is added per line.
    pub palette: u32,
    /// +0x18: glyph table address.
    pub glyph_table: u32,
    /// +0x1C: width of the box that lines are aligned in, and the limit for
    /// `FLAG_FAIL_IF_WIDER`. In `MODE_MEASURE` it receives the width of the widest line.
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

/// Stop with the carry flag set as soon as a measured line gets wider than the box.
pub const FLAG_FAIL_IF_WIDER: u8 = 2;
/// Pull each glyph left as far as the outline of the line so far allows.
pub const FLAG_KERNING: u8 = 4;
