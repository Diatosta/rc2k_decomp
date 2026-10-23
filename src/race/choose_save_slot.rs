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
use crate::framework::Regs;
use crate::race::viewer::regs_entry;

/// The Rust version: the registers it leaves.
pub fn choose_save_slot(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    for place in 1..=3 {
        crate::race::choose_save_slot::game_part(place, &mut r)?;
    }
    Ok(r)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::{viewer_load_stage as load, viewer_tidy_stage as tidy};
    *r = match place {
        1 => tidy::viewer_tidy_stage(r)?,
        2 => crate::menu::replay_slot_loop::replay_slot_loop(r)?,
        _ => load::viewer_load_stage(r)?,
    };
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| choose_save_slot(r))
}
