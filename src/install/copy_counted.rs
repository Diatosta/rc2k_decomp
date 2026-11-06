//! `install::copy_counted` (0x4139DA): `dword_51BAE8` (the files copied so far) raised by one
//! and the file copied (`files::copy_file`, eax the source esi, ebx the destination edi).
//!
//! In:  esi the source path, edi the destination path
//! Out: every register kept (`pusha` / `popa`)
//! Callers `install::install_files`, `install::install_clio`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::copy_file;

/// `from` (esi) copied to `to` (edi); whether it was all done (the original leaves
/// `copy_file`'s carry through its `pusha` / `popa`: the game's files' copies and the
/// Clio's stop on it). Until 2026-10-23 the port dropped it, so a failed copy went unseen.
pub fn copy_counted(
    g: &'static crate::install::owner::Install,
    from: impl Into<crate::place::Place>,
    to: impl Into<crate::place::Place>,
) -> Result<bool, Refused> {
    let from: u32 = from.into().game();
    let to: u32 = to.into().game();
    g.run
        .files_copied
        .set(g.run.files_copied.get().wrapping_add(1));
    copy_file(from, to)
}
