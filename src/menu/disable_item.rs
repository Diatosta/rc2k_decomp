//! `menu::disable_item` (0x46B4F6): an item of the current menu greyed out.
//!
//! In:  ebx = the item's number (from 1)
//! Out: every register preserved
//!
//! The item (`menu::item_at`): +0x48 = 0xFFFFFC1C (the moves up and down pass over it)
//! and its font +0x18 = the grey one at 0x51EA04. Its twin `menu::enable_item`
//! (0x46B50F) puts 0xFFFFFC1B and the font at 0x51E8E4 back. Eight callers each (the
//! player details screens: the controller's item when there is no controller).
//! Refuses item 0.
//! Self-test: items of a menu of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr};
use crate::menu::item_at::item_at;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

use rc2k_core::menu::items::{Font, choosable};

const GREY_FONT: usize = 0x51EA04;
const PLAIN_FONT: usize = 0x51E8E4;

// ---- Rust version ----

// Converted: `rc2k_core::menu::items::choosable`.

/// The item's +0x48 and font set: what both twins do.
pub fn set_item(number: u32, on: bool) -> Result<(), Refused> {
    let item = item_at(number)?;
    let (state, font) = choosable(on);
    let font = match font {
        Font::Plain => PLAIN_FONT,
        Font::Grey => GREY_FONT,
    };
    unsafe {
        ((item + 0x48) as *mut u32).write_unaligned(state);
        ((item + 0x18) as *mut u32).write_unaligned(game_addr(crate::framework::rva(font)) as u32);
    }
    Ok(())
}

pub fn disable_item(number: u32) -> Result<(), Refused> {
    set_item(number, false)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    disable_item(r.ebx)?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
