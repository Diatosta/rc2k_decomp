//! `install::mark_main_page` (0x4141EF): the main page's items marked by the installation's
//! state. Not this version (`dword_51B9C8` != 1): item +0x28 gets 0x80 (unusable); no
//! installation (`g_installStatus` 0): item +0x38 gets 0x80; registration not offered
//! (`dword_51B9E4` != 1): item +0x48 gets 0x40 (hidden). If the title word (`[edi+2Ah]`) is
//! 0xBD (the update script): +0x28 loses 0x80, +0x38 and +0x48 get 0x40.
//!
//! In:  edi the list
//! Out: every register kept; the flags of its last test (no caller reads them)
//! A page action; `ebx` is ignored. Caller `install::run_page`. Shadow mode compares.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{OFFER_REGISTRATION, SAME_VERSION, STATUS, g, gb, rb, rw, wb};
use rc2k_core::install::pages;

pub fn mark_main_page(list: u32) {
    let title = rw(list + 0x2A);
    // Converted: `rc2k_core::install::pages::main_page_marks`.
    let marks = pages::main_page_marks(
        g(SAME_VERSION) == 1,
        gb(STATUS) != 0,
        g(OFFER_REGISTRATION) == 1,
        title,
    );
    for (offset, or, and) in marks {
        wb(list + offset, (rb(list + offset) | or) & and);
    }
}
