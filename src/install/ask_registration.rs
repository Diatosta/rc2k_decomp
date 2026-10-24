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
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{RUN_REGISTRATION, p};

pub fn ask_registration() {
    p(RUN_REGISTRATION, 1);
}
