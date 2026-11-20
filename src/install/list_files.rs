//! The files matching a pattern added to the files-to-copy table: each found name is appended with
//! a NUL behind its folder, the cursor advanced and `file_count` raised. More than 0x40000 bytes of
//! names is fatal error 0x80AF2206. The script's opcode 3.

use crate::errors::Refused;
use crate::files::search::DIRECTORY;
use crate::install::installer::{close_search, fatal, find_first, find_next, resolve_path};
use crate::install::name_tables::found_name;
use rc2k_core::install::script;

/// FindFileData.cFileName (the search record: `files::search`), up to its NUL.
fn find_name() -> Vec<u8> {
    crate::files::search::found_name()
}

/// The files of `pattern` added to the files' table.
pub fn list_files(install: &crate::install::owner::Install, pattern: &[u8]) -> Result<(), Refused> {
    let pattern = resolve_path(pattern)?;
    if find_first(&pattern, DIRECTORY) {
        loop {
            // `append_directory` of the pattern's path, at the cursor.
            let name = found_name(&pattern, &find_name())?;
            let used = install.files.with(|t| t.push(name))?;
            install.run.file_cursor.set(used);
            install
                .run
                .file_count
                .set(install.run.file_count.get().wrapping_add(1));
            script::room_left(used, script::FILES_ROOM, script::FILES_FULL).map_err(fatal)?;
            if !find_next(DIRECTORY) {
                break;
            }
        }
    }
    close_search();
    Ok(())
}
