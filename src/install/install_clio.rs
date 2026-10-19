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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    COPY_COUNTED, DIR_COUNT, DIRS, FILE_COUNT, FILES, GAME_DIR, INSTALLED, INSTALLING,
    MAKE_FOLDERS, NORMALISE, PATH_A, PATH_B, Part, RUN_SCRIPT, SET_CD, SLEEP, TAKE_KEY, a,
    append_byte, at, both, g, p, quiet, region, str_append, str_append_indexed,
};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::install_clio", rva(0x41675B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

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

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        install_clio(r, &mut crate::install::installer::game_part)
    });
}

static KEY_N: Mutex<u32> = Mutex::new(0);

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call lock_surface
    let copy = match crate::install::installer::copy(
        0x41675B,
        0x174,
        &[0x60, 0xE8, 0x4C, 0xF1, 0xFE, 0xFF],
        &[
            (0x1, 0x4058AD),
            (0x8, 0x4060E3),
            (0x22, 0x40234A),
            (0x27, 0x405949),
            (0x2C, 0x4057D1),
            (0x36, 0x41100C),
            (0x47, 0x4139EC),
            (0x5E, 0x4139EC),
            (0x6F, 0x403105),
            (0x80, 0x41365D),
            (0x9E, 0x401F7D),
            (0xAA, 0x401F8E),
            (0xAF, 0x401F16),
            (0xB4, 0x413A97),
            (0xCE, 0x403105),
            (0xEC, 0x401F7D),
            (0xF8, 0x401F8E),
            (0xFD, 0x401F16),
            (0x110, 0x401F7D),
            (0x11C, 0x401F8E),
            (0x121, 0x401F16),
            (0x126, 0x4139DA),
            (0x12D, 0x407F95),
            (0x145, 0x403105),
            (0x16D, 0x41100C),
        ],
        &[
            0x4058AD, 0x4060E3, 0x40234A, 0x405949, 0x4057D1, 0x41100C, 0x4139EC, 0x403105,
            0x41365D, 0x413A97, 0x4139DA, 0x407F95,
        ],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::installer::set_text_of(Some(crate::install::installer::path_text));
    let mut files_tbl = b"a.ro3\0b.ro3\0c.ro3\0".to_vec();
    let mut dirs_tbl = b"files\0files\\b\0".to_vec();
    let fptr = files_tbl.as_mut_ptr() as u32;
    let dptr = dirs_tbl.as_mut_ptr() as u32;
    let regions = [
        region(INSTALLING, 4),
        region(UPDATING, 4),
        region(FROM_COMMAND, 4),
        region(PATH_A, 0x100),
        region(PATH_B, 0x100),
        region(GAME_DIR, 0x100),
        region(INSTALLED, 0x100),
        region(FILES, 4),
        region(FILE_COUNT, 4),
        region(DIRS, 4),
        region(DIR_COUNT, 4),
    ];
    let saved = crate::race::viewer::Saved::new(&regions);
    for (lock_fail, make_fail, escape_at) in [
        (false, false, 99u32),
        (true, false, 99),
        (false, true, 99),
        (false, false, 1),
    ] {
        let prepare = || {
            crate::install::installer::answer(Box::new(move |place, r| {
                if place == a(0x4058AD) {
                    r.set_carry(lock_fail);
                } else if place == a(0x4139EC) || place == a(0x403105) {
                    r.set_carry(false);
                } else if place == a(0x413A97) {
                    r.set_carry(make_fail);
                } else if place == a(0x4139DA) {
                    r.set_carry(false);
                } else if place == a(0x407F95) {
                    let mut n = KEY_N.lock().unwrap();
                    let k = *n;
                    *n += 1;
                    r.eax = u32::from(k == escape_at);
                }
            }));
            *KEY_N.lock().unwrap() = 0;
            for base in [PATH_A, PATH_B] {
                for (k, &b) in b"d:\\clio\\\0".iter().enumerate() {
                    crate::install::installer::wb(a(base) + k as u32, b);
                }
            }
            crate::install::installer::fill(a(GAME_DIR), 0x100, 0);
            crate::install::installer::fill(a(INSTALLED), 0x100, 0);
            p(FILES, fptr);
            p(FILE_COUNT, 3);
            p(DIRS, dptr);
            p(DIR_COUNT, 2);
        };
        both(
            &mut t,
            &format!("lock_fail {lock_fail}, make_fail {make_fail}, escape_at {escape_at}"),
            &copy,
            Regs::SENTINEL,
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| install_clio(r, part),
        );
    }
    crate::install::installer::set_text_of(None);
    quiet();
    drop(saved);
    drop((files_tbl, dirs_tbl));
    t.finish()
}
