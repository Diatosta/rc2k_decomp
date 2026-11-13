//! The Clio update copied from its disc: a frame is shown; the two source base directories are
//! normalised and the first made current; the Clio script is run; each listed folder is made and
//! each listed file copied (Escape stops the copy). A failed normalise, change or make ends it.
//! Finally `installing`, `updating` and `from_command` are cleared after a second. The action of
//! the third page's list item (for the Clio disc). Called by `install::run_page`.

use crate::errors::Refused;
use crate::install::installer::{
    copy_picture, draw_string, lock, normalise, set_cd, show, sleep, store_text, take_key, text_at,
    unlock,
};

const FONT_BE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BOX_TEXT;

/// `to` = `base` + name `index` of `table`, NUL-ended.
fn append_both(
    base: &'static crate::session::PlainBytes<256>,
    to: &'static crate::session::PlainBytes<256>,
    table: &crate::memory::owned::Owned<crate::install::name_tables::NameTable>,
    index: u32,
) -> Result<(), Refused> {
    let mut path = text_at(base);
    crate::install::name_tables::push_name(table, index, &mut path)?;
    path.push(0);
    store_text(to, &path);
    Ok(())
}

/// Where the work stops: with the current directory set back first, or not.
enum Stop {
    SetBack,
    Straight,
}

fn work(
    install: &'static crate::install::owner::Install,
    system: &crate::system::owner::System,
    input: &crate::input::owner::Input,
) -> Result<Stop, Refused> {
    use crate::install as i;
    if !normalise(&install.paths.path_a, &install.paths.game_dir)?
        || !normalise(&install.paths.path_b, &install.paths.installed)?
        || !set_cd(&install.paths.game_dir)
    {
        return Ok(Stop::Straight);
    }
    i::run_script::run_script(
        install,
        crate::place::BytePlace::from(&install.vars.clio_script),
    )?;
    if install.run.dir_count.get() != 0 {
        let mut n = 0;
        loop {
            append_both(
                &install.paths.installed,
                &install.paths.path_b,
                &crate::install::name_tables::FOLDERS,
                n,
            )?;
            if !i::make_folders::make_folders(&install.paths.path_b)? {
                return Ok(Stop::SetBack);
            }
            n = n.wrapping_add(1);
            if n == install.run.dir_count.get() {
                break;
            }
        }
        set_cd(&install.paths.game_dir);
    }
    if install.run.file_count.get() != 0 {
        let mut n = 0;
        loop {
            append_both(
                &install.paths.game_dir,
                &install.paths.path_a,
                &crate::install::name_tables::FILES,
                n,
            )?;
            append_both(
                &install.paths.installed,
                &install.paths.path_b,
                &crate::install::name_tables::FILES,
                n,
            )?;
            // A failed copy stops the install as Escape does.
            if !i::copy_counted::copy_counted(
                install,
                system,
                &install.paths.path_a,
                &install.paths.path_b,
            )? || take_key(input) == 1
            {
                return Ok(Stop::SetBack);
            }
            n = n.wrapping_add(1);
            if n == install.run.file_count.get() {
                break;
            }
        }
    }
    Ok(Stop::SetBack)
}

pub fn install_clio(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (system, gfx, input) = (o.system, o.gfx, o.input);
    if lock(gfx, system) {
        copy_picture(gfx, system)?;
        draw_string(
            crate::strings::append_string_by_id::StringOwners::of(o),
            0xBE,
            FONT_BE,
            0x1A9,
            0x113,
            gfx,
        )?;
        unlock(gfx);
        show(o);
        sleep(1000);
    }
    if let Stop::SetBack = work(install, system, input)? {
        set_cd(&install.paths.installed);
    }
    install.run.installing.set(0);
    install.run.updating.set(0);
    install.run.from_command.set(0);
    sleep(1000);
    Ok(())
}
