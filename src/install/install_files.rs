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
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    ADD_ITEMS, DIR_COUNT, DIRS, FILE_COUNT, FILES, FILES_COPIED, FILES_TO_COPY, GAME_DIR,
    IGNORE_ROOM, INSTALLED, INSTALLING, LIST_ANSWER, MEDIA_MODE, OFFER_REGISTRATION, PATH_A,
    PATH_B, PATH_PROBLEM, ROOM_NEEDED, RUN_REGISTRATION, STATUS, TYPED, a, append_byte, g,
    game_api, gb, lock, normalise, p, pb, pump, rb, run_frame, rw, set_cd, show, sleep, str_append,
    str_append_indexed, take_key, unlock, wb, wd,
};
use crate::session::FLAGS;

const SCRIPT_COUNT: usize = 0x51BD48; // byte_51BD48 (the count script)
const ROOM_BY_MODE: usize = 0x51C3F7; // dword_51C3F7
const COPY_SPARE: usize = 0x51BAF0; // dword_51BAF0
const SOURCE_VAR: usize = 0x51BB8D; // g_installSource
const SOURCE_SIZE: usize = 0x51BC90; // dword_51BC90
const LOCALE: usize = 0x518AA8; // g_localeDirectory
const MSG_ERROR: usize = 0x51D453; // unk_51D453
const MSG_CANCEL: usize = 0x51D463; // unk_51D463
const README: usize = 0x51BE6F; // "files\%readme\readme.txt"
const MANUAL: usize = 0x51BE88; // "files\%readme\manual.pdf"

/// `to` = `base` + name `index` of `table`, NUL-ended (`loc_4131A3`, `loc_4131F5`).
fn build(base: usize, to: usize, table: usize, index: u32) {
    let cursor = str_append(a(base), a(to));
    let cursor = str_append_indexed(g(table), index, cursor);
    append_byte(cursor, 0);
}

/// The error (count 2, `unk_51D453`, y 0) or cancel (count 1, `unk_51D463`, y -0x32)
/// message shown, then a wait for a key (`loc_4133AB`).
fn message(count: u32, table: usize, y: u32) -> Result<(), Refused> {
    run_frame(None)?;
    if lock() {
        crate::install::draw_string_list::draw_string_list(count, a(table), 0, y)?;
        unlock();
        show();
    }
    sleep(500);
    take_key();
    loop {
        if take_key() != 0 {
            break;
        }
        sleep(1);
        pump();
        if FLAGS.quitting.get() == 1 {
            break;
        }
    }
    Ok(())
}

/// The installer's page frames (`screens::run_frame` runs them).
fn copying_frame() -> Result<bool, Refused> {
    crate::install::install_frames::copying_frame()
}

fn progress_frame() -> Result<bool, Refused> {
    crate::install::install_frames::progress_frame()
}

pub fn install_files() -> Result<(), Refused> {
    use crate::install as i;
    pb(STATUS, 0x80);
    i::write_registry::write_registry(&mut game_api)?;
    normalise(a(GAME_DIR), a(GAME_DIR))?;
    i::run_script::run_script(a(SCRIPT_COUNT))?;
    p(FILES_COPIED, 0);
    p(COPY_SPARE, 0);
    p(FILES_TO_COPY, g(FILE_COUNT));
    let room = unsafe {
        (a(ROOM_BY_MODE) as *const u32)
            .add(g(MEDIA_MODE) as usize)
            .read_unaligned()
    };
    p(ROOM_NEEDED, room);
    // loc_413130: the path is bad; loc_4134D6: no room or not a CD.
    let bad_path = || {
        p(PATH_PROBLEM, 1);
        pb(LIST_ANSWER, 0);
        Ok(())
    };
    if !normalise(a(TYPED), a(INSTALLED))? {
        return bad_path();
    }
    if i::check_folder_name::check_folder_name(a(INSTALLED)) {
        return bad_path();
    }
    if rw(a(TYPED)) != 0x5C5C {
        let Some(free) = crate::files::drive_free_space_kb::drive_free_space_kb(
            rb(a(INSTALLED)) as u32,
            &mut game_api,
        ) else {
            return bad_path();
        };
        if free < g(ROOM_NEEDED) && g(IGNORE_ROOM) != 1 {
            p(PATH_PROBLEM, 2);
            pb(LIST_ANSWER, 3);
            return Ok(());
        }
    }
    if !i::make_folders::make_folders(a(INSTALLED))? {
        return bad_path();
    }
    // Its answer is not read: the standalone installs from the download folder as well
    // (owner, 2026-10-20).
    unsafe { crate::files::is_cd_drive::is_cd_drive(rb(a(GAME_DIR))) };
    run_frame(Some(copying_frame))?;
    // the folders
    if g(DIR_COUNT) != 0 {
        for n in 0..g(DIR_COUNT) {
            build(INSTALLED, PATH_B, DIRS, n);
            if !i::make_folders::make_folders(a(PATH_B))? {
                return message(2, MSG_ERROR, 0);
            }
        }
        set_cd(a(GAME_DIR));
    }
    // the files
    if g(FILE_COUNT) != 0 {
        let mut n = 0;
        loop {
            build(GAME_DIR, PATH_A, FILES, n);
            build(INSTALLED, PATH_B, FILES, n);
            // 0x413242: a failed copy is the error message.
            if !i::copy_counted::copy_counted(a(PATH_A), a(PATH_B))? {
                return message(2, MSG_ERROR, 0);
            }
            if run_frame(Some(progress_frame))? {
                run_frame(Some(copying_frame))?;
            }
            if take_key() == 1 {
                return message(1, MSG_CANCEL, 0xFFFF_FFCE);
            }
            n = n.wrapping_add(1);
            if n == g(FILE_COUNT) {
                break;
            }
        }
    }
    // 0x413280: the change of folder's result is not looked at; 0x41328A: a failed
    // record write is the error message.
    set_cd(a(INSTALLED));
    if !i::write_record::write_record()? {
        return message(2, MSG_ERROR, 0);
    }
    if gb(LOCALE) != 0 {
        // readme.txt then manual.pdf
        for (full, base) in [(README, README + 0xE), (MANUAL, MANUAL + 0xE)] {
            let cursor = str_append(a(GAME_DIR), a(PATH_A));
            let cursor = str_append(a(full), cursor);
            append_byte(cursor, 0);
            // Not looked at (0x413435, 0x413463).
            i::copy_counted::copy_counted(a(PATH_A), a(base))?;
        }
    }
    pb(LIST_ANSWER, 2);
    sleep(1000);
    pb(STATUS, 1);
    // byte_95E7F8 copied to g_installSource; dword_51BC90 its length with the NUL
    let mut n = 0;
    loop {
        let b = rb(a(INSTALLED) + n);
        wb(a(SOURCE_VAR) + n, b);
        n += 1;
        if b == 0 {
            break;
        }
    }
    wd(a(SOURCE_SIZE), n);
    i::write_registry::write_registry(&mut game_api)?;
    i::register_lobby::register_lobby(&mut game_api)?;
    p(ADD_ITEMS, 1);
    p(INSTALLING, 1);
    if g(OFFER_REGISTRATION) == 1 {
        p(RUN_REGISTRATION, 1);
    }
    Ok(())
}
