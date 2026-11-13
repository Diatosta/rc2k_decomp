//! `install::run_installer` (0x412DE0): the installer's branch of the main loop. The base
//! directory is cleared, the languages read (`install::read_languages`), the fonts built
//! (`gfx::build_fonts`) and the installer opened (`install::open_installer`). With the `-i`
//! paths given (`run.from_command`), `install::install_from_command` does it all; otherwise base
//! directory B is normalised (failing: fatal error 0x80AF8101), the drives listed
//! (`install::list_drives`), a default folder built in `InstallerPaths::typed` (strings 0x9F, 0xA0, the
//! first drive letter when `g_keyboardType` is 2), the registry read, the recorded source
//! copied over it, and the installed version checked (`install::check_installed_version`); if
//! it is this build, its record is read and language loaded. Then the pages run
//! (`install::run_pages` with `update_pages` for an update, else `normal_pages`); if installing
//! from a CD a "please wait" frame is shown. Finally the installer is closed
//! (`install::close_installer`) and the fonts freed (`gfx::free_fonts`).
//!
//! Caller `system::main_loop` (called directly).

use crate::errors::Refused;
use crate::install::installer::{
    copy_picture, draw_string, fatal, load_language, lock, normalise, push_string_by_id, rb,
    set_cd, show, sleep, store_text, unlock, wb,
};
use crate::place::BytePlace;

const FONT_BE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BOX_TEXT;

pub fn run_installer(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (gfx, system) = (o.gfx, o.system);
    use crate::install as i;
    crate::platform_events::keep_running_after_close();
    // The folder page's edit item (0x51DBC8) shows the folder from its third byte
    // (0x95EBFA, past the drive's `c:`); a Unix path has no drive, so the item shows it
    // whole.
    #[cfg(unix)]
    install.pages.folder_menu.items[1]
        .target
        // A link to `InstallerPaths::typed` from its first byte.
        .set(crate::install::page_links::PageLink::typed(0).raw());

    crate::files::state::set_directory_byte(0, 0);
    i::read_languages::read_languages(install, system, &install.names.lang_cfg.text()[..])?;
    crate::gfx::build_fonts::build_fonts_in(gfx, system, o.strings)?;
    i::open_installer::open_installer(install, o)?;
    // From the command line: the install, and on past the pages (`jmp loc_412EDC`).
    let from_command = install.run.from_command.get() == 1;
    if from_command {
        i::install_from_command::install_from_command(install, o)?;
    } else {
        if !normalise(&install.texts.base_b, &install.paths.game_dir)? {
            return Err(fatal(0x80AF_8101));
        }
        i::list_drives::list_drives(install, o);
        let mut typed = Vec::new();
        push_string_by_id(
            crate::strings::append_string_by_id::StringOwners::of(o),
            &mut typed,
            0x9F,
        );
        push_string_by_id(
            crate::strings::append_string_by_id::StringOwners::of(o),
            &mut typed,
            0xA0,
        );
        typed.push(0);
        store_text(&install.paths.typed, &typed);
        if system.machine().keyboard_type.get() == 2 {
            install
                .paths
                .typed
                .set(0, install.run.first_drive.get() as u8);
        }
        i::read_registry::read_registry_here(install);
        if install.names.source.get(0) != 0 {
            let n = install.names.source_size.get();
            for k in 0..n {
                wb(
                    BytePlace::from(&install.paths.typed).add(k),
                    rb(BytePlace::from(&install.names.source).add(k)),
                );
            }
        }
        show_folder(install);
        i::check_installed_version::check_installed_version(install, system)?;
        // The version-match branch changes into the installed folder.
        if install.run.same_version.get() == 1 {
            install.pages.main_menu.header.highlight.set(1);
            set_cd(&install.paths.installed);
            i::read_record::read_record(system, &install.texts.record_path.text()[..])?;
            crate::files::state::set_directory_byte(0, 0);
            set_cd(&install.paths.game_dir);
            load_language(o)?;
        }
        let pages = if install.run.updating.get() == 1 {
            crate::install::page_records::PageEntry::update_run()
        } else {
            crate::install::page_records::PageEntry::normal_run()
        };
        i::run_pages::run_pages(install, o, pages)?;
    }
    if install.run.installing.get() == 1
        && install.vars.running_from_cd.get() == 1
        && lock(o.gfx, o.system)
    {
        copy_picture(o.gfx, o.system)?;
        draw_string(
            crate::strings::append_string_by_id::StringOwners::of(o),
            0xBE,
            FONT_BE,
            0x1A9,
            0x113,
            o.gfx,
        )?;
        unlock(o.gfx);
        show(o);
        sleep(2000);
    }
    i::close_installer::close_installer(install)?;
    crate::gfx::free_fonts::free_fonts_in_game(gfx)?;
    Ok(())
}

/// The folder page's text by the system's rules (`install::folders`): a recorded folder as
/// shown, else the system's first folder (on Windows both the original's, unchanged).
fn show_folder(install: &crate::install::owner::Install) {
    use crate::install::folders::{Folders, Native};
    let typed = &install.paths.typed;
    let len = (0..256).position(|k| typed.get(k) == 0).unwrap_or(0);
    let now: Vec<u8> = (0..len).map(|k| typed.get(k)).collect();
    let recorded = install.names.source.get(0) != 0;
    let text = match (recorded, <Native as Folders>::first_folder()) {
        (true, _) => <Native as Folders>::shown(&now),
        (false, Some(first)) => first,
        (false, None) => return,
    };
    put_typed(install, &text);
}

/// `text` as the folder page's text (cut to its place, NUL-ended).
pub fn put_typed(install: &crate::install::owner::Install, text: &[u8]) {
    let typed = &install.paths.typed;
    let n = text.len().min(255);
    for (k, &b) in text[..n].iter().enumerate() {
        typed.set(k, b);
    }
    typed.set(n, 0);
}
