//! `install::install_files` (0x4130CD): the install itself. The status is marked busy and
//! written; the source is scanned (`install::run_script` with `script_count`) to count the
//! files and the room needed (`room_by_mode[g_mediaMode]`); the destination is normalised,
//! checked for a bad name (`install::check_folder_name`), for room
//! (`files::drive_free_space_kb` unless UNC) and made (`install::make_folders`); the source
//! must be a CD (`files::is_cd_drive`). Then the page is shown and the folders made and the
//! files copied one by one (`install::make_folders`, `install::copy_counted`), a frame drawn
//! after each and Escape watched; the record is written (`install::write_record`); the readme
//! and manual are copied if a language is set; finally the status is set installed, written,
//! the lobby registered (`install::register_lobby`), the Start-menu add and
//! `g_installing` asked, and registration if offered. Failures set `list_answer`
//! (0 bad path, 3 no room / not a CD) or show an error/cancel message.
//!
//! The action of a list item. Caller `install::run_page`.

use crate::errors::Refused;
use crate::install::installer::{
    lock, normalise, pump, rb, run_frame, set_cd, show, sleep, store_text, take_key, text_at,
    unlock, wb,
};
use crate::install::name_tables::{FILES, FOLDERS};
use crate::install::vars::StringRecord;
use crate::place::BytePlace;

fn game_dir(install: &'static crate::install::owner::Install) -> BytePlace {
    BytePlace::from(&install.paths.game_dir)
}

fn installed(install: &'static crate::install::owner::Install) -> BytePlace {
    BytePlace::from(&install.paths.installed)
}

fn path_a(install: &'static crate::install::owner::Install) -> BytePlace {
    BytePlace::from(&install.paths.path_a)
}

fn path_b(install: &'static crate::install::owner::Install) -> BytePlace {
    BytePlace::from(&install.paths.path_b)
}

/// `to` = `base` + name `index` of `table`, NUL-ended.
fn build(
    base: BytePlace,
    to: BytePlace,
    table: &crate::memory::owned::Owned<crate::install::name_tables::NameTable>,
    index: u32,
) -> Result<(), Refused> {
    let mut path = text_at(base);
    crate::install::name_tables::push_name(table, index, &mut path)?;
    path.push(0);
    store_text(to, &path);
    Ok(())
}

/// The error message's two records: its own and the cancel message's that follows it in the
/// pages (the original draws two records from the error message's place).
fn error_messages(install: &'static crate::install::owner::Install) -> [&'static StringRecord; 2] {
    [&install.pages.msg_error[0], &install.pages.msg_cancel[0]]
}

/// The error (count 2, `InstallerPages.msg_error`, y 0) or cancel (count 1, `InstallerPages.msg_cancel`, y -0x32)
/// message shown, then a wait for a key.
fn message(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    system: &crate::system::owner::System,
    table: &[&StringRecord],
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
    input: &crate::input::owner::Input,
) -> Result<(), Refused> {
    crate::platform_events::hold_quit(system, false);
    run_frame(system, None)?;
    if lock(gfx, system) {
        crate::install::draw_string_list::draw_string_list(so, table, 0, y, gfx)?;
        unlock(gfx);
        show();
    }
    sleep(500);
    take_key(input);
    loop {
        if take_key(input) != 0 {
            break;
        }
        sleep(1);
        pump();
        if system.session.flags.quitting.is_set() {
            break;
        }
    }
    Ok(())
}

/// The installer's page frames (`screens::run_frame` runs them).
fn copying_frame(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    install: &'static crate::install::owner::Install,
    gfx: &'static crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    crate::install::install_frames::copying_frame(so, install, gfx, system)
}

fn progress_frame(
    install: &'static crate::install::owner::Install,
    strings: &crate::strings::owner::Strings,
    gfx: &'static crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    crate::install::install_frames::progress_frame(install, strings, gfx, system)
}

