//! `install::show_backdrop` (0x41525D): three frames (`screens::run_frame`) of the
//! installer's backdrop alone (`install::backdrop_frame`), as a page opens and closes.
//!
//! In:  nothing (the callers set or clear carry before it; it does not read it)
//! Out: every register kept (`pusha` / `popa`); the flags of the last frame's run and
//!      `dec ebp` (zero set; no caller reads them)
//! Caller `install::run_page` (twice).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::run_frame;

/// The backdrop's frame run three times.
pub fn show_backdrop() -> Result<(), Refused> {
    for _ in 0..3 {
        run_frame(Some(crate::install::backdrop_frame::backdrop_frame))?;
    }
    Ok(())
}
