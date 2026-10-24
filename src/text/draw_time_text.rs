//! `text::draw_time_text` (0x402395): draws a time's text one character at a time, each
//! with the font's fixed width (+0x1C) set to 6 for ':' and '.', 0x10 for the others, so
//! that the digits keep their columns; at most 11 characters, up to the NUL.
//!
//! In:  eax = font, ebx = text, ecx = x, edx = y
//! Out: every register kept (`pusha` .. `popa`); the flags of the loop's last test (no
//!      caller reads them)
//!
//! Each character is drawn as a text of its own: the dword after it is swapped for 0
//! (`xchg edi, [ebx+1]`, edi 0) around `text::draw_text_keep_regs`, then put back; x moves
//! on by the width. The font's +0x1C is 0 at the end. Callers: `net::lobby_frame` (twice).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::text::font::Font;

/// The font's fixed width.
const WIDTH: u32 = 0x1C;

fn read32(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

fn write32(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The Rust version (see the header): the time text at `text` in `font` at (`x`, `y`).
/// The flags the original leaves are read by neither caller (register liveness,
/// 2026-10-23: after the two calls DF, ebp, ebx/edi, edx, esi, all kept).
pub unsafe fn draw_time_text(font: *mut Font, text: u32, x: u32, y: u32) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    let font_at = font as u32;
    let (mut at, mut x, mut edi) = (text, x, 0u32);
    // Converted: `rc2k_core::text::time_text_widths`; here the original's way of drawing
    // one character at a time (the dword after it swapped for 0 around the call).
    let chars = unsafe { crate::bridge::Raw::new(text as *const u8) };
    for width in crate::bridge::read_through(rc2k_core::text::time_text_widths(&chars)) {
        write32(font_at + WIDTH, width);
        let kept = read32(at + 1);
        write32(at + 1, edi);
        edi = kept;
        // `text::draw_text_keep_regs` on a copy of the font, written back if it changed.
        unsafe {
            let mut copy = font.read_unaligned();
            crate::text::draw_text_keep_regs::draw_text_keep_regs(
                &mut copy,
                at as *const u8,
                x as i32,
                y as i32,
                &mut GameCanvas,
            )?;
            if copy != font.read_unaligned() {
                font.write_unaligned(copy);
            }
        }
        let zero = read32(at + 1);
        write32(at + 1, edi);
        edi = zero;
        x = x.wrapping_add(width);
        at = at.wrapping_add(1);
    }
    write32(font_at + WIDTH, 0);
    Ok(())
}
