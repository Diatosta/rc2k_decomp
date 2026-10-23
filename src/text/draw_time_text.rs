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
//! Its call through `part` (`net::lobby_calls`: the Rust version).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::DRAW_KEEP;

/// The font's fixed width.
const WIDTH: u32 = 0x1C;

fn read32(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

fn write32(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The Rust version: the registers it leaves (the entry's).
pub fn draw_time_text(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.edi = 0;
    r.ebp = 11;
    // Converted: `rc2k_core::text::time_text_widths`; here the original's way of drawing
    // one character at a time (the dword after it swapped for 0 around the call).
    let text = unsafe { crate::bridge::Raw::new(r.ebx as *const u8) };
    for width in crate::bridge::read_through(rc2k_core::text::time_text_widths(&text)) {
        let x = r.ecx;
        r.esi = width;
        write32(r.eax + WIDTH, r.esi);
        let kept = read32(r.ebx + 1);
        write32(r.ebx + 1, r.edi);
        r.edi = kept;
        crate::net::lobby_calls::call(DRAW_KEEP, &mut r)?;
        let zero = read32(r.ebx + 1);
        write32(r.ebx + 1, r.edi);
        r.edi = zero;
        r.ecx = x.wrapping_add(r.esi);
        r.ebx = r.ebx.wrapping_add(1);
        r.ebp = r.ebp.wrapping_sub(1);
    }
    write32(r.eax + WIDTH, 0);
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}
