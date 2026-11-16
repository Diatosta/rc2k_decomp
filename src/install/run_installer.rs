//! The installer's branch of the main loop: the languages read, the fonts built, the installer
//! opened. With the `-i` paths given, `install::install_from_command` does it all; otherwise base
//! directory B is normalised (failing: fatal error 0x80AF8101), the drives listed, a default
//! folder built, the registry read, and the installed version checked (this build: its record
//! read and its language loaded). Then the pages run
//! (`install::run_pages`); if installing from a CD a "please wait" frame is shown. Finally the
//! installer is closed and the fonts freed. Called by `system::main_loop`.

use crate::errors::Refused;
use crate::install::installer::{
    copy_picture, draw_string, fatal, load_language, lock, normalise, pause, push_string_by_id, rb,
    set_cd, show, store_text, unlock, wb,
};
use crate::memory::place::BytePlace;

const FONT_BE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BOX_TEXT;

pub fn run_installer(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (gfx, system) = (o.gfx, o.system);
    use crate::install as i;
    crate::platform::events::keep_running_after_close();
    // The folder page's edit item shows the folder from its third byte, past the drive's `c:`;
    // a Unix path has no drive, so the item shows it whole.
    #[cfg(unix)]
    install.pages.folder_menu.items[1]
        .target
        // A link to `InstallerPaths::typed` from its first byte.
        .set(crate::install::page_links::PageLink::typed(0).raw());

    crate::files::state::set_directory_byte(0, 0);
    i::read_languages::load_languages(install, system, &install.names.lang_cfg.text()[..])?;
    crate::gfx::build_fonts::build_fonts_in(gfx, system, o.strings)?;
    i::open_installer::open_installer(install, o)?;
    // From the command line: the install, and on past the pages.
    let from_command = install.flags.from_command.get();
    if from_command {
        i::install_from_command::install_from_command(install, o)?;
    } else {
        if !normalise(&crate::files::state::SECOND_BASE, &install.paths.game_dir)? {
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
            i::read_record::load_record(system, &install.texts.record_path.text()[..])?;
            crate::files::state::set_directory_byte(0, 0);
            set_cd(&install.paths.game_dir);
            load_language(o)?;
        }
        let pages = if install.flags.updating.get() {
            crate::install::page_records::PageEntry::update_run()
        } else {
            crate::install::page_records::PageEntry::normal_run()
        };
        i::run_pages::run_pages(install, o, pages)?;
    }
    if install.flags.installing.get()
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
        pause(o, 2000);
    }
    i::close_installer::close_installer(install, gfx)?;
    crate::gfx::free_fonts::free_fonts(&gfx.fonts)?;
    Ok(())
}

/// The folder page's text by the system's rules (`install::folders`): a recorded folder as
/// shown, else the system's first folder.
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
