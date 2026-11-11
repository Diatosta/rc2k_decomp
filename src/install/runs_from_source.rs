//! `install::runs_from_source` (0x412DA3): whether the game was started from the
//! directory the registry names as its installation's source.
//!
//! `read_registry`; then, only if the status is 1 and a source is named and the current
//! directory can be had (`files::current_directory` into the numbered paths' buffer), the two are
//! compared with `strings::same_text`. Everything else is "no" (zero and carry clear).
//! Callers: the main loop `system::main_loop` on every start, and `install::check_clio_disc`.

use crate::errors::BadInput;
use crate::install::outside::Outside;
use std::cmp::Ordering;

/// The path buffer the checks build in (`files::numbered_path`'s, 0x100 bytes).
const SCRATCH_ROOM: usize = 0x100;

/// `text` written into the path buffer at `at`.
pub fn scratch_put(at: usize, text: &[u8]) -> Result<(), BadInput> {
    if at + text.len() > SCRATCH_ROOM {
        return Err(BadInput("a path past the path buffer"));
    }
    crate::files::state::PATHS.write_at(at, text, "the scratch path")
}

/// The path buffer's bytes.
pub fn scratch_bytes() -> Result<Vec<u8>, BadInput> {
    crate::files::state::PATHS.read_at(0, SCRATCH_ROOM, "the scratch path")
}

/// The path buffer's text (up to its NUL).
pub fn scratch_text() -> Result<Vec<u8>, BadInput> {
    let all = scratch_bytes()?;
    let n = all
        .iter()
        .position(|&c| c == 0)
        .ok_or(BadInput("the scratch path without its end"))?;
    Ok(all[..n].to_vec())
}

/// The registry's source (`InstallerNames.source`) against the current directory (written into the
/// path buffer, as the original): None when there is no installed source or no directory.
pub fn source_against_directory(
    install: &'static crate::install::owner::Install,
    outside: &mut dyn Outside,
) -> Result<Option<Ordering>, BadInput> {
    outside.read_registry();
    if install.names.status.get() != 1 || install.names.source.get(0) == 0 {
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

pub fn runs_from_source(
    install: &'static crate::install::owner::Install,
    outside: &mut dyn Outside,
) -> Result<bool, BadInput> {
    Ok(source_against_directory(install, outside)? == Some(Ordering::Equal))
}
