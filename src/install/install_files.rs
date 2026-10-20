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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    ADD_ITEMS, COPY_COUNTED, DIR_COUNT, DIRS, FILE_COUNT, FILES, FILES_COPIED, FILES_TO_COPY,
    GAME_DIR, IGNORE_ROOM, INSTALLED, INSTALLING, LIST_ANSWER, MAKE_FOLDERS, MEDIA_MODE, NORMALISE,
    OFFER_REGISTRATION, PATH_A, PATH_B, PATH_PROBLEM, Part, ROOM_NEEDED, RUN_REGISTRATION,
    RUN_SCRIPT, STATUS, TAKE_KEY, TYPED, a, append_byte, at, both, g, gb, p, pb, quiet, rb, region,
    rw, str_append, str_append_indexed, wb, wd,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::install_files", rva(0x4130CD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

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
    if crate::framework::standalone() {
        r.set_zero(true);
    }
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

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_parts(&HOOK, r, call_original, &|r| {
        install_files(r, &mut crate::install::installer::game_part)
    });
}

static KEY_N: std::sync::Mutex<u32> = std::sync::Mutex::new(0);

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::install::installer::copy(
        0x4130CD,
        0x412,
        &[0x60, 0xC6, 0x05, 0x8C, 0xBB, 0x51, 0x00, 0x80],
        &CALLS,
        &STOOD,
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::installer::set_text_of(Some(crate::install::installer::path_text));
    let mut files_tbl = b"a.ro3\0b.ro3\0".to_vec();
    let mut dirs_tbl = b"files\0files\\b\0".to_vec();
    let fptr = files_tbl.as_mut_ptr() as u32;
    let dptr = dirs_tbl.as_mut_ptr() as u32;
    let regions = [
        region(STATUS, 4),
        region(PATH_PROBLEM, 4),
        region(LIST_ANSWER, 1),
        region(INSTALLING, 4),
        region(ADD_ITEMS, 4),
        region(RUN_REGISTRATION, 4),
        region(FILES_COPIED, 4),
        region(COPY_SPARE, 4),
        region(FILES_TO_COPY, 4),
        region(ROOM_NEEDED, 4),
        region(SOURCE_VAR, 0x100),
        region(SOURCE_SIZE, 4),
        region(TYPED, 0x100),
        region(INSTALLED, 0x100),
        region(PATH_A, 0x100),
        region(PATH_B, 0x100),
        region(FILES, 4),
        region(FILE_COUNT, 4),
        region(DIRS, 4),
        region(DIR_COUNT, 4),
        // written by `prepare` only: in the list so that they are put back
        region(MEDIA_MODE, 4),
        region(IGNORE_ROOM, 4),
        region(OFFER_REGISTRATION, 4),
        region(LOCALE, 0x100),
        region(GAME_DIR, 0x100),
    ];
    let saved = crate::race::viewer::Saved::new(&regions);
    for (what, norm_fail, folder_bad, free_kb, is_cd, locale, cancel_at) in [
        ("success", false, false, 0x100000u32, true, 0u8, 99u32),
        ("bad path", true, false, 0x100000, true, 0, 99),
        ("no room", false, false, 1, true, 0, 99),
        ("not a cd", false, false, 0x100000, false, 0, 99),
        ("cancel", false, false, 0x100000, true, 0, 0),
        ("with readme", false, false, 0x100000, true, b'f', 99),
    ] {
        let prepare = || {
            *KEY_N.lock().unwrap() = 0;
            // Every call answers its flags (a stand-in otherwise hands back the flags it
            // was called with, which differ between the sides): carry clear but where
            // asked; the key's zero flag by the key, and a key at the 20th wait at most.
            crate::install::installer::answer(Box::new(move |place, r| {
                r.set_carry(false);
                r.set_zero(false);
                match place {
                    x if x == a(NORMALISE) => r.set_carry(norm_fail && r.esi == a(TYPED)),
                    x if x == a(CHECK_FOLDER) => r.set_carry(folder_bad),
                    x if x == a(FREE_SPACE) => r.eax = free_kb,
                    x if x == a(IS_CD) => r.set_zero(is_cd),
                    x if x == a(TAKE_KEY) => {
                        let mut n = KEY_N.lock().unwrap();
                        let k = *n;
                        *n += 1;
                        r.eax = u32::from(k >= cancel_at || k >= 20);
                        r.set_zero(r.eax == 0);
                    }
                    _ => {}
                }
            }));
            pb(STATUS, 9);
            p(PATH_PROBLEM, 9);
            pb(LIST_ANSWER, 9);
            p(INSTALLING, 0);
            p(ADD_ITEMS, 0);
            p(RUN_REGISTRATION, 0);
            p(MEDIA_MODE, 0);
            p(IGNORE_ROOM, 0);
            p(OFFER_REGISTRATION, 1);
            pb(LOCALE, locale);
            for (k, &b) in b"c:\\rally\\\0".iter().enumerate() {
                wb(a(TYPED) + k as u32, b);
            }
            crate::install::installer::fill(a(INSTALLED), 0x100, 0);
            for (k, &b) in b"c:\\rally\\\0".iter().enumerate() {
                wb(a(GAME_DIR) + k as u32, b);
            }
            p(FILES, fptr);
            p(FILE_COUNT, 2);
            p(DIRS, dptr);
            p(DIR_COUNT, 2);
        };
        both(
            &mut t,
            what,
            &copy,
            Regs::SENTINEL,
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| install_files(r, part),
        );
    }
    crate::install::installer::set_text_of(None);
    quiet();
    drop(saved);
    drop((files_tbl, dirs_tbl));
    t.finish()
}