pub fn install_files(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    use crate::install as i;
    let so = crate::strings::append_string_by_id::StringOwners::of(o);
    install.names.status.set(0x80);
    i::write_registry::write_registry(install, &mut crate::install::portable::Portable)?;
    normalise(&install.paths.game_dir, &install.paths.game_dir)?;
    i::run_script::run_script(
        install,
        crate::place::BytePlace::in_store(
            &install.names,
            core::mem::offset_of!(crate::install::vars::InstallerNames, script_count),
        ),
    )?;
    install.run.files_copied.set(0);
    install.run.copy_spare.set(0);
    install.run.files_to_copy.set(install.run.file_count.get());
    let room = crate::install::installer::rd(
        crate::place::BytePlace::from(&install.names.room_by_mode)
            .add(4 * crate::install::installer::media_mode()),
    );
    install.run.room_needed.set(room);
    // The path is bad (or there is no room, or it is not a CD): the page is told.
    let bad_path = || {
        install.run.path_problem.set(1);
        install.pages.list_answer.set(0);
        Ok(())
    };
    if !normalise(&install.paths.typed, &install.paths.installed)? {
        return bad_path();
    }
    if i::check_folder_name::check_folder_name(&install.paths.installed) {
        return bad_path();
    }
    // The room where it goes, by the system's rules (`install::folders`; the original
    // reads the drive's, not a network path's).
    let folder = BytePlace::from(&install.paths.installed).text("the install folder")?;
    match <i::folders::Native as i::folders::Folders>::room(&folder, o.system) {
        i::folders::Room::Unreadable => return bad_path(),
        i::folders::Room::Kb(free)
            if free < install.run.room_needed.get() && install.run.ignore_room.get() != 1 =>
        {
            install.run.path_problem.set(2);
            install.pages.list_answer.set(3);
            return Ok(());
        }
        _ => {}
    }
    if !i::make_folders::make_folders(&install.paths.installed)? {
        return bad_path();
    }
    // Its answer is not read: the standalone installs from the download folder as well
    // (owner).
    crate::files::is_cd_drive::is_cd_drive(rb(&install.paths.game_dir));
    // A close during the copy waits for its end (the progress still shown).
    crate::platform_events::hold_quit(o.system, true);
    run_frame(
        o.system,
        Some(&|| copying_frame(so, install, o.gfx, o.system)),
    )?;
    // the folders
    if install.run.dir_count.get() != 0 {
        for n in 0..install.run.dir_count.get() {
            build(installed(install), path_b(install), &FOLDERS, n)?;
            if !i::make_folders::make_folders(&install.paths.path_b)? {
                return message(so, o.system, &error_messages(install), 0, o.gfx, o.input);
            }
        }
        set_cd(&install.paths.game_dir);
    }
    // the files
    if install.run.file_count.get() != 0 {
        let mut n = 0;
        loop {
            build(game_dir(install), path_a(install), &FILES, n)?;
            build(installed(install), path_b(install), &FILES, n)?;
            // 0x413242: a failed copy is the error message.
            if !i::copy_counted::copy_counted(
                install,
                o.system,
                &install.paths.path_a,
                &install.paths.path_b,
            )? {
                return message(so, o.system, &error_messages(install), 0, o.gfx, o.input);
            }
            if run_frame(
                o.system,
                Some(&|| progress_frame(o.install, o.strings, o.gfx, o.system)),
            )? {
                run_frame(
                    o.system,
                    Some(&|| copying_frame(so, install, o.gfx, o.system)),
                )?;
            }
            if take_key(o.input) == 1 {
                return message(
                    so,
                    o.system,
                    &install.pages.msg_cancel.each_ref(),
                    0xFFFF_FFCE,
                    o.gfx,
                    o.input,
                );
            }
            n = n.wrapping_add(1);
            if n == install.run.file_count.get() {
                break;
            }
        }
    }
    // 0x413280: the change of folder's result is not looked at; 0x41328A: a failed
    // record write is the error message.
    set_cd(&install.paths.installed);
    if !i::write_record::write_record(install, o.strings)? {
        return message(so, o.system, &error_messages(install), 0, o.gfx, o.input);
    }
    if crate::files::state::directory_byte(1) != 0 {
        // readme.txt then manual.pdf
        for (full, base) in [
            (
                &install.names.readme,
                BytePlace::from(&install.names.readme).add(0xE),
            ),
            (
                &install.names.manual,
                BytePlace::from(&install.names.manual).add(0xE),
            ),
        ] {
            let mut path = text_at(&install.paths.game_dir);
            path.extend(text_at(full));
            path.push(0);
            store_text(&install.paths.path_a, &path);
            // Not looked at (0x413435, 0x413463).
            i::copy_counted::copy_counted(install, o.system, &install.paths.path_a, base)?;
        }
    }
    crate::platform_events::hold_quit(o.system, false);
    install.pages.list_answer.set(2);
    sleep(1000);
    install.names.status.set(1);
    // The installed folder's path copied to `g_installSource`, with its length (NUL included)
    // kept in `source_size`.
    let mut n = 0;
    loop {
        let b = rb(BytePlace::from(&install.paths.installed).add(n));
        wb(BytePlace::from(&install.names.source).add(n), b);
        n += 1;
        if b == 0 {
            break;
        }
    }
    install.names.source_size.set(n);
    i::write_registry::write_registry(install, &mut crate::install::portable::Portable)?;
    i::register_lobby::register_lobby(
        install,
        &mut crate::install::portable::Portable,
        o.strings,
        o.net,
    )?;
    install.run.add_items.set(1);
    install.run.installing.set(1);
    if install.run.offer_registration.get() == 1 {
        install.run.run_registration.set(1);
    }
    Ok(())
}
