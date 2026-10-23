//! `install::install_files` (0x4130CD): the install itself. The status is marked busy and
//! written; the source is scanned (`install::run_script` with `byte_51BD48`) to count the
//! files and the room needed (`dword_51C3F7[g_mediaMode]`); the destination is normalised,
//! checked for a bad name (`install::check_folder_name`), for room
//! (`files::drive_free_space_kb` unless UNC) and made (`install::make_folders`); the source
//! must be a CD (`files::is_cd_drive`). Then the page is shown and the folders made and the
//! files copied one by one (`install::make_folders`, `install::copy_counted`), a frame drawn
//! after each and Escape watched; the record is written (`install::write_record`); the readme
//! and manual are copied if a language is set; finally the status is set installed, written,
//! the lobby registered (`install::register_lobby`), the Start-menu add and
//! `g_installing` asked, and registration if offered. Failures set `byte_51D415`
//! (0 bad path, 3 no room / not a CD) or show an error/cancel message.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD18`). Caller `install::run_page`. Its calls through `part`; the
//! path strings built directly; the two progress frames are the game's (section 15, reached
//! via `screens::run_frame`). Ungated. Shadow mode runs the original alone. Self-test: a copy
//! whose disk/registry/frame/script calls go to stand-ins, the lists pre-filled as the script
//! would, against the Rust version; the main success path and the bad-path, no-room, not-a-CD,
//! copy-failure and cancel branches.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    ADD_ITEMS, COPY_COUNTED, DIR_COUNT, DIRS, FILE_COUNT, FILES, FILES_COPIED, FILES_TO_COPY,
    GAME_DIR, IGNORE_ROOM, INSTALLED, INSTALLING, LIST_ANSWER, MAKE_FOLDERS, MEDIA_MODE, NORMALISE,
    OFFER_REGISTRATION, PATH_A, PATH_B, PATH_PROBLEM, Part, ROOM_NEEDED, RUN_REGISTRATION,
    RUN_SCRIPT, STATUS, TAKE_KEY, TYPED, a, append_byte, at, g, gb, p, pb, rb, rw, str_append,
    str_append_indexed, wb, wd,
};

const WRITE_REGISTRY: usize = 0x413DC5;
const CHECK_FOLDER: usize = 0x413A6C;
const IS_CD: usize = 0x403310;
const FREE_SPACE: usize = 0x40317D;
const SET_CD: usize = 0x403105;
const WRITE_RECORD: usize = 0x411186;
const REGISTER_LOBBY: usize = 0x419030;
const RUN_FRAME: usize = 0x405A71;
const LOCK: usize = 0x4058AD;
const UNLOCK: usize = 0x405949;
const SHOW: usize = 0x4057D1;
const DRAW_LIST: usize = 0x402370;
const SLEEP: usize = 0x41100C;
const SLEEP_1MS: usize = 0x410FFE;
const PUMP: usize = 0x410535;
const SCRIPT_COUNT: usize = 0x51BD48; // byte_51BD48 (the count script)
const ROOM_BY_MODE: usize = 0x51C3F7; // dword_51C3F7
const PAGE_FRAME: usize = 0x413340;
const PROGRESS_FRAME: usize = 0x413295;
const COPY_SPARE: usize = 0x51BAF0; // dword_51BAF0
const SOURCE_VAR: usize = 0x51BB8D; // g_installSource
const SOURCE_SIZE: usize = 0x51BC90; // dword_51BC90
const LOCALE: usize = 0x518AA8; // g_localeDirectory
const MSG_ERROR: usize = 0x51D453; // unk_51D453
const MSG_CANCEL: usize = 0x51D463; // unk_51D463
const README: usize = 0x51BE6F; // "files\%readme\readme.txt"
const MANUAL: usize = 0x51BE88; // "files\%readme\manual.pdf"
const QUITTING: usize = 0x51837C; // g_quitting

