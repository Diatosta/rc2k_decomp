//! `menu::replay_slot_loop` (0x46F786): the replay slots' screen, by eax: 0 the loading
//! one (`menu::load_replay_loop`), else the saving one (`menu::save_replay_loop`); each
//! left in eax the slot chosen, -1 for none (their `jmp`-less tail calls: what they leave
//! is what the caller gets).
//!
//! In:  eax 0 to load, else to save
//! Out: as the screen leaves them (eax the slot or -1)
//!
//! Callers: `race::replay_viewer` (modes 2, 3), `race::viewer_action` (0x14),
//! `race::choose_save_slot`, 0x46F79C, 0x46F7A7.

use crate::errors::Refused;
use crate::menu::screen_loop::{Carried, run_screen};

/// The Rust version: the slot chosen (eax; -1 for none). `c` the carried values the screen
/// is run with; `c.arg` 0 to load, else to save.
pub fn replay_slot_loop(c: &Carried) -> Result<u32, Refused> {
    let screen = if c.arg != 0 {
        &crate::menu::save_replay_loop::SCREEN
    } else {
        &crate::menu::load_replay_loop::SCREEN
    };
    // (A screen with no result leaves eax as it was.)
    Ok({ run_screen(screen, c) }?.unwrap_or(c.arg))
}
