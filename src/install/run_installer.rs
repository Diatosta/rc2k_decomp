//! `install::run_installer` (0x412DE0): the installer's branch of the main loop. The base
//! directory is cleared, the languages read (`install::read_languages`), the fonts built
//! (`gfx::build_fonts`) and the installer opened (`install::open_installer`). With the `-i`
//! paths given (`dword_51B9D0`), `install::install_from_command` does it all; otherwise base
//! directory B is normalised (failing: fatal error 0x80AF8101), the drives listed
//! (`install::list_drives`), a default folder built in `byte_95EBF8` (strings 0x9F, 0xA0, the
//! first drive letter when `g_keyboardType` is 2), the registry read, the recorded source
//! copied over it, and the installed version checked (`install::check_installed_version`); if
//! it is this build, its record is read and language loaded. Then the pages run
//! (`install::run_pages` with `off_51D5D8` for an update, else `off_51D594`); if installing
//! from a CD a "please wait" frame is shown. Finally the installer is closed
//! (`install::close_installer`) and the fonts freed (`gfx::free_fonts`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `system::main_loop` (the Rust version calls this directly).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    append_byte, append_string_by_id, copy_picture, draw_string, fatal, load_language, lock,
    normalise, rb, set_cd, show, sleep, unlock, wb,
};
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PAGES;
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;
use crate::place::Place;
use crate::session::MACHINE;

const FONT_BE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BOX_TEXT;

pub fn run_installer() -> Result<(), Refused> {
    use crate::install as i;
    crate::platform_events::keep_running_after_close();
    // The folder page's edit item (0x51DBC8) shows the folder from its third byte
    // (0x95EBFA, past the drive's `c:`); a Unix path has no drive, so the item shows it
    // whole (2026-10-28: it showed `ome/..`).
    #[cfg(unix)]
    crate::install::installer::wd(
        // 0x51DBD0.
        crate::place::Place::from(&crate::install::vars::INSTALLER_PAGES.folder_page).add(0x30),
        // A link: `INSTALLER_PATHS.typed`'s IDA address (`InstallerPages::resolved`).
        0x95EBF8,
    );
    crate::install::installer::wb(crate::files::state::base_directory_place(), 0);
    i::read_languages::read_languages(crate::pointers::GamePtr::to(&INSTALLER_NAMES.lang_cfg))?;
    { crate::gfx::build_fonts::build_fonts() }?;
    i::open_installer::open_installer()?;
    // From the command line: the install, and on past the pages (`jmp loc_412EDC`).
    let from_command = INSTALLER_RUN.from_command.get() == 1;
    if from_command {
        i::install_from_command::install_from_command()?;
    } else {
        if !normalise(
            &crate::install::installer::INSTALL_TEXTS.base_b,
            &INSTALLER_PATHS.game_dir,
        )? {
            return Err(fatal(0x80AF_8101));
        }
        i::list_drives::list_drives();
        let cursor = append_string_by_id(0x9F, &INSTALLER_PATHS.typed);
        let cursor = append_string_by_id(0xA0, cursor);
        append_byte(cursor, 0);
        if MACHINE.keyboard_type.get() == 2 {
            INSTALLER_PATHS
                .typed
                .set(0, INSTALLER_RUN.first_drive.get() as u8);
        }
        i::read_registry::read_registry_here();
        if INSTALLER_NAMES.source.get(0) != 0 {
            let n = INSTALLER_NAMES.source_size.get();
            for k in 0..n {
                wb(
                    Place::from(&INSTALLER_PATHS.typed).add(k),
                    rb(Place::from(&INSTALLER_NAMES.source).add(k)),
                );
            }
        }
        show_folder();
        i::check_installed_version::check_installed_version()?;
        // The version-match branch changes into the installed folder.
        if INSTALLER_RUN.same_version.get() == 1 {
            INSTALLER_PAGES.record_flag.set(1);
            set_cd(&INSTALLER_PATHS.installed);
            i::read_record::read_record(crate::pointers::GamePtr::to(
                &crate::install::installer::INSTALL_TEXTS.record_path,
            ))?;
            crate::install::installer::wb(crate::files::state::base_directory_place(), 0);
            set_cd(&INSTALLER_PATHS.game_dir);
            load_language()?;
        }
        let pages = if INSTALLER_RUN.updating.get() == 1 {
            crate::install::page_records::PageEntry::of(&INSTALLER_PAGES.update_pages)
        } else {
            crate::install::page_records::PageEntry::of(&INSTALLER_PAGES.normal_pages)
        };
        i::run_pages::run_pages(pages)?;
    }
    if INSTALLER_RUN.installing.get() == 1
        && crate::install::store::INSTALL.running_from_cd.get() == 1
        && lock()
    {
        copy_picture()?;
        draw_string(0xBE, FONT_BE.game_ptr(), 0x1A9, 0x113)?;
        unlock();
        show();
        sleep(2000);
    }
    i::close_installer::close_installer()?;
    crate::gfx::free_fonts::free_fonts_in_game()?;
    Ok(())
}

/// The folder page's text by the system's rules (`install::folders`): a recorded folder as
/// shown, else the system's first folder (on Windows both the original's, unchanged).
fn show_folder() {
    use crate::install::folders::{Folders, Native};
    let typed = &INSTALLER_PATHS.typed;
    let len = (0..256).position(|k| typed.get(k) == 0).unwrap_or(0);
    let now: Vec<u8> = (0..len).map(|k| typed.get(k)).collect();
    let recorded = INSTALLER_NAMES.source.get(0) != 0;
    let text = match (recorded, <Native as Folders>::first_folder()) {
        (true, _) => <Native as Folders>::shown(&now),
        (false, Some(first)) => first,
        (false, None) => return,
    };
    put_typed(&text);
}

/// `text` as the folder page's text (cut to its place, NUL-ended).
pub fn put_typed(text: &[u8]) {
    let typed = &INSTALLER_PATHS.typed;
    let n = text.len().min(255);
    for (k, &b) in text[..n].iter().enumerate() {
        typed.set(k, b);
    }
    typed.set(n, 0);
}
