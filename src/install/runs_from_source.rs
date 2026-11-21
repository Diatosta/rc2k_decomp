//! The installation's recorded source folder against the current directory (only when the record
//! says it is installed and names a source), and the path buffer the start checks build in.

use crate::errors::BadInput;
use crate::install::outside::Outside;
use std::cmp::Ordering;

/// The path buffer the checks build in (`files::numbered_path`'s, 0x100 bytes).
const SCRATCH_ROOM: usize = 0x100;

/// `text` written into the path buffer at `at`.
pub(super) fn scratch_put(at: usize, text: &[u8]) -> Result<(), BadInput> {
    if at + text.len() > SCRATCH_ROOM {
        return Err(BadInput("a path past the path buffer"));
    }
    crate::files::state::PATHS.write_at(at, text, "the scratch path")
}

pub(super) fn scratch_bytes() -> Result<Vec<u8>, BadInput> {
    crate::files::state::PATHS.read_at(0, SCRATCH_ROOM, "the scratch path")
}

/// The path buffer's text (up to its NUL).
pub(super) fn scratch_text() -> Result<Vec<u8>, BadInput> {
    let all = scratch_bytes()?;
    let n = all
        .iter()
        .position(|&c| c == 0)
        .ok_or(BadInput("the scratch path without its end"))?;
    Ok(all[..n].to_vec())
}

/// The record's source (`InstallerNames::source`) against the current directory (written into the
/// path buffer): None when there is no installed source or no directory.
pub fn source_against_directory(
    install: &crate::install::owner::Install,
    outside: &mut dyn Outside,
) -> Result<Option<Ordering>, BadInput> {
    outside.read_installed();
    if install.names.status.get() != 1 || install.names.source.is_empty() {
        return Ok(None);
    }
    let Some(directory) = outside.current_directory()? else {
        return Ok(None);
    };
    scratch_put(0, &directory)?;
    Ok(Some(crate::strings::same_text::compare_bytes(
        &install.names.source.text(),
        &directory,
    )))
}
