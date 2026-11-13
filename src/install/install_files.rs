//! The install itself, the action of a list item: the source is scanned to count the files and
//! the room needed, the destination checked (bad name, room, folders made, source a CD), then
//! the folders made and the files copied one by one with a frame after each and Escape watched,
//! the record written, the readme and manual copied if a language is set, and finally the status
//! set to installed and the lobby, shortcuts and registration done. Failures set `list_answer`
//! (0 bad path, 3 no room / not a CD) or show an error / cancel message. Called by
//! `install::run_page`.

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
/// pages.
fn error_messages(install: &'static crate::install::owner::Install) -> [&'static StringRecord; 2] {
    [&install.pages.msg_error[0], &install.pages.msg_cancel[0]]
}

/// The error (count 2, `InstallerPages.msg_error`, y 0) or cancel (count 1,
/// `InstallerPages.msg_cancel`, y -0x32) message shown, then a wait for a key.
fn message(
    o: crate::owners::Owners<'static>,
    table: &[&StringRecord],
    y: u32,
) -> Result<(), Refused> {
    let (system, gfx, input) = (o.system, o.gfx, o.input);
    crate::platform_events::hold_quit(system, false);
    run_frame(o, None)?;
    if lock(gfx, system) {
        crate::install::draw_string_list::draw_string_list(
            crate::strings::append_string_by_id::StringOwners::of(o),
            table,
            0,
            y,
            gfx,
        )?;
        unlock(gfx);
        show(o);
    }
    sleep(500);
    take_key(input);
    loop {
        if take_key(input) != 0 {
            break;
        }
        sleep(1);
        pump(o);
        if system.session.flags.quitting.is_set() {
            break;
        }
    }
    Ok(())
}

pub fn install_files(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    use crate::install as i;
    install.names.status.set(0x80);
    i::write_registry::write_registry(install, &mut crate::install::portable::Portable)?;
    normalise(&install.paths.game_dir, &install.paths.game_dir)?;
    i::run_script::run_script(
        install,
        crate::place::BytePlace::from(&install.names.script_count),
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
    // The room where it goes, by the system's rules (`install::folders`).
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
        o,
        Some(&|| crate::install::install_frames::copying_frame(o)),
    )?;
    // the folders
    if install.run.dir_count.get() != 0 {
        for n in 0..install.run.dir_count.get() {
            build(installed(install), path_b(install), &FOLDERS, n)?;
            if !i::make_folders::make_folders(&install.paths.path_b)? {
                return message(o, &error_messages(install), 0);
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
            // A failed copy is the error message.
            if !i::copy_counted::copy_counted(
                install,
                o.system,
                &install.paths.path_a,
                &install.paths.path_b,
            )? {
                return message(o, &error_messages(install), 0);
            }
            if run_frame(
                o,
                Some(&|| crate::install::install_frames::progress_frame(o)),
            )? {
                run_frame(
                    o,
                    Some(&|| crate::install::install_frames::copying_frame(o)),
                )?;
            }
            if take_key(o.input) == 1 {
                return message(o, &install.pages.msg_cancel.each_ref(), 0xFFFF_FFCE);
            }
            n = n.wrapping_add(1);
            if n == install.run.file_count.get() {
                break;
            }
        }
    }
    // The change of folder's result is not looked at; a failed record write is the error
    // message.
    set_cd(&install.paths.installed);
    if !i::write_record::write_record(install, o.strings)? {
        return message(o, &error_messages(install), 0);
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
            // Not looked at.
            i::copy_counted::copy_counted(install, o.system, &install.paths.path_a, base)?;
        }
    }
    crate::platform_events::hold_quit(o.system, false);
    install.pages.list_answer.set(2);
    sleep(1000);
    install.names.status.set(1);
    // The installed folder's path copied to the install source, with its length (NUL included)
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
    i::register_lobby::register_lobby(install, &mut crate::install::portable::Portable, o.strings)?;
    install.run.add_items.set(1);
    install.run.installing.set(1);
    if install.run.offer_registration.get() == 1 {
        install.run.run_registration.set(1);
    }
    Ok(())
}
