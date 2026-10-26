//! `menu::draw_menu` (0x4746B7): draws a menu, its fixed labels and then its items.
//!
//! In:  edi = the menu; ebx, ecx as they come (`draw_menu_items` uses them)
//! Out: every register preserved
//! `pusha`, `screens::draw_menu_labels`, `screens::draw_menu_items`, `popa`. Called by
//! `menu::draw_frame` and `menu::frame_callback`, and by the screens with a frame callback
//! of their own.
//! Refuses what the two functions refuse.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::draw_menu_items::draw_menu_items;
use crate::screens::draw_menu_labels::draw_menu_labels;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

// ---- Rust version ----

/// `ebx` and `ecx` are those registers at entry.
pub unsafe fn draw_menu(
    menu: u32,
    ebx: u32,
    ecx: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        draw_menu_labels(menu, canvas)?;
        draw_menu_items(menu, ebx, ecx, canvas)
    }
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_menu(r.edi, r.ebx, r.ecx, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    // As for `draw_menu_items`: the blended font is put back before the original runs.
    draw_entry("menu::draw_menu", r, &rust)
}

// ---- self-test: a menu of its own with two labels and three plain items, each of them
// highlighted in turn, with the game's strings and fonts ----
