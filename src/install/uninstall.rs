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

use crate::errors::Refused;
use crate::install::installer::{
    DIR_COUNT, DIRS, FILE_COUNT, FILES, INSTALLED, INSTALLING, SOURCE, STATUS, a, append_byte,
    copy_picture, delete_file, g, game_api, gb, lock, normalise, p, pb, remove_dir, run_frame,
    set_cd, show, sleep, str_append, str_append_indexed, unlock, wb,
};

const REMOVE_ITEMS: usize = 0x51B9D8; // g_removeStartItems
const PATH_B: usize = 0x95E9F8; // byte_95E9F8
const SOURCE_VAR: usize = 0x51BB8D; // g_installSource
const SCRIPT: usize = 0x51BE10; // byte_51BE10
const REMOVE_STRINGS: usize = 0x51D473; // unk_51D473
const DOTDOT: usize = 0x5295B9; // ".."

/// byte_95E9F8 = byte_95E7F8 + name #index of `table`, NUL-ended.
fn build_path(table: usize, index: u32) -> u32 {
    let cursor = str_append(a(INSTALLED), a(PATH_B));
    let cursor = str_append_indexed(g(table), index, cursor);
    append_byte(cursor, 0);
    a(PATH_B)
}

pub fn uninstall() -> Result<(), Refused> {
    use crate::install as i;
    if gb(STATUS) == 0 || gb(SOURCE_VAR) == 0 {
        return Ok(());
    }
    pb(STATUS, 0x80);
    i::write_registry::write_registry(&mut game_api)?;
    // the "removing" frame
    run_frame(None)?;
    if lock() {
        copy_picture()?;
        i::draw_string_list::draw_string_list(1, a(REMOVE_STRINGS), 0, 0)?;
        unlock();
        show();
    }
    normalise(a(SOURCE), a(INSTALLED))?;
    set_cd(a(INSTALLED));
    i::run_script::run_script(a(SCRIPT))?;
    if g(FILE_COUNT) != 0 {
        let mut n = 0;
        loop {
            delete_file(build_path(FILES, n));
            n = n.wrapping_add(1);
            if n == g(FILE_COUNT) {
                break;
            }
        }
    }
    if g(DIR_COUNT) != 0 {
        let mut n = g(DIR_COUNT).wrapping_sub(1);
        loop {
            remove_dir(build_path(DIRS, n));
            n = n.wrapping_sub(1);
            if (n as i32) < 0 {
                break;
            }
        }
    }
    set_cd(a(DOTDOT));
    remove_dir(a(INSTALLED));
    sleep(1000);
    pb(STATUS, 0);
    wb(a(SOURCE_VAR), 0);
    i::write_registry::write_registry(&mut game_api)?;
    i::unregister_lobby::unregister_lobby(&mut game_api);
    p(REMOVE_ITEMS, 1);
    p(INSTALLING, 0);
    Ok(())
}
