//! `install::uninstall` (0x4134DF): the installation removed. Only when installed
//! (`g_installStatus` != 0) with a source. The status is marked busy and written
//! (`install::write_registry`); a "removing" frame is shown; the source is made current and
//! the install script run (`install::run_script`) to rebuild the file and folder lists; each
//! listed file is deleted (`files::delete_file`) and each folder removed in reverse
//! (`sub_403123`); the parent is entered and the install folder removed; after a second the
//! status and source are cleared, written again, the DirectPlay key removed
//! (`install::unregister_lobby`), the Start-menu removal asked (`g_removeStartItems`) and
//! `g_installing` cleared. Each path is built in `byte_95E9F8` from `byte_95E7F8` plus the
//! listed name.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD30`). Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.
//!
//! "Remove all" (2026-11-03, a decided difference): the original removed only what its
//! script lists, so files the game made (its log, `rc2k.log`, the program and its
//! libraries) kept the folder. With every folder chosen (`script_mark` not 5, "keep saved
//! data"'s End) the installation's folder goes whole once the listed files are gone; on
//! Windows what is still in use goes after the program ends (`os::remove_folder_after_exit`).
//! Only a folder that holds an installation's record (`var\install.cfg`) and is neither a
//! root nor the home folder.

use crate::errors::Refused;
use crate::install::installer::{
    append_byte, copy_picture, delete_file, lock, normalise, remove_dir, run_frame, set_cd, show,
    sleep, str_append, unlock, wb,
};
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PAGES;
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;

/// byte_95E9F8 = byte_95E7F8 + name #index of `table`, NUL-ended.
fn build_path(
    table: &crate::memory::owned::Owned<crate::install::name_tables::NameTable>,
    index: u32,
) -> Result<crate::place::Place, Refused> {
    let cursor = str_append(&INSTALLER_PATHS.installed, &INSTALLER_PATHS.path_b);
    let cursor = crate::install::name_tables::append_name(table, index, cursor)?;
    append_byte(cursor, 0);
    Ok(crate::place::Place::from(&INSTALLER_PATHS.path_b))
}

/// The installation's folder, when "remove all" was chosen and it is one that may go whole.
fn whole_folder() -> Option<std::path::PathBuf> {
    if INSTALLER_NAMES.script_mark.get() == 5 {
        return None;
    }
    let text = INSTALLER_PATHS.installed.read();
    let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
    let mut record = text.to_vec();
    if !record.ends_with(b"\\") {
        record.push(b'\\');
    }
    record.extend(crate::install::portable::FILE.as_bytes());
    let folder = std::path::absolute(crate::files::host_path(text)).ok()?;
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .and_then(|h| std::path::absolute(h).ok());
    let kept = folder.parent().is_none()
        || home.is_some_and(|h| h == folder)
        || !crate::files::host_path(&record).is_file();
    if kept {
        crate::framework::log(&format!(
            "install: \"{}\" is not removed whole (not an installation's folder)",
            folder.display()
        ));
        return None;
    }
    Some(folder)
}

/// What is left of `folder` removed (now, and on Windows what is in use after the end).
fn remove_whole(folder: &std::path::Path) {
    let now = match std::fs::remove_dir_all(folder) {
        Ok(()) => "removed".to_string(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => "already gone".to_string(),
        Err(e) => format!("not all removed ({e})"),
    };
    #[cfg(windows)]
    let now = if folder.exists() {
        let later = crate::os::remove_folder_after_exit(folder);
        format!(
            "{now}; the rest after the end: {}",
            if later { "asked" } else { "not asked" }
        )
    } else {
        now
    };
    crate::framework::log(&format!("install: \"{}\": {now}", folder.display()));
}

pub fn uninstall() -> Result<(), Refused> {
    use crate::install as i;
    if INSTALLER_NAMES.status.get() == 0 || INSTALLER_NAMES.source.get(0) == 0 {
        return Ok(());
    }
    INSTALLER_NAMES.status.set(0x80);
    i::write_registry::write_registry(&mut crate::install::portable::Portable)?;
    // the "removing" frame
    run_frame(None)?;
    if lock() {
        copy_picture()?;
        i::draw_string_list::draw_string_list(1, &INSTALLER_PAGES.remove_strings, 0, 0)?;
        unlock();
        show();
    }
    normalise(&INSTALLER_NAMES.source, &INSTALLER_PATHS.installed)?;
    let whole = whole_folder();
    set_cd(&INSTALLER_PATHS.installed);
    i::run_script::run_script(&INSTALLER_NAMES.script)?;
    if INSTALLER_RUN.file_count.get() != 0 {
        let mut n = 0;
        loop {
            delete_file(build_path(&crate::install::name_tables::FILES, n)?);
            n = n.wrapping_add(1);
            if n == INSTALLER_RUN.file_count.get() {
                break;
            }
        }
    }
    if INSTALLER_RUN.dir_count.get() != 0 {
        let mut n = INSTALLER_RUN.dir_count.get().wrapping_sub(1);
        loop {
            remove_dir(build_path(&crate::install::name_tables::FOLDERS, n)?);
            n = n.wrapping_sub(1);
            if (n as i32) < 0 {
                break;
            }
        }
    }
    set_cd(&crate::install::installer::INSTALL_TEXTS.dotdot);
    remove_dir(&INSTALLER_PATHS.installed);
    if let Some(folder) = whole {
        remove_whole(&folder);
    }
    sleep(1000);
    INSTALLER_NAMES.status.set(0);
    wb(&INSTALLER_NAMES.source, 0);
    i::write_registry::write_registry(&mut crate::install::portable::Portable)?;
    i::unregister_lobby::unregister_lobby(&mut crate::install::portable::Portable);
    INSTALLER_RUN.remove_items.set(1);
    INSTALLER_RUN.installing.set(0);
    Ok(())
}
