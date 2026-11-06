//! `install::close_installer` (0x414490): the installer's memory list (`stru_51BAFC`) freed
//! (`memory::free_list`) and its folder- and file-name table pointers (`dword_51BB40`,
//! `dword_51BB44`) zeroed.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;

pub fn close_installer(g: &'static crate::install::owner::Install) -> Result<(), Refused> {
    { crate::memory::free_list::free_list(crate::pointers::GamePtr::to(&g.run.buffers)) }?;
    g.run.dirs.set(0);
    g.run.files.set(0);
    crate::install::name_tables::FOLDERS.give_back();
    crate::install::name_tables::FILES.give_back();
    Ok(())
}
