//! `install::copy_counted` (0x4139DA): `dword_51BAE8` (the files copied so far) raised by one
//! and the file copied (`files::copy_file`, eax the source esi, ebx the destination edi).
//!
//! In:  esi the source path, edi the destination path
//! Out: every register kept (`pusha` / `popa`)
//! Callers `install::install_files`, `install::install_clio`. The copy through `part`.
//! Ungated. Shadow mode runs the original alone (one copy). Self-test: a copy whose one
//! call goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{COPY_FILE, FILES_COPIED, Part, at, g, p};
use crate::race::viewer::regs_entry;

pub fn copy_counted(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    p(FILES_COPIED, g(FILES_COPIED).wrapping_add(1));
    let mut r = Regs {
        eax: entry.esi,
        ebx: entry.edi,
        ..*entry
    };
    part(at(COPY_FILE), &mut r)?;
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        copy_counted(r, &mut crate::install::installer::game_part)
    })
}
