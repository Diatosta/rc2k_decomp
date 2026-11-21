//! The installation removed. Only when installed with a source: the status is marked busy and
//! written; a "removing" frame is shown; the source is made current and the install script run to
//! rebuild the file and folder lists; each listed file is deleted and each folder removed in
//! reverse; the parent is entered and the install folder removed; after a second the status and
//! source are cleared and written again, the DirectPlay key removed, the Start menu removal asked
//! and the installing flag cleared.
//!
//! "Remove all" (a decided difference: removing only what the script lists would leave the
//! files the game made, and so the folder): with every folder chosen (`script_mark` not 5) the
//! installation's folder goes whole once the listed files are gone; on Windows what is still in
//! use goes after the program ends (`os::remove_folder_after_exit`). Only a folder that holds an
//! installation's record (`var\install.cfg`) and is neither a root nor the home folder.

use crate::errors::Refused;
use crate::install::installer::{
    copy_picture, delete_file, lock, normalise, pause, remove_dir, run_frame, set_cd, show, unlock,
};
use crate::install::vars::fixed;

/// `paths.installed` + name #index of `table`, kept as `path_b`.
fn build_path(
    install: &'static crate::install::owner::Install,
    table: &crate::memory::owned::Owned<crate::install::name_tables::NameTable>,
    index: u32,
) -> Result<Vec<u8>, Refused> {
    let mut path = install.paths.installed.text();
    crate::install::name_tables::push_name(table, index, &mut path)?;
    install.paths.path_b.set(&path)?;
    Ok(install.paths.path_b.text())
}

fn whole_folder(install: &crate::install::owner::Install) -> Option<std::path::PathBuf> {
    if install.names.script_mark.get() == fixed::MARK_KEEP_SAVES {
        return None;
    }
    let text = &install.paths.installed.text();
    let mut record = text.to_vec();
    if !record.ends_with(b"\\") {
        record.push(b'\\');
    }
    record.extend(crate::install::record::FILE.as_bytes());
    let folder = std::path::absolute(crate::files::host_path(text)).ok()?;
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .and_then(|h| std::path::absolute(h).ok());
    let kept = folder.parent().is_none()
        || home.is_some_and(|h| h == folder)
        || !crate::files::host_path(&record).is_file();
    if kept {
        crate::program_log::log(&format!(
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
        let later = crate::platform::os::remove_folder_after_exit(folder);
        format!(
            "{now}; the rest after the end: {}",
            if later { "asked" } else { "not asked" }
        )
    } else {
        now
    };
    crate::program_log::log(&format!("install: \"{}\": {now}", folder.display()));
}

pub fn uninstall(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (system, gfx) = (o.system, o.gfx);
    use crate::install as i;
    if install.names.status.get() == 0 || install.names.source.is_empty() {
        return Ok(());
    }
    install.names.status.set(0x80);
    i::record::write_installed(install);
    // the "removing" frame
    run_frame(o, None)?;
    if lock(gfx, system) {
        copy_picture(gfx, system)?;
        i::draw_string_list::draw_string_list(
            crate::strings::append_string_by_id::StringOwners::of(o),
            &install.pages.remove_strings.each_ref(),
            0,
            0,
            gfx,
        )?;
        unlock(gfx);
        show(o);
    }
    if let Some(installed) = normalise(&install.names.source.text())? {
        install.paths.installed.set(&installed)?;
    }
    let whole = whole_folder(install);
    set_cd(&install.paths.installed.text());
    i::run_script::run_script(install, &install.names.uninstall_script())?;
    if install.run.file_count.get() != 0 {
        let mut n = 0;
        loop {
            delete_file(&build_path(install, &install.files, n)?);
            n = n.wrapping_add(1);
            if n == install.run.file_count.get() {
                break;
            }
        }
    }
    if install.run.dir_count.get() != 0 {
        let mut n = install.run.dir_count.get().wrapping_sub(1);
        loop {
            remove_dir(&build_path(install, &install.folders, n)?);
            n = n.wrapping_sub(1);
            if (n as i32) < 0 {
                break;
            }
        }
    }
    set_cd(&install.texts.dotdot.text());
    remove_dir(&install.paths.installed.text());
    if let Some(folder) = whole {
        remove_whole(&folder);
    }
    pause(o, 1000);
    install.names.status.set(0);
    install.names.source.set(&[])?;
    i::record::write_installed(install);
    install.run.remove_items.set(true);
    install.flags.installing.set(false);
    Ok(())
}
