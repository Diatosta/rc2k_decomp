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
//! anything) a length below 3 (the original's loop then runs on: with 0 its copy, with 1 or 2
//! its drawing loop) and one whose copies pass the 0x100 bytes of `byte_A8BC54`. Compared in
//! shadow mode (`dispatch_draw`). Self-test: lengths 3 .. 8 of digit texts, two fonts and
//! places, nothing drawn.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_write, rva};
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::{Canvas, draw_text_at};
use crate::text::font::Font;

const CELLS_RVA: usize = rva(0xA8BC54); // byte_A8BC54
const CELLS_LEN: usize = 0x100;
const LENGTH_RVA: usize = rva(0x7559B8); // dword_7559B8

/// The Rust version: `text` .. `end`, the column of the last character at `x` + 0x32.
pub unsafe fn draw_spaced_number(
    font: *mut Font,
    text: *const u8,
    end: *const u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::screens::captions::spaced_places`.
        let n = (end as u32).wrapping_sub(text as u32);
        let places = rc2k_core::screens::captions::spaced_places(n, x, CELLS_LEN)?;
        game_write(LENGTH_RVA, n);
        let cells = game_addr(CELLS_RVA) as *mut u8;
        for i in 0..n as usize {
            *cells.add(2 * i) = *text.add(i);
            *cells.add(2 * i + 1) = 0;
        }
        for (i, x) in places {
            draw_text_at(font, cells.add(2 * i), x, y, canvas)?;
        }
        Ok(())
    }
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe {
        draw_spaced_number(
            r.eax as *mut Font,
            r.ebx as *const u8,
            r.ebp as *const u8,
            r.ecx as i32,
            r.edx as i32,
            canvas,
        )
    }?;
    Ok(Left::carry(false))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_spaced_number", r, &rust)
}
