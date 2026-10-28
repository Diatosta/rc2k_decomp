//! `install::run_pages` (0x4144AA): the installer's pages, from the entry esi on: each entry
//! is the page's list and the entries that follow its answers (+4 + 4 x answer); a page
//! answered 0 or 1 has its highlight put back to 0; an answer of no entry (0) or the game
//! quitting ends it. (The original calls itself with `jnz` to its start.)
//!
//! In:  esi the first entry (`off_51D594`, or `off_51D5D8` for an update)
//! Out: esi 0 (or the next entry when quitting), edi the last page's list, eax its answer;
//!      the flags of the last test
//! Caller `sub_412DE0` (`install::run_installer`, twice).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{rd, wd};
use crate::session::FLAGS;
use rc2k_core::install::pages;

/// The pages from the entry `first` on.
pub fn run_pages(first: u32) -> Result<(), Refused> {
    let mut entry = first;
    // The entry before, and the passed-over pages' ways (`pass_over`).
    let (mut prev, mut passed) = (0, Vec::new());
    loop {
        let list = rd(entry);
        if let Some(next) = pass_over(entry, prev, &mut passed) {
            (prev, entry) = (entry, next);
            continue;
        }
        let answer = crate::install::run_page::run_page(list)?;
        // Converted: `rc2k_core::install::pages` (`resets_highlight`, `next_entry_at`).
        if pages::resets_highlight(answer) {
            wd(list + 4, 0);
        }
        prev = entry;
        entry = rd(entry.wrapping_add(pages::next_entry_at(answer)));
        if FLAGS.quitting.get() == 1 || entry == 0 {
            return Ok(());
        }
    }
}

/// Whether `list`'s action (+8) is the page at IDA `page`.
fn is_page(list: u32, page: usize) -> bool {
    list != 0 && crate::install::installer::ida_of(rd(list + 8)) == page
}

/// A page passed over: its entry, the entry it was come to from, the one it went on to.
struct Passed {
    entry: u32,
    before: u32,
    after: u32,
}

/// The pages not shown, passed over as if answered:
/// - the installation-kind page (small / normal / full): always the full installation,
///   `g_mediaMode` 3 (owner, 2026-10-28: nobody installs another, and the game no longer
///   reads a CD); on to its answer 3's entry;
/// - the drive page on a system without drives (`install::folders`, `DRIVES` false): on
///   to the entry of its answer that is the folder page.
///
/// Come back to from where it went on to, a page goes back to where it was come to from.
/// None for a page that is shown.
fn pass_over(entry: u32, prev: u32, passed: &mut Vec<Passed>) -> Option<u32> {
    use crate::install::folders::{Folders, Native};
    const MODE_PAGE: usize = 0x41422B;
    const DRIVE_PAGE: usize = 0x41425E;
    const FOLDER_PAGE: usize = 0x4142F5;
    const FULL: u32 = 3;
    let list = rd(entry);
    let on = if is_page(list, MODE_PAGE) {
        crate::install::installer::set_media_mode(FULL);
        rd(entry.wrapping_add(pages::next_entry_at(FULL)))
    } else if !<Native as Folders>::DRIVES && is_page(list, DRIVE_PAGE) {
        (0..8)
            .map(|answer| rd(entry.wrapping_add(pages::next_entry_at(answer))))
            .find(|&next| next != 0 && is_page(rd(next), FOLDER_PAGE))?
    } else {
        return None;
    };
    if let Some(p) = passed.iter().find(|p| p.entry == entry)
        && prev != 0
        && prev == p.after
    {
        return Some(p.before);
    }
    passed.retain(|p| p.entry != entry);
    passed.push(Passed {
        entry,
        before: prev,
        after: on,
    });
    Some(on)
}
