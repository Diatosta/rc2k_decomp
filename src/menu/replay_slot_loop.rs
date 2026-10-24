//! `menu::replay_slot_loop` (0x46F786): the replay slots' screen, by eax: 0 the loading
//! one (`menu::load_replay_loop`), else the saving one (`menu::save_replay_loop`); each
//! left in eax the slot chosen, -1 for none (their `jmp`-less tail calls: what they leave
//! is what the caller gets).
//!
//! In:  eax 0 to load, else to save
//! Out: as the screen leaves them (eax the slot or -1)
//!
//! Callers: `race::replay_viewer` (modes 2, 3), `race::viewer_action` (0x14),
//! `race::choose_save_slot`, 0x46F79C, 0x46F7A7. A jump at +3 (the patch would cut it):
//! private copy, manual patch, both calls relocated. Its calls through `part` (1 the
//! saving screen, 2 the loading one; in the game their Rust versions,
//! `screen_loop::run_screen`). Ungated (a screen runs for long; the screens gate their
//! frames). Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins, against the Rust version, eax 0, 1 and 7.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

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
    Ok(unsafe { run_screen(screen, c) }?.unwrap_or(c.arg))
}
