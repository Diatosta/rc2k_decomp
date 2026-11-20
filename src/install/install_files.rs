//! The install itself, the action of a list item: the source is scanned to count the files and the
//! room needed, the destination checked (bad name, room, folders made, source a CD), then the
//! folders made and the files copied one by one with a frame after each and Escape watched, the
//! record written, the readme and manual copied if a language is set, and finally the status set to
//! installed and the shortcuts asked for. How it ended is returned
//! (`flow::Copied`): a folder that cannot be used, too little room, or the copy stopped with its
//! error / cancel message shown. Always the full installation.

use crate::errors::Refused;
use crate::install::installer::{
    hold, lock, normalise, pause, pump, rb, run_frame, set_cd, show, sleep, store_text, take_key,
    text_at, unlock, wb,
};

use crate::install::vars::StringRecord;
use crate::memory::place::BytePlace;
use rc2k_core::install::flow::Copied;

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

/// Whether file `index` of the CD's list is not installed: the CD's `ral.exe` (the 1999 program,
/// which this one replaces), and on macOS every file outside the CD's folders (the readmes and the
/// manual), the application holding only the game's files.
fn left_out(install: &'static crate::install::owner::Install, index: u32) -> Result<bool, Refused> {
    let mut name = Vec::new();
    crate::install::name_tables::push_name(&install.files, index, &mut name)?;
    let top_level = !name.iter().any(|&c| c == b'\\' || c == b'/');
    Ok(top_level && (name.eq_ignore_ascii_case(b"ral.exe") || cfg!(target_os = "macos")))
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
/// `InstallerPages.msg_cancel`, y -0x32) message shown, then a wait for a key: the copy
/// is stopped.
fn message(
    o: crate::owners::Owners<'static>,
    table: &[&StringRecord],
    y: u32,
) -> Result<Copied, Refused> {
    let (system, gfx, input) = (o.system, o.gfx, o.input);
    crate::shell::events::hold_quit(system, false);
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
    hold(o, 500);
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
    Ok(Copied::Stopped)
}

pub(super) fn install_files(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<Copied, Refused> {
    use crate::install as i;
    crate::install::installer::set_full_installation();
    install.names.status.set(0x80);
    i::record::write_installed(install);
    normalise(&install.paths.game_dir, &install.paths.game_dir)?;
    i::run_script::run_script(
        install,
        crate::memory::place::BytePlace::from(&install.names.script_count),
    )?;
    install.run.files_copied.set(0);
    install.run.copy_spare.set(0);
    install.run.files_to_copy.set(install.run.file_count.get());
    let room = crate::install::installer::rd(
        crate::memory::place::BytePlace::from(&install.names.room_by_mode)
            .add(4 * crate::install::installer::FULL_INSTALLATION),
    );
    install.run.room_needed.set(room);
    // The folder cannot be used: the folder page says so.
    let bad_path = || {
        install.run.path_problem.set(1);
        Ok(Copied::BadFolder)
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
            if free < install.run.room_needed.get() && !install.flags.ignore_room.get() =>
        {
            install.run.path_problem.set(2);
            return Ok(Copied::NoRoom);
        }
        _ => {}
    }
    if !i::make_folders::make_folders(&install.paths.installed)? {
        return bad_path();
    }
    // A close during the copy waits for its end (the progress still shown).
    crate::shell::events::hold_quit(o.system, true);
    run_frame(
        o,
        Some(&|| crate::install::install_frames::copying_frame(o)),
    )?;
    // On macOS the program's own bundle is installed, the game's files going inside it.
    #[cfg(target_os = "macos")]
    {
        let support = crate::files::host_path(&install.paths.installed.text());
        if let Err(why) = crate::install::mac_bundle::begin(&support) {
            crate::program_log::log(&format!("install: the application: {why}"));
            return message(o, &error_messages(install), 0);
        }
    }
    // the folders
    if install.run.dir_count.get() != 0 {
        for n in 0..install.run.dir_count.get() {
            build(installed(install), path_b(install), &install.folders, n)?;
            if !i::make_folders::make_folders(&install.paths.path_b)? {
                return message(o, &error_messages(install), 0);
            }
        }
        set_cd(&install.paths.game_dir);
    }
    // the files
    if install.run.file_count.get() != 0 {
        let mut n = 0;
        // The bar is drawn after this much copying since the last frame ended (about 15 frames a
        // second): a frame per file (about 3,000 files) waits for the display's refresh each
        // time, which costs far more than the copying.
        const FRAME_GAP: std::time::Duration = std::time::Duration::from_millis(66);
        let mut last_frame: Option<std::time::Instant> = None;
        loop {
            build(game_dir(install), path_a(install), &install.files, n)?;
            build(installed(install), path_b(install), &install.files, n)?;
            // A failed copy is the error message.
            if !left_out(install, n)?
                && !i::copy_counted::copy_counted(
                    install,
                    &install.paths.path_a,
                    &install.paths.path_b,
                )?
            {
                return message(o, &error_messages(install), 0);
            }
            let last_file = n.wrapping_add(1) == install.run.file_count.get();
            if last_frame.is_none_or(|at| at.elapsed() >= FRAME_GAP) || last_file {
                if run_frame(
                    o,
                    Some(&|| crate::install::install_frames::progress_frame(o)),
                )? {
                    run_frame(
                        o,
                        Some(&|| crate::install::install_frames::copying_frame(o)),
                    )?;
                }
                last_frame = Some(std::time::Instant::now());
            } else {
                // The window's events and keys still go on between frames.
                run_frame(o, None)?;
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
    // On macOS only the game's files go into the application.
    if crate::files::state::directory_byte(1) != 0 && !cfg!(target_os = "macos") {
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
            i::copy_counted::copy_counted(install, &install.paths.path_a, base)?;
        }
    }
    crate::shell::events::hold_quit(o.system, false);
    pause(o, 1000);
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
    i::record::write_installed(install);
    install.run.add_items.set(1);
    install.flags.installing.set(true);
    Ok(Copied::Installed)
}
