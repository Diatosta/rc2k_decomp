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

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{RUN_FRAME, a, at};
use crate::race::viewer::regs_entry;

pub const FRAME: usize = 0x415273; // install::backdrop_frame

pub fn show_backdrop(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = Regs { ebp: 3, ..*entry };
    for _ in 0..3 {
        r.eax = a(FRAME);
        crate::install::installer::game_part(at(RUN_FRAME), &mut r)?;
        r.ebp = r.ebp.wrapping_sub(1);
        r.set_zero(r.ebp == 0);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| show_backdrop(r))
}