#[rustfmt::skip]
const CALLS: [(usize, u32); 61] = [
    (0x8,0x413DC5),(0x19,0x4139EC),(0x24,0x41365D),(0x5C,0x4139EC),(0x72,0x413A6C),(0x82,0x40317D),
    (0xA9,0x413A97),(0xB5,0x403310),(0xC6,0x405A71),(0xE4,0x401F7D),(0xF0,0x401F8E),(0xF5,0x401F16),
    (0xFA,0x413A97),(0x114,0x403105),(0x136,0x401F7D),(0x142,0x401F8E),(0x147,0x401F16),(0x15A,0x401F7D),
    (0x166,0x401F8E),(0x16B,0x401F16),(0x170,0x4139DA),(0x181,0x405A71),(0x18E,0x405A71),(0x193,0x407F95),
    (0x1B3,0x403105),(0x1B8,0x411186),(0x1CE,0x4058AD),(0x1E7,0x406869),(0x226,0x402628),(0x23C,0x401ACB),
    (0x241,0x401F16),(0x262,0x4023D5),(0x267,0x405949),(0x26C,0x4057D1),(0x279,0x4058AD),(0x280,0x4060E3),
    (0x294,0x402370),(0x2AE,0x4066DF),(0x2B3,0x405949),(0x2B8,0x4057D1),(0x2E1,0x405A71),(0x2E7,0x4058AD),
    (0x2F0,0x402370),(0x2F5,0x405949),(0x2FA,0x4057D1),(0x304,0x41100C),(0x309,0x407F95),(0x30E,0x407F95),
    (0x319,0x410FFE),(0x31E,0x410535),(0x348,0x401F7D),(0x353,0x401F7D),(0x358,0x401F16),(0x363,0x4139DA),
    (0x376,0x401F7D),(0x381,0x401F7D),(0x386,0x401F16),(0x391,0x4139DA),(0x3A2,0x41100C),(0x3CD,0x413DC5),
    (0x3D2,0x419030),
];
#[rustfmt::skip]
const STOOD: [u32; 20] = [
    0x413DC5,0x4139EC,0x41365D,0x40317D,0x413A6C,0x413A97,0x403310,0x405A71,0x403105,0x4139DA,
    0x411186,0x41100C,0x410FFE,0x410535,0x419030,0x407F95,0x4058AD,0x405949,0x4057D1,0x402370,
];
