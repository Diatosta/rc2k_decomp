//! One folder path added to the folders-to-make table: resolved, appended with a NUL, the cursor
//! advanced and `dir_count` raised. More than 0x10000 bytes of names is fatal error 0x80AF2205. The
//! script's opcode 1.

use crate::errors::Refused;
use crate::install::installer::{fatal, resolve_path};
use rc2k_core::install::script;

/// The folder `path` added to the folders' table.
pub fn list_folder(
    install: &crate::install::owner::Install,
    path: impl Into<crate::memory::place::BytePlace>,
) -> Result<(), Refused> {
    let path = resolve_path(path)?;
    let (used, scan) = install
        .folders
        .with(|t| (t.push(path), t.names.len() as u32))?;
    install.run.dir_cursor.set(used);
    install.run.dir_scan.set(scan);
    install
        .run
        .dir_count
        .set(install.run.dir_count.get().wrapping_add(1));
    script::room_left(used, script::FOLDERS_ROOM, script::FOLDERS_FULL).map_err(fatal)?;
    Ok(())
}
