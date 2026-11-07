//! `install::ask_registration` (0x413650): `g_runRegistration` set, so the registration
//! program runs after the game.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD60`). Caller `install::run_page` through a page's item.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

pub fn ask_registration(install: &'static crate::install::owner::Install) {
    install.run.run_registration.set(1);
}
