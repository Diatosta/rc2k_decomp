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
//! A list action (`off_51DD18`). Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    append_byte, lock, normalise, pump, rb, run_frame, set_cd, show, sleep, str_append, take_key,
    unlock, wb,
};
use crate::install::name_tables::{FILES, FOLDERS};
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PAGES;
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;
use crate::place::Place;
use crate::session::FLAGS;

fn game_dir() -> Place {
    Place::from(&INSTALLER_PATHS.game_dir)
}

fn installed() -> Place {
    Place::from(&INSTALLER_PATHS.installed)
}

fn path_a() -> Place {
    Place::from(&INSTALLER_PATHS.path_a)
}

fn path_b() -> Place {
    Place::from(&INSTALLER_PATHS.path_b)
}

/// `to` = `base` + name `index` of `table`, NUL-ended (`loc_4131A3`, `loc_4131F5`).
fn build(
    base: Place,
    to: Place,
    table: &crate::memory::owned::Owned<crate::install::name_tables::NameTable>,
    index: u32,
) -> Result<(), Refused> {
    let cursor = str_append(base, to);
    let cursor = crate::install::name_tables::append_name(table, index, cursor)?;
    append_byte(cursor, 0);
    Ok(())
}

/// The error (count 2, `unk_51D453`, y 0) or cancel (count 1, `unk_51D463`, y -0x32)
/// message shown, then a wait for a key (`loc_4133AB`).
fn message(count: u32, table: impl Into<crate::place::Place>, y: u32) -> Result<(), Refused> {
    let table: u32 = table.into().game();
    crate::platform_events::hold_quit(false);
    run_frame(None)?;
    if lock() {
        crate::install::draw_string_list::draw_string_list(count, table, 0, y)?;
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
    INSTALLER_NAMES.status.set(0x80);
    i::write_registry::write_registry(&mut crate::install::portable::Portable)?;
    normalise(&INSTALLER_PATHS.game_dir, &INSTALLER_PATHS.game_dir)?;
    i::run_script::run_script(&INSTALLER_NAMES.script_count)?;
    INSTALLER_RUN.files_copied.set(0);
    INSTALLER_RUN.copy_spare.set(0);
    INSTALLER_RUN
        .files_to_copy
        .set(INSTALLER_RUN.file_count.get());
    let room = crate::install::installer::rd(
        crate::place::Place::from(&INSTALLER_NAMES.room_by_mode)
            .add(4 * crate::install::installer::media_mode()),
    );
    INSTALLER_RUN.room_needed.set(room);
    // loc_413130: the path is bad; loc_4134D6: no room or not a CD.
    let bad_path = || {
        INSTALLER_RUN.path_problem.set(1);
        INSTALLER_PAGES.list_answer.set(0);
        Ok(())
    };
    if !normalise(&INSTALLER_PATHS.typed, &INSTALLER_PATHS.installed)? {
        return bad_path();
    }
    if i::check_folder_name::check_folder_name(&INSTALLER_PATHS.installed) {
        return bad_path();
    }
    // The room where it goes, by the system's rules (`install::folders`; the original
    // reads the drive's, not a network path's).
    let folder = Place::from(&INSTALLER_PATHS.installed).text("the install folder")?;
    match <i::folders::Native as i::folders::Folders>::room(&folder) {
        i::folders::Room::Unreadable => return bad_path(),
        i::folders::Room::Kb(free)
            if free < INSTALLER_RUN.room_needed.get() && INSTALLER_RUN.ignore_room.get() != 1 =>
        {
            INSTALLER_RUN.path_problem.set(2);
            INSTALLER_PAGES.list_answer.set(3);
            return Ok(());
        }
        _ => {}
    }
    if !i::make_folders::make_folders(&INSTALLER_PATHS.installed)? {
        return bad_path();
    }
    // Its answer is not read: the standalone installs from the download folder as well
    // (owner, 2026-10-20).
    crate::files::is_cd_drive::is_cd_drive(rb(&INSTALLER_PATHS.game_dir));
    // A close during the copy waits for its end (the progress still shown).
    crate::platform_events::hold_quit(true);
    run_frame(Some(copying_frame))?;
    // the folders
    if INSTALLER_RUN.dir_count.get() != 0 {
        for n in 0..INSTALLER_RUN.dir_count.get() {
            build(installed(), path_b(), &FOLDERS, n)?;
            if !i::make_folders::make_folders(&INSTALLER_PATHS.path_b)? {
                return message(2, &INSTALLER_PAGES.msg_error, 0);
            }
        }
        set_cd(&INSTALLER_PATHS.game_dir);
    }
    // the files
    if INSTALLER_RUN.file_count.get() != 0 {
        let mut n = 0;
        loop {
            build(game_dir(), path_a(), &FILES, n)?;
            build(installed(), path_b(), &FILES, n)?;
            // 0x413242: a failed copy is the error message.
            if !i::copy_counted::copy_counted(&INSTALLER_PATHS.path_a, &INSTALLER_PATHS.path_b)? {
                return message(2, &INSTALLER_PAGES.msg_error, 0);
            }
            if run_frame(Some(progress_frame))? {
                run_frame(Some(copying_frame))?;
            }
            if take_key() == 1 {
                return message(1, &INSTALLER_PAGES.msg_cancel, 0xFFFF_FFCE);
            }
            n = n.wrapping_add(1);
            if n == INSTALLER_RUN.file_count.get() {
                break;
            }
        }
    }
    // 0x413280: the change of folder's result is not looked at; 0x41328A: a failed
    // record write is the error message.
    set_cd(&INSTALLER_PATHS.installed);
    if !i::write_record::write_record()? {
        return message(2, &INSTALLER_PAGES.msg_error, 0);
    }
    if crate::install::installer::rb(crate::files::state::locale_directory_place()) != 0 {
        // readme.txt then manual.pdf
        for (full, base) in [
            (
                &INSTALLER_NAMES.readme,
                Place::from(&INSTALLER_NAMES.readme).add(0xE),
            ),
            (
                &INSTALLER_NAMES.manual,
                Place::from(&INSTALLER_NAMES.manual).add(0xE),
            ),
        ] {
            let cursor = str_append(&INSTALLER_PATHS.game_dir, &INSTALLER_PATHS.path_a);
            let cursor = str_append(full, cursor);
            append_byte(cursor, 0);
            // Not looked at (0x413435, 0x413463).
            i::copy_counted::copy_counted(&INSTALLER_PATHS.path_a, base)?;
        }
    }
    crate::platform_events::hold_quit(false);
    INSTALLER_PAGES.list_answer.set(2);
    sleep(1000);
    INSTALLER_NAMES.status.set(1);
    // byte_95E7F8 copied to g_installSource; dword_51BC90 its length with the NUL
    let mut n = 0;
    loop {
        let b = rb(Place::from(&INSTALLER_PATHS.installed).add(n));
        wb(Place::from(&INSTALLER_NAMES.source).add(n), b);
        n += 1;
        if b == 0 {
            break;
        }
    }
    INSTALLER_NAMES.source_size.set(n);
    i::write_registry::write_registry(&mut crate::install::portable::Portable)?;
    i::register_lobby::register_lobby(&mut crate::install::portable::Portable)?;
    INSTALLER_RUN.add_items.set(1);
    INSTALLER_RUN.installing.set(1);
    if INSTALLER_RUN.offer_registration.get() == 1 {
        INSTALLER_RUN.run_registration.set(1);
    }
    Ok(())
}
