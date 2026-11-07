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
    lock, normalise, pump, rb, run_frame, set_cd, show, sleep, store_text, take_key, text_at,
    unlock, wb,
};
use crate::install::name_tables::{FILES, FOLDERS};
use crate::place::Place;
use crate::system::owner::system;

fn game_dir() -> Place {
    Place::from(&crate::install::owner::install().paths.game_dir)
}

fn installed() -> Place {
    Place::from(&crate::install::owner::install().paths.installed)
}

fn path_a() -> Place {
    Place::from(&crate::install::owner::install().paths.path_a)
}

fn path_b() -> Place {
    Place::from(&crate::install::owner::install().paths.path_b)
}

/// `to` = `base` + name `index` of `table`, NUL-ended.
fn build(
    base: Place,
    to: Place,
    table: &crate::memory::owned::Owned<crate::install::name_tables::NameTable>,
    index: u32,
) -> Result<(), Refused> {
    let mut path = text_at(base);
    crate::install::name_tables::push_name(table, index, &mut path)?;
    path.push(0);
    store_text(to, &path);
    Ok(())
}

/// The error (count 2, `unk_51D453`, y 0) or cancel (count 1, `unk_51D463`, y -0x32)
/// message shown, then a wait for a key.
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
        if system().session.flags.quitting.get() == 1 {
            break;
        }
    }
    Ok(())
}

/// The installer's page frames (`screens::run_frame` runs them).
fn copying_frame(g: &'static crate::install::owner::Install) -> Result<bool, Refused> {
    crate::install::install_frames::copying_frame(g)
}

fn progress_frame(g: &'static crate::install::owner::Install) -> Result<bool, Refused> {
    crate::install::install_frames::progress_frame(g)
}

pub fn install_files(g: &'static crate::install::owner::Install) -> Result<(), Refused> {
    use crate::install as i;
    g.names.status.set(0x80);
    i::write_registry::write_registry(g, &mut crate::install::portable::Portable)?;
    normalise(&g.paths.game_dir, &g.paths.game_dir)?;
    i::run_script::run_script(g, &g.names.script_count)?;
    g.run.files_copied.set(0);
    g.run.copy_spare.set(0);
    g.run.files_to_copy.set(g.run.file_count.get());
    let room = crate::install::installer::rd(
        crate::place::Place::from(&g.names.room_by_mode)
            .add(4 * crate::install::installer::media_mode()),
    );
    g.run.room_needed.set(room);
    // The path is bad (or there is no room, or it is not a CD): the page is told.
    let bad_path = || {
        g.run.path_problem.set(1);
        g.pages.list_answer.set(0);
        Ok(())
    };
    if !normalise(&g.paths.typed, &g.paths.installed)? {
        return bad_path();
    }
    if i::check_folder_name::check_folder_name(&g.paths.installed) {
        return bad_path();
    }
    // The room where it goes, by the system's rules (`install::folders`; the original
    // reads the drive's, not a network path's).
    let folder = Place::from(&g.paths.installed).text("the install folder")?;
    match <i::folders::Native as i::folders::Folders>::room(&folder) {
        i::folders::Room::Unreadable => return bad_path(),
        i::folders::Room::Kb(free)
            if free < g.run.room_needed.get() && g.run.ignore_room.get() != 1 =>
        {
            g.run.path_problem.set(2);
            g.pages.list_answer.set(3);
            return Ok(());
        }
        _ => {}
    }
    if !i::make_folders::make_folders(&g.paths.installed)? {
        return bad_path();
    }
    // Its answer is not read: the standalone installs from the download folder as well
    // (owner, 2026-10-20).
    crate::files::is_cd_drive::is_cd_drive(rb(&g.paths.game_dir));
    // A close during the copy waits for its end (the progress still shown).
    crate::platform_events::hold_quit(true);
    run_frame(Some(|| copying_frame(crate::install::owner::install())))?;
    // the folders
    if g.run.dir_count.get() != 0 {
        for n in 0..g.run.dir_count.get() {
            build(installed(), path_b(), &FOLDERS, n)?;
            if !i::make_folders::make_folders(&g.paths.path_b)? {
                return message(2, &g.pages.msg_error, 0);
            }
        }
        set_cd(&g.paths.game_dir);
    }
    // the files
    if g.run.file_count.get() != 0 {
        let mut n = 0;
        loop {
            build(game_dir(), path_a(), &FILES, n)?;
            build(installed(), path_b(), &FILES, n)?;
            // 0x413242: a failed copy is the error message.
            if !i::copy_counted::copy_counted(g, &g.paths.path_a, &g.paths.path_b)? {
                return message(2, &g.pages.msg_error, 0);
            }
            if run_frame(Some(|| progress_frame(crate::install::owner::install())))? {
                run_frame(Some(|| copying_frame(crate::install::owner::install())))?;
            }
            if take_key() == 1 {
                return message(1, &g.pages.msg_cancel, 0xFFFF_FFCE);
            }
            n = n.wrapping_add(1);
            if n == g.run.file_count.get() {
                break;
            }
        }
    }
    // 0x413280: the change of folder's result is not looked at; 0x41328A: a failed
    // record write is the error message.
    set_cd(&g.paths.installed);
    if !i::write_record::write_record(g)? {
        return message(2, &g.pages.msg_error, 0);
    }
    if crate::install::installer::rb(crate::files::state::locale_directory_place()) != 0 {
        // readme.txt then manual.pdf
        for (full, base) in [
            (&g.names.readme, Place::from(&g.names.readme).add(0xE)),
            (&g.names.manual, Place::from(&g.names.manual).add(0xE)),
        ] {
            let mut path = text_at(&g.paths.game_dir);
            path.extend(text_at(full));
            path.push(0);
            store_text(&g.paths.path_a, &path);
            // Not looked at (0x413435, 0x413463).
            i::copy_counted::copy_counted(g, &g.paths.path_a, base)?;
        }
    }
    crate::platform_events::hold_quit(false);
    g.pages.list_answer.set(2);
    sleep(1000);
    g.names.status.set(1);
    // byte_95E7F8 copied to g_installSource; dword_51BC90 its length with the NUL
    let mut n = 0;
    loop {
        let b = rb(Place::from(&g.paths.installed).add(n));
        wb(Place::from(&g.names.source).add(n), b);
        n += 1;
        if b == 0 {
            break;
        }
    }
    g.names.source_size.set(n);
    i::write_registry::write_registry(g, &mut crate::install::portable::Portable)?;
    i::register_lobby::register_lobby(g, &mut crate::install::portable::Portable)?;
    g.run.add_items.set(1);
    g.run.installing.set(1);
    if g.run.offer_registration.get() == 1 {
        g.run.run_registration.set(1);
    }
    Ok(())
}
