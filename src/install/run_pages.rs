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

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{QUITTING, at, cmp, g, rd, wd};
use crate::race::viewer::regs_entry;
use rc2k_core::install::pages;

const RUN_PAGE: usize = 0x4144CF; // install::run_page

pub fn run_pages(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    loop {
        r.edi = rd(r.esi);
        crate::install::installer::game_part(at(RUN_PAGE), &mut r)?;
        let answer = r.eax;
        cmp(&mut r, answer, 1);
        // Converted: `rc2k_core::install::pages` (`resets_highlight`, `next_entry_at`).
        if pages::resets_highlight(answer) {
            wd(r.edi + 4, 0);
        }
        r.esi = rd(r.esi.wrapping_add(pages::next_entry_at(answer)));
        let quitting = g(QUITTING);
        cmp(&mut r, quitting, 1);
        if quitting == 1 {
            return Ok(r);
        }
        let next = r.esi;
        crate::install::installer::logic(&mut r, next);
        if r.esi == 0 {
            return Ok(r);
        }
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| run_pages(r))
}
