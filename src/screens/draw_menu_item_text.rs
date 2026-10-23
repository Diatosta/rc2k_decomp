//! `screens::draw_menu_item_text` (0x471E99): draws a menu item's text.
//!
//! In:  eax = string id, ebx = font, ecx = x, edx = y, ebp = 1 for spaced-out letters
//! Out: every register preserved
//! Normally one `draw_string_by_id`. With ebp = 1 the string is built in the line buffer
//! (`g_lineBuffer`) and drawn one character at a time, 18 pixels apart, from x - 26, each
//! through a one-character text at `byte_A8BC54`; the font is kept in `dword_7559EC`.
//! One caller, `draw_menu_items`.
//! Manual patch with a private copy (0x57 bytes, four calls relocated): a conditional jump
//! sits in the first bytes.
//! An empty string spaced out draws on from the bytes the line buffer held after its NUL,
//! as the original does (seen on the load screen's empty slots in the standalone program,
//! 2026-10-14). Refuses, in the spaced-out way, a string of 0x100 bytes or more (it would
//! run into the one-character text).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_ptr};
use crate::screens::LINE_BUFFER_RVA;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_string_by_id::string_text;
use crate::text::draw_text::Canvas;
use crate::text::font::Font;

// ---- game data ----

// dword_7559EC
// byte_A8BC54
// from g_lineBuffer to byte_A8BC54

// ---- Rust version ----

pub unsafe fn draw_menu_item_text(
    id: u32,
    font: *mut Font,
    x: i32,
    y: i32,
    spaced: bool,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::menus::item_text`; the line buffer past the string read where
    // the original reads it.
    use crate::screens::game_screen::{FontAt, MenuScreen};
    if spaced {
        // What the original refuses first: a string past the text buffer.
        unsafe { string_text(id) }?;
    }
    let line: *const u8 = game_ptr(LINE_BUFFER_RVA);
    rc2k_core::screens::menus::item_text(
        &mut MenuScreen::new(canvas),
        id,
        FontAt(font as u32),
        font as u32,
        x,
        y,
        spaced,
        &|n| unsafe { *line.add(n) },
    )
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe {
        draw_menu_item_text(
            r.eax,
            r.ebx as *mut Font,
            r.ecx as i32,
            r.edx as i32,
            r.ebp == 1,
            canvas,
        )
    }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_menu_item_text", r, &rust)
}
