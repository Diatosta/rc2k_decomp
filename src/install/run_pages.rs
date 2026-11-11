//! `install::run_pages` (0x4144AA): the installer's pages, from the first entry on: each entry
//! is the page's list and the entries that follow its answers (+4 + 4 x answer); a page
//! answered 0 or 1 has its highlight put back to 0; an answer of no entry (0) or the game
//! quitting ends it. (The original calls itself with `jnz` to its start.)
//!
//! Returns 0 (or the next entry when quitting), last page's list, its answer.
//! Caller 0x412DE0 (`install::run_installer`, twice).

use crate::errors::Refused;
use crate::install::page_records::PageEntry;
use crate::system::owner::system;
use rc2k_core::install::pages;

/// The pages from the entry `first` on.
pub fn run_pages(
    install: &'static crate::install::owner::Install,
    strings: &'static crate::strings::owner::Strings,
    first: PageEntry,
) -> Result<(), Refused> {
    let mut entry = first;
    // The entry before, and the passed-over pages' ways (`pass_over`).
    let (mut prev, mut passed) = (None, Vec::new());
    loop {
        let list = entry.list();
        if let Some(next) = pass_over(entry, prev, &mut passed) {
            (prev, entry) = (Some(entry), next);
            continue;
        }
        let answer = crate::install::run_page::run_page(install, strings, list)?;
        // Converted: `rc2k_core::install::pages` (`resets_highlight`, `next_entry_at`).
        if pages::resets_highlight(answer) {
            list.set_highlight(0);
        }
        prev = Some(entry);
        let Some(next) = entry.next(pages::next_entry_at(answer)) else {
            return Ok(());
        };
        entry = next;
        if system().session.flags.quitting.is_set() {
            return Ok(());
        }
    }
}

/// Whether `list`'s action (+8) is the page at IDA `page`.
fn is_page(entry: PageEntry, page: u32) -> bool {
    entry.has_list() && entry.list().action() == page
}

/// A page passed over: its entry, the entry it was come to from, the one it went on to.
struct Passed {
    entry: PageEntry,
    before: Option<PageEntry>,
    after: PageEntry,
}

/// The pages not shown, passed over as if answered:
/// - the installation-kind page (small / normal / full): always the full installation,
///   `g_mediaMode` 3 (owner: nobody installs another, and the game does not
///   read a CD); on to its answer 3's entry;
/// - the drive page on a system without drives (`install::folders`, `DRIVES` false): on
///   to the entry of its answer that is the folder page.
///
/// Come back to from where it went on to, a page goes back to where it was come to from.
/// None for a page that is shown.
fn pass_over(
    entry: PageEntry,
    prev: Option<PageEntry>,
    passed: &mut Vec<Passed>,
) -> Option<PageEntry> {
    use crate::install::folders::{Folders, Native};
    const MODE_PAGE: u32 = 0x41422B;
    const DRIVE_PAGE: u32 = 0x41425E;
    const FOLDER_PAGE: u32 = 0x4142F5;
    const FULL: u32 = 3;
    let on = if is_page(entry, MODE_PAGE) {
        crate::install::installer::set_media_mode(FULL);
        entry.next(pages::next_entry_at(FULL)).unwrap_or_else(|| {
            crate::install::installer::refuse(crate::errors::BadInput(
                "the installation-kind page with no page to go on to",
            ))
        })
    } else if !<Native as Folders>::DRIVES && is_page(entry, DRIVE_PAGE) {
        (0..8)
            .filter_map(|answer| entry.next(pages::next_entry_at(answer)))
            .find(|&next| is_page(next, FOLDER_PAGE))?
    } else {
        return None;
    };
    if let Some(p) = passed.iter().find(|p| p.entry == entry)
        && prev == Some(p.after)
    {
        return Some(p.before.unwrap_or_else(|| {
            crate::install::installer::refuse(crate::errors::BadInput(
                "a passed page that was come to from no page",
            ))
        }));
    }
    passed.retain(|p| p.entry != entry);
    passed.push(Passed {
        entry,
        before: prev,
        after: on,
    });
    Some(on)
}
