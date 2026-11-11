//! `screens::draw_spaced_number` (0x468180): draws a text one character at a time from its
//! last, right to left, in fixed columns: the last at x + 0x32, the next two 9 apart, the rest
//! 0xD apart (so "12:34" style figures line up). Each character is copied with a NUL after it
//! into the cells text buffer (two bytes each) and drawn from there (`text::draw_text_keep_regs`);
//! the length is kept in `draw_work.scratch_b8`.
//!
//! Takes the font, text, its end, x and y.
//!
//! Callers: the players' standings pages (`screens::draw_players_results_points`, 0x4836CF,
//! 0x48D32E; eight calls). Refuses (before anything) a length below 3 (the original's loop then
//! runs on: with 0 its copy, with 1 or 2 its drawing loop) and one whose copies pass the 0x100
//! bytes of the cells text buffer.

use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::text::draw_text::{Canvas, draw_text_at};

const CELLS_LEN: usize = 0x100;

/// The `digits`, the column of the last character at `x` + 0x32.
pub fn draw_spaced_number(
    screens: &Screens,
    font: impl Into<crate::gfx::typefaces::FontRef>,
    digits: &[u8],
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::captions::spaced_places`.
    let font = font.into();
    let n = digits.len() as u32;
    let places = rc2k_core::screens::captions::spaced_places(n, x, CELLS_LEN)?;
    screens.draw_work.scratch_b8.set(n);
    let mut spaced = Vec::with_capacity(2 * digits.len());
    for &digit in digits {
        spaced.extend([digit, 0]);
    }
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .cells
        .write_at(0, &spaced, "the spaced digits")?;
    for (i, x) in places {
        // The cells from character `i` on: a pair each, the whole run ending in the last NUL.
        draw_text_at(font, &spaced[2 * i..], x, y, canvas)?;
    }
    Ok(())
}
