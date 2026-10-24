//! `race::choose_save_slot` (0x41E6DC): the viewer's stage tidied away
//! (`race::viewer_tidy_stage`), the saving screen of the replay slots
//! (`menu::replay_slot_loop`, eax as given: the caller's 1), the stage loaded again
//! (`race::viewer_load_stage`).
//!
//! In:  eax 1 (the saving screen)
//! Out: eax the slot chosen or -1 (the screen's; the loading keeps it); the rest as the
//!      screen leaves them
//!
//! Caller: `race::viewer_action` (0x16). Its calls through `part`: 1 `viewer_tidy_stage`,
//! 2 `replay_slot_loop`, 3 `viewer_load_stage`; in the game their Rust versions. Ungated (a
//! screen). Shadow mode runs the original alone. Self-test: a copy whose calls go to
//! stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::menu::screen_loop::Carried;

/// The Rust version: the slot chosen (eax; -1 for none). `c` the carried values the
/// slot screen is run with (`c.arg` non-zero: the saving screen).
pub fn choose_save_slot(c: &Carried) -> Result<u32, Refused> {
    // Both keep every register.
    crate::race::viewer_tidy_stage::viewer_tidy_stage()?;
    let slot = crate::menu::replay_slot_loop::replay_slot_loop(c)?;
    crate::race::viewer_load_stage::viewer_load_stage()?;
    Ok(slot)
}
