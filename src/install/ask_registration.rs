//! `install::ask_registration` (0x413650): `g_runRegistration` set, so the registration
//! program runs after the game.
//!
//! The action of a list item. Caller `install::run_page` through a page's item.

pub fn ask_registration(install: &crate::install::owner::Install) {
    install.run.run_registration.set(1);
}
