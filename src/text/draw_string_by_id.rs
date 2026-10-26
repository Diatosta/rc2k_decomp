//! `text::draw_string_by_id` (0x40234A): builds the text of a string id in the shared text
//! buffer, NUL-terminates it and draws it with `draw_text`.
//!
//! In:  eax = string id, ebx = font, ecx = x, edx = y
//! Out: as `draw_text` leaves it: carry, eax = 0, ebx = position in the text buffer, ecx = x,
//!      edx = y; esi, edi and ebp preserved
//! A text longer than the buffer is refused: the original writes past it.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::{Canvas, GameCanvas, Outcome, draw_text};
use crate::text::font::Font;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;

// ---- Rust version ----

/// The text of string `id` with its NUL, as the original leaves it in the text buffer.
pub unsafe fn string_text(id: u32) -> Result<Vec<u8>, BadInput> {
    let mut out = Vec::new();
    unsafe { append_string_by_id(id, &mut |p, n| out.extend_from_slice(from_raw_parts(p, n))) };
    out.push(0);
    if out.len() > TEXT_LEN {
        return Err(BadInput(
            "text longer than the text buffer: the original writes past it",
        ));
    }
    Ok(out)
}

/// `buffer` is where the text is built: the game's text buffer, or a private one of at least
/// `TEXT_LEN` bytes when the game's must not be touched. `Outcome::text` points into it.
pub unsafe fn draw_string_by_id(
    id: u32,
    font: &mut Font,
    x: i32,
    y: i32,
    buffer: *mut u8,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    unsafe {
        let bytes = string_text(id)?;
        copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len());
        Ok(draw_text(font, buffer, x, y, canvas)?)
    }
}

/// The same with a font object in game memory and the game's text buffer, for Rust callers.
pub unsafe fn draw_string_by_id_at(
    id: u32,
    font: *mut Font,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    unsafe {
        let before = font.read_unaligned();
        let mut copy = before;
        let out = draw_string_by_id(id, &mut copy, x, y, text(), canvas)?;
        if copy != before && canvas.keeps_font_changes() {
            font.write_unaligned(copy);
        }
        Ok(out)
    }
}

/// The original's registers (see the header) around the Rust version, on the game's screen:
/// for Rust callers that hold registers. Nothing is written when it refuses.
pub unsafe fn draw_string_by_id_regs(r: &mut Regs) -> Result<(), Refused> {
    let (id, font, x, y) = (r.eax, r.ebx as *mut Font, r.ecx as i32, r.edx as i32);
    let mut copy = unsafe { font.read_unaligned() };
    let out = unsafe { draw_string_by_id(id, &mut copy, x, y, text(), &mut GameCanvas) }?;
    if copy != unsafe { font.read_unaligned() } {
        unsafe { font.write_unaligned(copy) };
    }
    (r.eax, r.ebx, r.ecx, r.edx) = (0, out.text as u32, out.x as u32, out.y as u32);
    r.set_carry(out.too_wide);
    r.set_zero(!out.too_wide);
    Ok(())
}
