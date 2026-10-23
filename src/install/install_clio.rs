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

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    COPY_COUNTED, DIR_COUNT, DIRS, FILE_COUNT, FILES, GAME_DIR, INSTALLED, INSTALLING,
    MAKE_FOLDERS, NORMALISE, PATH_A, PATH_B, Part, RUN_SCRIPT, SET_CD, SLEEP, TAKE_KEY, a,
    append_byte, at, g, p, str_append, str_append_indexed,
};

const LOCK: usize = 0x4058AD;
const UNLOCK: usize = 0x405949;
const SHOW: usize = 0x4057D1;
const COPY_PICTURE: usize = 0x4060E3;
const DRAW_STRING: usize = 0x40234A;
const UPDATING: usize = 0x51B9CC; // dword_51B9CC
const FROM_COMMAND: usize = 0x51B9D0; // dword_51B9D0
const SCRIPT: usize = 0x529DCC; // byte_529DCC (the Clio script)
const FONT_BE: usize = 0x51E904; // stru_51E904

/// `ebx` = `dst` (the caller sets esi, edi or edx to it), then `src_base` + name #ebp of
/// `table`, NUL-ended.
fn append_both(src_base: usize, dst: usize, table: usize, r: &mut Regs) {
    r.ebx = a(dst);
    r.eax = a(src_base);
    str_append(r);
    r.eax = g(table);
    r.ecx = r.ebp;
    str_append_indexed(r);
    append_byte(r, 0);
}

/// Where the work stops: `loc_41689A` (the current directory set back first) or
/// `loc_4168A5` (not).
enum Stop {
    SetBack,
    Straight,
}

/// The Rust version, register for register as the original.
pub fn install_clio(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(at(LOCK), &mut r)?;
    if !r.carry() {
        part(at(COPY_PICTURE), &mut r)?;
        r.eax = 0xBE;
        r.ebx = a(FONT_BE);
        r.ecx = 0x1A9;
        r.edx = 0x113;
        part(at(DRAW_STRING), &mut r)?;
        part(at(UNLOCK), &mut r)?;
        part(at(SHOW), &mut r)?;
        r.eax = 1000;
        part(at(SLEEP), &mut r)?;
    }
    let stop = (|| -> Result<Stop, Refused> {
        r.esi = a(PATH_A);
        r.edx = a(GAME_DIR);
        part(at(NORMALISE), &mut r)?;
        if r.carry() {
            return Ok(Stop::Straight);
        }
        r.esi = a(PATH_B);
        r.edx = a(INSTALLED);
        part(at(NORMALISE), &mut r)?;
        if r.carry() {
            return Ok(Stop::Straight);
        }
        r.edx = a(GAME_DIR);
        part(at(SET_CD), &mut r)?;
        if r.carry() {
            return Ok(Stop::Straight);
        }
        r.ebx = a(SCRIPT);
        part(at(RUN_SCRIPT), &mut r)?;
        if g(DIR_COUNT) != 0 {
            r.ebp = 0;
            loop {
                r.edx = a(PATH_B);
                append_both(INSTALLED, PATH_B, DIRS, &mut r);
                part(at(MAKE_FOLDERS), &mut r)?;
                if r.carry() {
                    return Ok(Stop::SetBack);
                }
                r.ebp = r.ebp.wrapping_add(1);
                if r.ebp == g(DIR_COUNT) {
                    break;
                }
            }
            r.edx = a(GAME_DIR);
            part(at(SET_CD), &mut r)?;
        }
        if g(FILE_COUNT) != 0 {
            r.ebp = 0;
            loop {
                r.esi = a(PATH_A);
                append_both(GAME_DIR, PATH_A, FILES, &mut r);
                r.edi = a(PATH_B);
                append_both(INSTALLED, PATH_B, FILES, &mut r);
                part(at(COPY_COUNTED), &mut r)?;
                if r.carry() {
                    return Ok(Stop::SetBack);
                }
                part(at(TAKE_KEY), &mut r)?;
                if r.eax as u8 == 1 {
                    return Ok(Stop::SetBack);
                }
                r.ebp = r.ebp.wrapping_add(1);
                if r.ebp == g(FILE_COUNT) {
                    break;
                }
            }
        }
        Ok(Stop::SetBack)
    })()?;
    if let Stop::SetBack = stop {
        r.edx = a(INSTALLED);
        part(at(SET_CD), &mut r)?;
    }
    p(INSTALLING, 0);
    p(UPDATING, 0);
    p(FROM_COMMAND, 0);
    r.eax = 1000;
    part(at(SLEEP), &mut r)?;
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        install_clio(r, &mut crate::install::installer::game_part)
    })
}
