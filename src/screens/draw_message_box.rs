//! `screens::draw_message_box` (0x415EA6): string 0x1FE ("not in this version" and the
//! like) in a box in the middle of the surface: the string measured (font 0x51E824, x as
//! the caller's ecx), the box 0x32 wider and 0x19 higher (`dword_52983C` .. `dword_529848`:
//! x, y, width, height) filled with colour 1 (`gfx::fill_rect`) and framed with colour 0xA
//! (`gfx::frame_rect`), both through `g_fontPalettes`, then the string drawn centred (font
//! 0x51E904).
//!
//! In:  ecx = the x the measuring starts at
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `screens::cheat_locked_note` .. (0x4AAAD5, 0x48ADF2 and others). Refuses what
//! `gfx::fill_rect` refuses (a surface of neither 16 nor 32 bits), before anything.

use crate::errors::{BadInput, Refused};
use crate::gfx::fonts_store::FONTS;
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::text::draw_text::Canvas;
use rc2k_core::screens::message_box::{FILL, FRAME, STRING, layout};

/// IDA's address of the box's rectangle.
pub const BOX_START: usize = 0x52983C;

/// `dword_52983C` .. `dword_529848`: the box's rectangle (left, top, width, height).
pub static BOX: [crate::session::Dword; 4] = [const { crate::session::Dword::new(0) }; 4];

/// The Rust version (`x` the caller's ecx).
pub fn draw_message_box(x: i32, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::gfx::rect::{Palette, fill_rect, frame_rect};
    use crate::text::draw_string_by_id::draw_string_by_id_at;
    use crate::text::measure_string_by_id::measure_string_by_id_at;
    let bits = crate::gfx::surface::surface().bits_per_pixel;
    if bits != 16 && bits != 32 {
        return Err(BadInput("a message box on a surface of neither 16 nor 32 bits").into());
    }
    // Converted: `rc2k_core::screens::message_box::layout`.
    let m = measure_string_by_id_at(
        STRING,
        crate::gfx::typefaces::fonts::MEASURE.game_ptr(),
        x,
        canvas,
    )?;
    let (sw, sh) = (DRAW_SURFACE.width.get(), DRAW_SURFACE.height.get());
    let font = crate::gfx::typefaces::fonts::BOX_TEXT.game_ptr();
    let b = layout(
        m.width,
        m.outcome.y,
        sw,
        sh,
        crate::gfx::typefaces::fonts::BOX_TEXT
            .object()
            .line_height
            .get(),
    );
    BOX[2].set(b.rect[2]);
    BOX[3].set(b.rect[3]);
    BOX[0].set(b.rect[0]);
    BOX[1].set(b.rect[1]);
    let rect = crate::gfx::rect::rect_of(&BOX);
    let palettes = Palette::Held(FONTS.palettes.get());
    fill_rect(rect, FILL, palettes)?;
    frame_rect(rect, 0, 0, FRAME, palettes);
    draw_string_by_id_at(STRING, font, b.text_x, b.text_y, canvas)?;
    Ok(())
}

#[cfg(test)]
mod box_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&BOX, BOX_START, BOX_START + 16);
        assert_eq!(crate::moved::place(BOX_START + 8), BOX[2].addr() as usize);
    }
}
