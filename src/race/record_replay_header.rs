//! `race::record_replay_header` (0x460246): the race's settings (the arcade's seed, start
//! place and count, the stage, the mode, the players, the cars, the first four kinds, the
//! two players' absences) into the replay's car header (`dword_719B14` ..), each logged
//! ("[replayheader.x]" and the value).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_end.rs`. Callers: `sub_443F0B`, `sub_443F96` (`race::open_replay`
//! follows; no flags read). Shadow mode runs the original alone (it logs). Self-test: a
//! copy whose 26 log calls note themselves, against the Rust version on the same, the
//! header's fields filled two ways.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_end::*;

/// `log_write_l1` / `log_int_l1_nl` in the game.
fn log_part(place: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe {
        if place == LOG_WRITE {
            crate::logfile::log_write::log_write(r.eax as *const u8, false, 1);
        } else {
            crate::logfile::log_int::log_int(r.eax as i32, true, 1);
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { record_replay_header(r, &mut log_part) }?;
    Ok(())
}
