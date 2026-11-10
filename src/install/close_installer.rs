//! `install::close_installer` (0x414490): the installer's memory list (`stru_51BAFC`) freed
//! (`memory::free_list`) and its folder- and file-name table pointers (`dword_51BB40`,
//! `dword_51BB44`) zeroed.
//!
//! Caller `install::run_installer`.

use crate::errors::Refused;

pub fn close_installer(install: &'static crate::install::owner::Install) -> Result<(), Refused> {
    { crate::memory::free_list::free_list(&install.run.buffers) }?;
    install.run.dirs.set(0);
    install.run.files.set(0);
    crate::install::name_tables::FOLDERS.give_back();
    crate::install::name_tables::FILES.give_back();
    Ok(())
}
