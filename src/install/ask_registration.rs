//! `install::ask_registration` (0x413650): `g_runRegistration` set, so the registration
//! program runs after the game.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD60`). Caller `install::run_page` through a page's item. Shadow
//! mode compares. Self-test: against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{RUN_REGISTRATION, p};
use crate::race::viewer::regs_entry;

pub fn ask_registration(entry: &Regs) -> Regs {
    p(RUN_REGISTRATION, 1);
    *entry
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(ask_registration(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}
