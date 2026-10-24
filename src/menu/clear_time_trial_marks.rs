//! `menu::clear_time_trial_marks` (0x48C85C): the +0x40 of each of the chosen stage's ten
//! best times set to 0 (the mark of a new entry, set by `menu::enter_time_trial_best`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: the time trial's events. Refuses (before anything) what
//! `event_flow::stage_times` refuses. Self-test: the game's tables for each stage choice
//! there, the marks filled first, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;

pub fn clear_time_trial_marks() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::bests::clear_marks`.
    let mut list = crate::menu::rank_time_trial_best::stage_list()?;
    rc2k_core::menu::bests::clear_marks(&mut list);
    Ok(())
}
