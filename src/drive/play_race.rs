//! `drive::play_race` (0x444CEE): "CALLING DOPLAYGAME - START" logged between rules, the
//! log off (`g_logEnabled` 0) for `drive::race_loop`, back on, "... - FINIS" between
//! rules.
//!
//! In:  nothing
//! Out: the registers as `race_loop` leaves them; no flags read
//!
//! Callers: `race::run_race`, `sub_443F96`, 0x44402F, `race::credits_race`, `sub_4640AA`.
//! Refuses what `race_loop` refuses.

use crate::drive::race_parts::log_line;
use crate::errors::Refused;

const EMPTY: usize = 0x4E00B7; // what `log_newline_l1` writes

/// The Rust version. The registers it leaves are read by no caller (liveness: esi after
/// `race::run_race`'s and `sub_443F96`'s calls, ecx edi edx esi after
/// `race::credits_race`'s: no read point; only DF after `net::lets_race`'s).
pub unsafe fn play_race() -> Result<(), Refused> {
    log_line(EMPTY);
    log_line(0x4DC41B);
    log_line(0x4DC436);
    crate::logfile::write_line::set_enabled(0); // g_logEnabled
    unsafe { crate::drive::race_loop::race_loop() }?;
    crate::logfile::write_line::set_enabled(1);
    log_line(0x4DC451);
    log_line(0x4DC46C);
    log_line(EMPTY);
    Ok(())
}
