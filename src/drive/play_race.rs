//! `drive::play_race` (0x444CEE): "CALLING DOPLAYGAME - START" logged between rules, the
//! log off (`g_logEnabled` 0) for `drive::race_loop`, back on, "... - FINIS" between
//! rules.
//!
//! In:  nothing
//! Out: the registers as `race_loop` leaves them; no flags read
//!
//! Callers: `race::run_race`, `sub_443F96`, 0x44402F, `race::credits_race`, `sub_4640AA`.
//! Refuses what `race_loop` refuses. Hook removed 2026-10-22 (direct calls): Rust callers run
//! its `entry` (`calls::call`, `routes.rs`); the registers above are the original's, for
//! reference.

use crate::drive::race_parts::log_line;
use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::p;
use crate::screens::harness::Left;

const EMPTY: usize = 0x4E00B7; // what `log_newline_l1` writes

/// The Rust version.
pub unsafe fn play_race(entry: &Regs) -> Result<Left, Refused> {
    log_line(EMPTY);
    log_line(0x4DC41B);
    log_line(0x4DC436);
    p(0x4E0094, 0);
    let left = unsafe { crate::drive::race_loop::race_loop(entry) }?;
    p(0x4E0094, 1);
    log_line(0x4DC451);
    log_line(0x4DC46C);
    log_line(EMPTY);
    Ok(left)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { play_race(r) }?;
    left.apply(r);
    Ok(())
}
