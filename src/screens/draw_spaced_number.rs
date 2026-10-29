//! `screens::draw_spaced_number` (0x468180): draws a text one character at a time from its
//! last, right to left, in fixed columns: the last at x + 0x32, the next two 9 apart, the rest
//! 0xD apart (so "12:34" style figures line up). Each character is copied with a NUL after it
//! into `byte_A8BC54` (two bytes each) and drawn from there (`text::draw_text_keep_regs`);
//! the length is kept in `dword_7559B8`.
//!
//! In:  eax = font, ebx = text, ebp = its end, ecx = x, edx = y
//! Out: every register preserved; CF clear (the loop's last `cmp` is equal)
//!
//! Callers: the players' standings pages (0x4836CF, 0x48D32E; eight calls). Refuses (before
//! anything) a length below 3 (the original's loop then runs on: with 0 its copy, with 1 or 2 its
//! drawing loop) and one whose copies pass the 0x100 bytes of `byte_A8BC54`.

use crate::errors::Refused;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::pointers::GamePtr;
use crate::text::draw_text::{Canvas, draw_text_at};

const CELLS_LEN: usize = 0x100;

/// The Rust version: `text` .. `end`, the column of the last character at `x` + 0x32.
pub fn draw_spaced_number(
    font: GamePtr,
    text: GamePtr,
    end: GamePtr,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::captions::spaced_places`.
    let n = end.addr().wrapping_sub(text.addr());
    let places = rc2k_core::screens::captions::spaced_places(n, x, CELLS_LEN)?;
    DRAW_WORK.work_b8.set(n);
    let cells = crate::screens::text_buffers_store::TEXT_BUFFERS
        .cells
        .addr();
    let digits = text.read_bytes(n as usize, "the digits")?;
    let mut spaced = Vec::with_capacity(2 * digits.len());
    for &digit in &digits {
        spaced.extend([digit, 0]);
    }
    GamePtr::new(cells).write_bytes(&spaced, "the spaced digits")?;
    for (i, x) in places {
        draw_text_at(
            font,
            crate::pointers::GamePtr::new((cells as usize + 2 * i) as u32),
            x,
            y,
            canvas,
        )?;
    }
    Ok(())
}
