//! `install::install_clio` (0x41675B): the Clio update copied from its disc. A frame is
//! shown; the two source base directories (`byte_95E8F8`, `byte_95E9F8`) are normalised into
//! `byte_95E6F8` and `byte_95E7F8` and the first made current; the Clio script
//! (`byte_529DCC`) is run (`install::run_script`); each listed folder is made
//! (`install::make_folders`, path built in `byte_95E9F8`) and each listed file copied
//! (`install::copy_counted`, source in `byte_95E8F8`, destination in `byte_95E9F8`), Escape
//! stopping the copy. A failed normalise, change or make ends it. Finally `g_installing`,
//! `dword_51B9CC` and `dword_51B9D0` are cleared after a second.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD48` for the Clio disc). Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    copy_picture, draw_string, lock, normalise, set_cd, show, sleep, store_text, take_key, text_at,
    unlock,
};

const FONT_BE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BOX_TEXT;

/// `to` = `base` + name `index` of `table`, NUL-ended.
fn append_both(
    base: &'static crate::session::Bytes<256>,
    to: &'static crate::session::Bytes<256>,
    table: &crate::memory::owned::Owned<crate::install::name_tables::NameTable>,
    index: u32,
) -> Result<(), Refused> {
    let mut path = text_at(base);
    crate::install::name_tables::push_name(table, index, &mut path)?;
    path.push(0);
    store_text(to, &path);
    Ok(())
}

/// Where the work stops: `loc_41689A` (the current directory set back first) or
/// `loc_4168A5` (not).
enum Stop {
    SetBack,
    Straight,
}

fn work(install: &'static crate::install::owner::Install) -> Result<Stop, Refused> {
    use crate::install as i;
    if !normalise(&install.paths.path_a, &install.paths.game_dir)?
        || !normalise(&install.paths.path_b, &install.paths.installed)?
        || !set_cd(&install.paths.game_dir)
    {
        return Ok(Stop::Straight);
    }
    i::run_script::run_script(install, &install.vars.clio_script)?;
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
            // 0x416886: a failed copy stops the install as Escape does.
            if !i::copy_counted::copy_counted(
                install,
                &install.paths.path_a,
                &install.paths.path_b,
            )? || take_key() == 1
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

pub fn install_clio(install: &'static crate::install::owner::Install) -> Result<(), Refused> {
    if lock() {
        copy_picture()?;
        draw_string(0xBE, FONT_BE, 0x1A9, 0x113)?;
        unlock();
        show();
        sleep(1000);
    }
    if let Stop::SetBack = work(install)? {
        set_cd(&install.paths.installed);
    }
    install.run.installing.set(0);
    install.run.updating.set(0);
    install.run.from_command.set(0);
    sleep(1000);
    Ok(())
}
