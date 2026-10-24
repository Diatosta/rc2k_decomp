//! `install::run_pages` (0x4144AA): the installer's pages, from the entry esi on: each entry
//! is the page's list and the entries that follow its answers (+4 + 4 x answer); a page
//! answered 0 or 1 has its highlight put back to 0; an answer of no entry (0) or the game
//! quitting ends it. (The original calls itself with `jnz` to its start.)
//!
//! In:  esi the first entry (`off_51D594`, or `off_51D5D8` for an update)
//! Out: esi 0 (or the next entry when quitting), edi the last page's list, eax its answer;
//!      the flags of the last test
//! Caller `sub_412DE0` (`install::run_installer`, twice). Its calls through `part`.
//! Ungated. Shadow mode runs the original alone. Self-test: a copy whose page runs go to a
//! stand-in answering as the test says, against the Rust version, on entries of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{rd, wd};
use crate::session::FLAGS;
use rc2k_core::install::pages;

/// The pages from the entry `first` on.
pub fn run_pages(first: u32) -> Result<(), Refused> {
    let mut entry = first;
    loop {
        let list = rd(entry);
        let answer = crate::install::run_page::run_page(list)?;
        // Converted: `rc2k_core::install::pages` (`resets_highlight`, `next_entry_at`).
        if pages::resets_highlight(answer) {
            wd(list + 4, 0);
        }
        entry = rd(entry.wrapping_add(pages::next_entry_at(answer)));
        if FLAGS.quitting.get() == 1 || entry == 0 {
            return Ok(());
        }
    }
}
