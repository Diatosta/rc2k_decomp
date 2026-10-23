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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    DELETE_FILE, DIR_COUNT, DIRS, FILE_COUNT, FILES, INSTALLED, INSTALLING, Part, REMOVE_DIR,
    RUN_FRAME, RUN_SCRIPT, SET_CD, SLEEP, SOURCE, STATUS, a, append_byte, at, both, g, gb, p, pb,
    quiet, region, str_append, str_append_indexed, wb,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::uninstall", rva(0x4134DF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

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

pub fn uninstall(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if gb(STATUS) == 0 || crate::install::installer::gb(SOURCE_VAR) == 0 {
        return Ok(*entry);
    }
    pb(STATUS, 0x80);
    part(at(WRITE_REGISTRY), &mut r)?;
    // the "removing" frame
    let mut f = Regs { eax: 0, ..r };
    part(at(RUN_FRAME), &mut f)?;
    part(at(LOCK), &mut r)?;
    if !r.carry() {
        part(at(COPY_PICTURE), &mut r)?;
        r.eax = 1;
        r.ebx = a(REMOVE_STRINGS);
        r.ecx = 0;
        r.edx = 0;
        part(at(DRAW_LIST), &mut r)?;
        part(at(UNLOCK), &mut r)?;
        part(at(SHOW), &mut r)?;
    }
    r.esi = a(SOURCE);
    r.edx = a(INSTALLED);
    part(at(NORMALISE), &mut r)?;
    r.edx = a(INSTALLED);
    part(at(SET_CD), &mut r)?;
    r.ebx = a(SCRIPT);
    part(at(RUN_SCRIPT), &mut r)?;
    // Register for register: ebp the index, edx the path being built.
    if g(FILE_COUNT) != 0 {
        r.ebp = 0;
        loop {
            r.edx = a(PATH_B);
            build_path(FILES, r.ebp, &mut r);
            r.eax = r.edx;
            part(at(DELETE_FILE), &mut r)?;
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
            part(at(REMOVE_DIR), &mut r)?;
            r.ebp = r.ebp.wrapping_sub(1);
            if (r.ebp as i32) < 0 {
                break;
            }
        }
    }
    r.edx = a(DOTDOT);
    part(at(SET_CD), &mut r)?;
    r.edx = a(INSTALLED);
    part(at(REMOVE_DIR), &mut r)?;
    r.eax = 1000;
    part(at(SLEEP), &mut r)?;
    pb(STATUS, 0);
    wb(a(SOURCE_VAR), 0);
    part(at(WRITE_REGISTRY), &mut r)?;
    part(at(UNREGISTER), &mut r)?;
    p(REMOVE_ITEMS, 1);
    p(INSTALLING, 0);
    Ok(Regs { ..*entry })
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        uninstall(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        uninstall(r, &mut crate::install::installer::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp g_installStatus, 0
    let copy = match crate::install::installer::copy(
        0x4134DF,
        0x14A,
        &[0x60, 0x80, 0x3D, 0x8C, 0xBB, 0x51, 0x00, 0x00],
        &[
            (0x22, 0x413DC5),
            (0x2A, 0x405A71),
            (0x30, 0x4058AD),
            (0x37, 0x4060E3),
            (0x4B, 0x402370),
            (0x50, 0x405949),
            (0x55, 0x4057D1),
            (0x66, 0x4139EC),
            (0x71, 0x403105),
            (0x7C, 0x41365D),
            (0x9A, 0x401F7D),
            (0xA6, 0x401F8E),
            (0xAB, 0x401F16),
            (0xB2, 0x402E8C),
            (0xDE, 0x401F7D),
            (0xEA, 0x401F8E),
            (0xEF, 0x401F16),
            (0xF4, 0x403123),
            (0x102, 0x403105),
            (0x10D, 0x403123),
            (0x117, 0x41100C),
            (0x12A, 0x413DC5),
            (0x12F, 0x4190E9),
        ],
        &[
            0x413DC5, 0x405A71, 0x4058AD, 0x4060E3, 0x402370, 0x405949, 0x4057D1, 0x4139EC,
            0x403105, 0x41365D, 0x402E8C, 0x403123, 0x41100C, 0x4190E9,
        ],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::installer::set_text_of(Some(crate::install::installer::path_text));
    // a file-name table (packed) and a folder-name table
    let mut files_tbl = b"game.exe\0data.bun\0readme.txt\0".to_vec();
    let mut dirs_tbl = b"files\0files\\d\0".to_vec();
    let fptr = files_tbl.as_mut_ptr() as u32;
    let dptr = dirs_tbl.as_mut_ptr() as u32;
    let regions = [
        region(STATUS, 4),
        region(SOURCE_VAR, 1),
        region(INSTALLING, 4),
        region(REMOVE_ITEMS, 4),
        region(PATH_B, 0x200),
        region(INSTALLED, 0x100),
        region(FILES, 4),
        region(FILE_COUNT, 4),
        region(DIRS, 4),
        region(DIR_COUNT, 4),
    ];
    let regions = &regions[..];
    let saved = crate::race::viewer::Saved::new(regions);
    for (status, source, lock_fail) in [(1u8, b's', false), (1, b's', true), (0, 0, false)] {
        let prepare = || {
            crate::install::installer::answer(Box::new(move |place, r| {
                if place == a(0x4058AD) {
                    r.set_carry(lock_fail);
                } else if place == a(0x4139EC) {
                    r.set_carry(false);
                }
            }));
            pb(STATUS, status);
            wb(a(SOURCE_VAR), source);
            p(INSTALLING, 1);
            p(REMOVE_ITEMS, 0);
            // the install folder and the lists the script "built"
            for (k, &b) in b"c:\\rally\\\0".iter().enumerate() {
                wb(a(INSTALLED) + k as u32, b);
            }
            p(FILES, fptr);
            p(FILE_COUNT, if status == 0 { 0 } else { 3 });
            p(DIRS, dptr);
            p(DIR_COUNT, if status == 0 { 0 } else { 2 });
            crate::install::installer::fill(a(PATH_B), 0x200, 0xAA);
        };
        both(
            &mut t,
            &format!("status {status}, source {source}, lock_fail {lock_fail}"),
            &copy,
            Regs::SENTINEL,
            regions,
            0,
            &[],
            &prepare,
            &|r, part, _| uninstall(r, part),
        );
    }
    crate::install::installer::set_text_of(None);
    quiet();
    drop(saved);
    drop((files_tbl, dirs_tbl));
    t.finish()
}
