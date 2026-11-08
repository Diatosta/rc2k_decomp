//! `menu::replay_slot_loop` (0x46F786): the replay slots' screen, by eax: 0 the loading
//! one (`menu::load_replay_loop`), else the saving one (`menu::save_replay_loop`); each
//! left in eax the slot chosen, -1 for none (their `jmp`-less tail calls: what they leave
//! is what the caller gets).
//!
//! Callers: `race::replay_viewer` (modes 2, 3), `race::viewer_action` (0x14),
//! `race::choose_save_slot`, 0x46F79C, 0x46F7A7.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};

/// The Rust version: the slot chosen (-1 for none), on the saving screen when `saving`
/// (the original's eax non-zero), else on the loading one.
pub fn replay_slot_loop(menus: &Menus, saving: bool) -> Result<u32, Refused> {
    let screen = if saving {
        &crate::menu::save_replay_loop::SCREEN
    } else {
        &crate::menu::load_replay_loop::SCREEN
    };
    let slot = { run_screen(screen, &Entry::new(menus)) }?;
    // Both screens leave the slot in eax (their `result`).
    Ok(slot.ok_or(BadInput("a replay slot screen without its result"))?)
}
