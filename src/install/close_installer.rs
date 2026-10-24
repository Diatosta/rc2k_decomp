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
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{BUFFERS, DIRS, FILES, a, p};

pub fn close_installer() -> Result<(), Refused> {
    unsafe { crate::memory::free_list::free_list(a(BUFFERS) as *mut _) }?;
    p(DIRS, 0);
    p(FILES, 0);
    Ok(())
}
