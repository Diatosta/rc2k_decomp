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
//! A list action (`off_51DD30`). Caller `install::run_page`. Its calls through `part`; the
//! path strings built directly. Ungated. Shadow mode runs the original alone. Self-test: a
//! copy whose disk/registry/frame/script calls go to stand-ins, the lists pre-filled as the
//! script would, against the Rust version; the deletes, removes and final globals compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    DELETE_FILE, DIR_COUNT, DIRS, FILE_COUNT, FILES, INSTALLED, INSTALLING, REMOVE_DIR, RUN_FRAME,
    RUN_SCRIPT, SET_CD, SLEEP, SOURCE, STATUS, a, append_byte, at, g, gb, p, pb, str_append,
    str_append_indexed, wb,
};

const WRITE_REGISTRY: usize = 0x413DC5;
const NORMALISE: usize = 0x4139EC;
const LOCK: usize = 0x4058AD;
const UNLOCK: usize = 0x405949;
const SHOW: usize = 0x4057D1;
const COPY_PICTURE: usize = 0x4060E3;
const DRAW_LIST: usize = 0x402370;
const UNREGISTER: usize = 0x4190E9;
const REMOVE_ITEMS: usize = 0x51B9D8; // g_removeStartItems
const PATH_B: usize = 0x95E9F8; // byte_95E9F8
const SOURCE_VAR: usize = 0x51BB8D; // g_installSource
const SCRIPT: usize = 0x51BE10; // byte_51BE10
const REMOVE_STRINGS: usize = 0x51D473; // unk_51D473
const DOTDOT: usize = 0x5295B9; // ".."

/// byte_95E9F8 = byte_95E7F8 + name #index of `table`, NUL-ended.
fn build_path(table: usize, index: u32, r: &mut Regs) {
    r.ebx = a(PATH_B);
    r.eax = a(INSTALLED);
    str_append(r);
    r.eax = g(table);
    r.ecx = index;
    str_append_indexed(r);
    append_byte(r, 0);
}

pub fn uninstall(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    if gb(STATUS) == 0 || crate::install::installer::gb(SOURCE_VAR) == 0 {
        return Ok(*entry);
    }
    pb(STATUS, 0x80);
    crate::install::installer::game_part(at(WRITE_REGISTRY), &mut r)?;
    // the "removing" frame
    let mut f = Regs { eax: 0, ..r };
    crate::install::installer::game_part(at(RUN_FRAME), &mut f)?;
    crate::install::installer::game_part(at(LOCK), &mut r)?;
    if !r.carry() {
        crate::install::installer::game_part(at(COPY_PICTURE), &mut r)?;
        r.eax = 1;
        r.ebx = a(REMOVE_STRINGS);
        r.ecx = 0;
        r.edx = 0;
        crate::install::installer::game_part(at(DRAW_LIST), &mut r)?;
        crate::install::installer::game_part(at(UNLOCK), &mut r)?;
        crate::install::installer::game_part(at(SHOW), &mut r)?;
    }
    r.esi = a(SOURCE);
    r.edx = a(INSTALLED);
    crate::install::installer::game_part(at(NORMALISE), &mut r)?;
    r.edx = a(INSTALLED);
    crate::install::installer::game_part(at(SET_CD), &mut r)?;
    r.ebx = a(SCRIPT);
    crate::install::installer::game_part(at(RUN_SCRIPT), &mut r)?;
    // Register for register: ebp the index, edx the path being built.
    if g(FILE_COUNT) != 0 {
        r.ebp = 0;
        loop {
            r.edx = a(PATH_B);
            build_path(FILES, r.ebp, &mut r);
            r.eax = r.edx;
            crate::install::installer::game_part(at(DELETE_FILE), &mut r)?;
            r.ebp = r.ebp.wrapping_add(1);
            if r.ebp == g(FILE_COUNT) {
                break;
            }
        }
    }
    if g(DIR_COUNT) != 0 {
        r.ebp = g(DIR_COUNT).wrapping_sub(1);
        loop {
            r.edx = a(PATH_B);
            build_path(DIRS, r.ebp, &mut r);
            crate::install::installer::game_part(at(REMOVE_DIR), &mut r)?;
            r.ebp = r.ebp.wrapping_sub(1);
            if (r.ebp as i32) < 0 {
                break;
            }
        }
    }
    r.edx = a(DOTDOT);
    crate::install::installer::game_part(at(SET_CD), &mut r)?;
    r.edx = a(INSTALLED);
    crate::install::installer::game_part(at(REMOVE_DIR), &mut r)?;
    r.eax = 1000;
    crate::install::installer::game_part(at(SLEEP), &mut r)?;
    pb(STATUS, 0);
    wb(a(SOURCE_VAR), 0);
    crate::install::installer::game_part(at(WRITE_REGISTRY), &mut r)?;
    crate::install::installer::game_part(at(UNREGISTER), &mut r)?;
    p(REMOVE_ITEMS, 1);
    p(INSTALLING, 0);
    Ok(Regs { ..*entry })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| uninstall(r))
}
