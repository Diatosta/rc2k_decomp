//! The installer's branch of the main loop: the languages read, the fonts built, the installer
//! opened, base directory B normalised (failing: fatal error 0x80AF8101), the drives listed, a
//! default folder built, the installation's record read, and the installed version checked (this
//! build: its record read and its language loaded). Then the pages run (`install::run_pages`); if
//! installing from a CD a "please wait" frame is shown. Finally the installer is closed and the
//! fonts freed.

use crate::errors::Refused;
use crate::install::installer::{
    copy_picture, draw_string, fatal, load_language, lock, normalise, pause, push_string_by_id,
    set_cd, show, store_text, unlock,
};
use crate::memory::place::BytePlace;

const FONT_BE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BOX_TEXT;

pub fn run_installer(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (gfx, system) = (o.gfx, o.system);
    use crate::install as i;
    crate::shell::events::keep_running_after_close();
    crate::files::state::set_directory_byte(0, 0);
    i::read_languages::load_languages(install, system, &install.names.lang_cfg.text()[..])?;
    crate::gfx::build_fonts::build_fonts_in(gfx, system, o.strings)?;
    i::open_installer::open_installer(install, o)?;
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
    i::record::read_installed(install);
    if install.names.source.get(0) != 0 {
        let n = install.names.source_size.get() as usize;
        let source = BytePlace::from(&install.names.source).read(n, "the source folder")?;
        BytePlace::from(&install.paths.typed).write(&source, "the folder typed")?;
    }
    show_folder(install);
    i::check_installed_version::check_installed_version(install, system)?;
    // The version-match branch changes into the installed folder.
    if install.run.same_version.get() == 1 {
        install
            .highlights
            .set(rc2k_core::install::model::List::Main, 1);
        set_cd(&install.paths.installed);
        i::read_record::load_record(system, &install.texts.record_path.text()[..])?;
        crate::files::state::set_directory_byte(0, 0);
        set_cd(&install.paths.game_dir);
        load_language(o)?;
    }
    let first = rc2k_core::install::flow::first_step(install.flags.updating.get());
    i::run_pages::run_pages(install, o, first)?;
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
pub(super) fn put_typed(install: &crate::install::owner::Install, text: &[u8]) {
    let typed = &install.paths.typed;
    let n = text.len().min(255);
    for (k, &b) in text[..n].iter().enumerate() {
        typed.set(k, b);
    }
    typed.set(n, 0);
}
