//! `install::finish_page` (0x414347): at the page's end (ebx negative) with the answer
//! (`byte_51D415`) 2, `byte_51BE21` becomes 5 and the answer 1 (the readme is shown).
//!
//! In:  ebx the state
//! Out: every register kept; the flags of its last test
//! Caller `install::run_page`. Shadow mode compares.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{LIST_ANSWER, gb, pb};

/// `state` the list action's (ebx).
pub fn finish_page(state: u32) {
    // Converted: `rc2k_core::install::pages::shows_readme`.
    if (state as i32) < 0 && rc2k_core::install::pages::shows_readme(gb(LIST_ANSWER)) {
        pb(0x51BE21, 5);
        pb(LIST_ANSWER, 1);
    }
}
