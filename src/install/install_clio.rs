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
    append_byte, copy_picture, draw_string, lock, normalise, set_cd, show, sleep, str_append,
    str_append_indexed, take_key, unlock,
};
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;

const FONT_BE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BOX_TEXT;

/// `to` = `base` + name `index` of `table`, NUL-ended.
fn append_both(base: u32, to: u32, table: u32, index: u32) {
    let cursor = str_append(base, to);
    let cursor = str_append_indexed(table, index, cursor);
    append_byte(cursor, 0);
}

/// Where the work stops: `loc_41689A` (the current directory set back first) or
/// `loc_4168A5` (not).
enum Stop {
    SetBack,
    Straight,
}

fn work() -> Result<Stop, Refused> {
    use crate::install as i;
    if !normalise(
        INSTALLER_PATHS.path_a.addr() as u32,
        INSTALLER_PATHS.game_dir.addr() as u32,
    )? || !normalise(
        INSTALLER_PATHS.path_b.addr() as u32,
        INSTALLER_PATHS.installed.addr() as u32,
    )? || !set_cd(INSTALLER_PATHS.game_dir.addr() as u32)
    {
        return Ok(Stop::Straight);
    }
    i::run_script::run_script(crate::install::store::INSTALL.clio_script.addr() as u32)?;
    if INSTALLER_RUN.dir_count.get() != 0 {
        let mut n = 0;
        loop {
            append_both(
                INSTALLER_PATHS.installed.addr() as u32,
                INSTALLER_PATHS.path_b.addr() as u32,
                INSTALLER_RUN.dirs.get(),
                n,
            );
            if !i::make_folders::make_folders(INSTALLER_PATHS.path_b.addr() as u32)? {
                return Ok(Stop::SetBack);
            }
            n = n.wrapping_add(1);
            if n == INSTALLER_RUN.dir_count.get() {
                break;
            }
        }
        set_cd(INSTALLER_PATHS.game_dir.addr() as u32);
    }
    if INSTALLER_RUN.file_count.get() != 0 {
        let mut n = 0;
        loop {
            append_both(
                INSTALLER_PATHS.game_dir.addr() as u32,
                INSTALLER_PATHS.path_a.addr() as u32,
                INSTALLER_RUN.files.get(),
                n,
            );
            append_both(
                INSTALLER_PATHS.installed.addr() as u32,
                INSTALLER_PATHS.path_b.addr() as u32,
                INSTALLER_RUN.files.get(),
                n,
            );
            // 0x416886: a failed copy stops the install as Escape does.
            if !i::copy_counted::copy_counted(
                INSTALLER_PATHS.path_a.addr() as u32,
                INSTALLER_PATHS.path_b.addr() as u32,
            )? || take_key() == 1
            {
                return Ok(Stop::SetBack);
            }
            n = n.wrapping_add(1);
            if n == INSTALLER_RUN.file_count.get() {
                break;
            }
        }
    }
    Ok(Stop::SetBack)
}

pub fn install_clio() -> Result<(), Refused> {
    if lock() {
        copy_picture()?;
        draw_string(0xBE, FONT_BE.addr() as u32, 0x1A9, 0x113)?;
        unlock();
        show();
        sleep(1000);
    }
    if let Stop::SetBack = work()? {
        set_cd(INSTALLER_PATHS.installed.addr() as u32);
    }
    INSTALLER_RUN.installing.set(0);
    INSTALLER_RUN.updating.set(0);
    INSTALLER_RUN.from_command.set(0);
    sleep(1000);
    Ok(())
}
