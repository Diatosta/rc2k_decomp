//! `install::show_backdrop` (0x41525D): three frames (`screens::run_frame`) of the
//! installer's backdrop alone (`install::backdrop_frame`), as a page opens and closes.
//!
//! In:  nothing (the callers set or clear carry before it; it does not read it)
//! Out: every register kept (`pusha` / `popa`); the flags of the last frame's run and
//!      `dec ebp` (zero set; no caller reads them)
//! Caller `install::run_page` (twice). Its calls through `part` (the frame's run is the
//! platform's). Ungated (it runs frames). Shadow mode runs the original alone. Self-test:
//! a copy whose frame runs go to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
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
