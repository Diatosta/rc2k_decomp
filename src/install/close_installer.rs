//! `install::close_installer` (0x414490): the installer's memory list (`stru_51BAFC`) freed
//! (`memory::free_list`) and its folder- and file-name table pointers (`dword_51BB40`,
//! `dword_51BB44`) zeroed.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`. The free through `part`. Shadow mode runs the original
//! alone (memory cannot be freed twice). Self-test: a copy whose one call goes to a
//! stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{BUFFERS, DIRS, FILES, FREE_LIST, Part, a, at, p};
use crate::race::viewer::regs_entry;

pub fn close_installer(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs {
        ebx: a(BUFFERS),
        ..*entry
    };
    part(at(FREE_LIST), &mut r)?;
    p(DIRS, 0);
    p(FILES, 0);
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        close_installer(r, &mut crate::install::installer::game_part)
    })
}