/// `ebx` = `dst`, `extra` (esi or edx) the same, then `src_base` + name #ebp of `table`,
/// NUL-ended (`loc_4131A3`, `loc_4131F5`).
fn build(src_base: usize, dst: usize, table: usize, r: &mut Regs) {
    r.ebx = a(dst);
    r.eax = a(src_base);
    str_append(r);
    r.eax = g(table);
    r.ecx = r.ebp;
    str_append_indexed(r);
    append_byte(r, 0);
}

/// `al` = `byte` (the rest of eax kept).
fn set_al(r: &mut Regs, byte: u8) {
    r.eax = (r.eax & !0xFF) | byte as u32;
}

/// The error (count 2, `unk_51D453`, y 0) or cancel (count 1, `unk_51D463`, y -0x32)
/// message shown, then a wait for a key (`loc_4133AB`).
fn message(r: &mut Regs, count: u32, table: usize, y: u32, part: Part) -> Result<(), Refused> {
    r.eax = count;
    r.ebx = a(table);
    r.edx = y;
    // pusha; xor eax, eax; call screens::run_frame; popa
    let mut f = Regs { eax: 0, ..*r };
    part(at(RUN_FRAME), &mut f)?;
    part(at(LOCK), r)?;
    if !r.carry() {
        r.ecx = 0;
        part(at(DRAW_LIST), r)?;
        part(at(UNLOCK), r)?;
        part(at(SHOW), r)?;
    }
    r.eax = 500;
    part(at(SLEEP), r)?;
    part(at(TAKE_KEY), r)?;
    loop {
        part(at(TAKE_KEY), r)?;
        if r.eflags & crate::framework::ZERO_FLAG == 0 {
            break;
        }
        part(at(SLEEP_1MS), r)?;
        part(at(PUMP), r)?;
        if g(QUITTING) == 1 {
            break;
        }
    }
    Ok(())
}

