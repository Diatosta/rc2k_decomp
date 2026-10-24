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
//! A list action (`off_51DD48` for the Clio disc). Caller `install::run_page`. Its calls
//! through `part`; the path strings built directly. Ungated. Shadow mode runs the original
//! alone. Self-test: a copy whose disk/graphics/script calls go to stand-ins, the lists
//! pre-filled, against the Rust version, with Escape pressed or not and a make failure.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    DIR_COUNT, DIRS, FILE_COUNT, FILES, GAME_DIR, INSTALLED, INSTALLING, PATH_A, PATH_B, a,
    append_byte, copy_picture, draw_string, g, lock, normalise, p, set_cd, show, sleep, str_append,
    str_append_indexed, take_key, unlock,
};

const UPDATING: usize = 0x51B9CC; // dword_51B9CC
const FROM_COMMAND: usize = 0x51B9D0; // dword_51B9D0
const SCRIPT: usize = 0x529DCC; // byte_529DCC (the Clio script)
const FONT_BE: usize = 0x51E904; // stru_51E904

/// `to` = `base` + name `index` of `table`, NUL-ended.
fn append_both(base: usize, to: usize, table: usize, index: u32) {
    let cursor = str_append(a(base), a(to));
    let cursor = str_append_indexed(g(table), index, cursor);
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
    if !normalise(a(PATH_A), a(GAME_DIR))?
        || !normalise(a(PATH_B), a(INSTALLED))?
        || !set_cd(a(GAME_DIR))
    {
        return Ok(Stop::Straight);
    }
    i::run_script::run_script(a(SCRIPT))?;
    if g(DIR_COUNT) != 0 {
        let mut n = 0;
        loop {
            append_both(INSTALLED, PATH_B, DIRS, n);
            if !i::make_folders::make_folders(a(PATH_B))? {
                return Ok(Stop::SetBack);
            }
            n = n.wrapping_add(1);
            if n == g(DIR_COUNT) {
                break;
            }
        }
        set_cd(a(GAME_DIR));
    }
    if g(FILE_COUNT) != 0 {
        let mut n = 0;
        loop {
            append_both(GAME_DIR, PATH_A, FILES, n);
            append_both(INSTALLED, PATH_B, FILES, n);
            // 0x416886: a failed copy stops the install as Escape does.
            if !i::copy_counted::copy_counted(a(PATH_A), a(PATH_B))? || take_key() == 1 {
                return Ok(Stop::SetBack);
            }
            n = n.wrapping_add(1);
            if n == g(FILE_COUNT) {
                break;
            }
        }
    }
    Ok(Stop::SetBack)
}

pub fn install_clio() -> Result<(), Refused> {
    if lock() {
        copy_picture()?;
        draw_string(0xBE, a(FONT_BE), 0x1A9, 0x113)?;
        unlock();
        show();
        sleep(1000);
    }
    if let Stop::SetBack = work()? {
        set_cd(a(INSTALLED));
    }
    p(INSTALLING, 0);
    p(UPDATING, 0);
    p(FROM_COMMAND, 0);
    sleep(1000);
    Ok(())
}
