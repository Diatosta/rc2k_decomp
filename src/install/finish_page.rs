//! `install::finish_page` (0x414347): at the page's end (ebx negative) with the answer
//! (`byte_51D415`) 2, `byte_51BE21` becomes 5 and the answer 1 (the readme is shown).
//!
//! In:  ebx the state
//! Out: every register kept; the flags of its last test
//! Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PAGES;

/// `state` the list action's (ebx).
pub fn finish_page(state: u32) {
    // Converted: `rc2k_core::install::pages::shows_readme`.
    if (state as i32) < 0
        && rc2k_core::install::pages::shows_readme(INSTALLER_PAGES.list_answer.get())
    {
        INSTALLER_NAMES.script_mark.set(5);
        INSTALLER_PAGES.list_answer.set(1);
    }
}