/// The Rust version, register for register as the original (its calls get what it has).
pub fn install_files(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    pb(STATUS, 0x80);
    part(at(WRITE_REGISTRY), &mut r)?;
    r.esi = a(GAME_DIR);
    r.edx = a(GAME_DIR);
    part(at(NORMALISE), &mut r)?;
    r.ebx = a(SCRIPT_COUNT);
    part(at(RUN_SCRIPT), &mut r)?;
    r.eax = 0;
    p(FILES_COPIED, 0);
    p(COPY_SPARE, 0);
    r.eax = g(FILE_COUNT);
    p(FILES_TO_COPY, r.eax);
    r.eax = g(MEDIA_MODE);
    r.eax = unsafe {
        (a(ROOM_BY_MODE) as *const u32)
            .add(r.eax as usize)
            .read_unaligned()
    };
    p(ROOM_NEEDED, r.eax);
    r.esi = a(TYPED);
    r.edx = a(INSTALLED);
    part(at(NORMALISE), &mut r)?;
    // loc_413130: the path is bad; loc_4134D6: no room or not a CD.
    let bad_path = || -> Result<Regs, Refused> {
        p(PATH_PROBLEM, 1);
        pb(LIST_ANSWER, 0);
        Ok(*entry)
    };
    if r.carry() {
        return bad_path();
    }
    part(at(CHECK_FOLDER), &mut r)?;
    if r.carry() {
        return bad_path();
    }
    if rw(r.esi) != 0x5C5C {
        let drive = rb(r.edx);
        set_al(&mut r, drive);
        part(at(FREE_SPACE), &mut r)?;
        if r.carry() {
            return bad_path();
        }
        if r.eax < g(ROOM_NEEDED) && g(IGNORE_ROOM) != 1 {
            p(PATH_PROBLEM, 2);
            pb(LIST_ANSWER, 3);
            return Ok(*entry);
        }
    }
    part(at(MAKE_FOLDERS), &mut r)?;
    if r.carry() {
        return bad_path();
    }
    set_al(&mut r, rb(a(GAME_DIR)));
    part(at(IS_CD), &mut r)?;
    // The standalone installs from the download folder as well (owner, 2026-10-20).
    r.set_zero(true);

    if r.eflags & crate::framework::ZERO_FLAG == 0 {
        pb(LIST_ANSWER, 3);
        return Ok(*entry);
    }
    r.eax = a(PAGE_FRAME);
    part(at(RUN_FRAME), &mut r)?;
    // the folders
    if g(DIR_COUNT) != 0 {
        r.ebp = 0;
        loop {
            r.edx = a(PATH_B);
            build(INSTALLED, PATH_B, DIRS, &mut r);
            part(at(MAKE_FOLDERS), &mut r)?;
            if r.carry() {
                message(&mut r, 2, MSG_ERROR, 0, part)?;
                return Ok(*entry);
            }
            r.ebp = r.ebp.wrapping_add(1);
            if r.ebp == g(DIR_COUNT) {
                break;
            }
        }
        r.edx = a(GAME_DIR);
        part(at(SET_CD), &mut r)?;
    }
    // the files
    if g(FILE_COUNT) != 0 {
        r.ebp = 0;
        loop {
            r.esi = a(PATH_A);
            build(GAME_DIR, PATH_A, FILES, &mut r);
            r.edi = a(PATH_B);
            build(INSTALLED, PATH_B, FILES, &mut r);
            part(at(COPY_COUNTED), &mut r)?;
            if r.carry() {
                message(&mut r, 2, MSG_ERROR, 0, part)?;
                return Ok(*entry);
            }
            r.eax = a(PROGRESS_FRAME);
            part(at(RUN_FRAME), &mut r)?;
            if r.carry() {
                r.eax = a(PAGE_FRAME);
                part(at(RUN_FRAME), &mut r)?;
            }
            part(at(TAKE_KEY), &mut r)?;
            if r.eax as u8 == 1 {
                message(&mut r, 1, MSG_CANCEL, 0xFFFF_FFCE, part)?;
                return Ok(*entry);
            }
            r.ebp = r.ebp.wrapping_add(1);
            if r.ebp == g(FILE_COUNT) {
                break;
            }
        }
    }
    r.edx = a(INSTALLED);
    part(at(SET_CD), &mut r)?;
    part(at(WRITE_RECORD), &mut r)?;
    if r.carry() {
        message(&mut r, 2, MSG_ERROR, 0, part)?;
        return Ok(*entry);
    }
    if gb(LOCALE) != 0 {
        // readme.txt then manual.pdf
        for (full, base) in [(README, README + 0xE), (MANUAL, MANUAL + 0xE)] {
            r.ebx = a(PATH_A);
            r.esi = r.ebx;
            r.eax = a(GAME_DIR);
            str_append(&mut r);
            r.eax = a(full);
            str_append(&mut r);
            append_byte(&mut r, 0);
            r.edi = a(base);
            part(at(COPY_COUNTED), &mut r)?;
        }
    }
    pb(LIST_ANSWER, 2);
    r.eax = 1000;
    part(at(SLEEP), &mut r)?;
    pb(STATUS, 1);
    // byte_95E7F8 copied to g_installSource; dword_51BC90 its length with the NUL
    r.esi = a(INSTALLED);
    r.edi = a(SOURCE_VAR);
    r.ecx = 0;
    loop {
        let b = rb(r.esi.wrapping_add(r.ecx));
        set_al(&mut r, b);
        wb(r.edi.wrapping_add(r.ecx), b);
        r.ecx = r.ecx.wrapping_add(1);
        if b == 0 {
            break;
        }
    }
    wd(a(SOURCE_SIZE), r.ecx);
    part(at(WRITE_REGISTRY), &mut r)?;
    part(at(REGISTER_LOBBY), &mut r)?;
    p(ADD_ITEMS, 1);
    p(INSTALLING, 1);
    if g(OFFER_REGISTRATION) == 1 {
        p(RUN_REGISTRATION, 1);
    }
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        install_files(r, &mut crate::install::installer::game_part)
    })
}
