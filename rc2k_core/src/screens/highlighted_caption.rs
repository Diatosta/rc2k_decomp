//! The highlighted entry's caption on a button at the bottom of the screen
//! (from `screens::draw_highlighted_caption` 0x4695C6): the caption (string
//! 0x1004, which reads the caption the game put in the caption buffer) measured, the button as
//! wide as it plus 12 centred on x 0x197 (sprite 0xF, pieces of 32 of sprite 0xB, the last
//! drawn over the one before so the width comes out, sprite 0x10), then the caption in the
//! font blended by the time since the last key.

use crate::Invalid;
use crate::screens::screen::Screen;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E824`.
    Measure,
    /// `stru_51E964`, `stru_51E844`: blended into `stru_51E384`.
    A,
    B,
    Blended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `g_markerX`, `g_markerY`.
    MarkerX,
    MarkerY,
    /// `seconds`, `dword_7559B0`, `dword_7559B4`, `glyph`, `dword_7559C4`: the button's
    /// width, x, y, left and right sprites.
    Width,
    X,
    Y,
    Left,
    Right,
    /// `dword_755A20` .. `dword_755A2C`: where its two ends were drawn.
    LeftX,
    LeftY,
    RightX,
    RightY,
}

/// The caption's string.
pub const CAPTION_STRING: u32 = 0x1004;

/// The button and caption, measured from x `measured_at` (the caller's ecx); the carry of
/// the caption's text. A width of 0 is the original's fatal error 0x80701010; refused: a
/// caption narrower than 20 (the original then draws 2^32 middle pieces).
pub fn highlighted_caption<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    measured_at: i32,
    since_key: u32,
) -> Result<bool, S::Error> {
    s.keep(Work::MarkerX, 0x197);
    s.keep(Work::MarkerY, 0x1FE);
    let measured = s.measure_string(Font::Measure, CAPTION_STRING, measured_at)?;
    let width = measured.wrapping_add(0x0C);
    let y = 0x1FEu32;
    let mut x = 0x197u32.wrapping_sub((width >> 1) + 0x22);
    s.keep(Work::Width, width);
    s.keep(Work::X, x);
    s.keep(Work::Y, y);
    s.keep(Work::Left, 0x0F);
    s.keep(Work::Right, 0x10);
    s.sprite(0x0F, x as i32, y as i32)?;
    s.keep(Work::LeftX, x);
    s.keep(Work::LeftY, y);
    x = x.wrapping_add(0x22);
    s.keep(Work::X, x);
    if width == 0 {
        return Err(s.fatal(0x8070_1010, ""));
    }
    let (wholes, rest) = (width / 32, width % 32);
    if wholes == 0 {
        return Err(Invalid("a caption too narrow for one middle piece").into());
    }
    for _ in 0..wholes {
        s.sprite(0x0B, x as i32, y as i32)?;
        x = x.wrapping_add(32);
    }
    if rest != 0 {
        x = x.wrapping_sub(32).wrapping_add(rest);
        s.sprite(0x0B, x as i32, y as i32)?;
        x = x.wrapping_add(32);
    }
    s.sprite(0x10, x as i32, y as i32)?;
    s.keep(Work::X, x);
    s.keep(Work::RightX, x);
    s.keep(Work::RightY, y);
    s.keep(Work::MarkerX, 0x195);
    s.keep(Work::MarkerY, 0x1FB);
    s.blend(since_key, Font::A, Font::B)?;
    Ok(s.string(Font::Blended, CAPTION_STRING, 0x195, 0x1FB)?
        .too_wide)
}
