//! The game's options in `var/rc2k.toml` and in the old `var\ral.dat`: which is read at start and
//! which is written (the rule is `rc2k_core::options`). The new file wins when it exists; the old
//! one is read only when the new one is not there yet (its values are brought over and the new
//! file written at once), with `--load-old-options`, or by `load_old`. The options are saved to
//! the new file; to the old one only with `--save-old-options` or by `save_old`.

use crate::errors::{BadInput, FatalError, Refused};
use crate::owners::Owners;
use crate::saves::load_options::load_game_options;
use crate::saves::options_state::{self, capture};
use crate::saves::owner::Saves;
use crate::saves::rebuild_car_lists::rebuild_car_lists;
use rc2k_core::options::files::{self, Asked, Paths, Store};
use rc2k_core::options::{Options, OptionsError};
use std::path::{Path, PathBuf};

/// Puts the options into the game's stores, as reading the old file does.
fn apply(o: Owners<'static>, options: &Options) -> Result<(), Refused> {
    options_state::apply(o, options)?;
    rebuild_car_lists(o.menus)?;
    o.menus.screens.loop_flags.options_loaded.set(true);
    Ok(())
}

/// Puts imported options into the game and saves them as a change on the options pages is
/// (the save code's import).
pub fn apply_and_save(o: Owners<'static>, options: &Options) -> Result<(), Refused> {
    apply(o, options)?;
    save(o)?;
    Ok(())
}

/// The options files on the disk.
struct Disk;

impl Store for Disk {
    fn read(&self, path: &Path) -> std::io::Result<Option<Vec<u8>>> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Written to a name beside the file, then put in place.
    fn write_whole(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        let mut temporary = path.as_os_str().to_owned();
        temporary.push(".tmp");
        let temporary = PathBuf::from(temporary);
        std::fs::write(&temporary, bytes)?;
        std::fs::rename(&temporary, path)
    }
}

fn paths(saves: &Saves) -> Result<Paths, Refused> {
    Ok(Paths {
        new: crate::files::host_path(b"var\\rc2k.toml"),
        old: crate::files::host_path(&crate::files::cstring(
            &saves.files.options_path,
            "the options file's path",
        )?),
    })
}

/// What the command line asked for.
fn asked() -> Asked {
    Asked::from_args(crate::platform::os::program_args())
}

/// A file's trouble as the program ends with it; the detail goes to standard error.
fn refusal(e: &OptionsError) -> Refused {
    eprintln!("options: {e}");
    match e {
        OptionsError::OutOfDate => FatalError {
            code: 0x80AC_FF01,
            message: "ral.dat out of date",
        }
        .into(),
        OptionsError::Text(_) => BadInput("var/rc2k.toml is not a valid options file").into(),
        OptionsError::TooNew(_) => BadInput("var/rc2k.toml is newer than this program").into(),
        OptionsError::Io(_) => BadInput("an options file could not be read or written").into(),
    }
}

/// Reads the old file into the stores (what a settings button will call, with
/// `--load-old-options` on the command line doing it at start). False if there is none.
pub fn load_old(o: Owners<'static>) -> Result<bool, Refused> {
    load_game_options(o)
}

/// Writes the old file from the stores.
pub fn save_old(o: Owners<'static>) -> Result<bool, Refused> {
    crate::saves::save_options::save_options(o, &o.saves.files.options_path)
}

/// The start-up load: the options into the stores. Whether any file was read.
pub fn load(o: Owners<'static>) -> Result<bool, Refused> {
    let paths = paths(o.saves)?;
    let base = capture(o)?;
    // The old file's own way is the game's reader; its trouble is kept to be raised as it is.
    let mut trouble = None;
    let mut read_old = || match load_old(o) {
        Ok(true) => capture(o).map(Some).map_err(|e| {
            trouble = Some(e);
            OptionsError::Io("the old options".into())
        }),
        Ok(false) => Ok(None),
        Err(e) => {
            trouble = Some(e);
            Err(OptionsError::Io("the old options".into()))
        }
    };
    let got = files::load(&Disk, &paths, asked(), &base, &mut read_old);
    if let Some(e) = trouble {
        return Err(e);
    }
    let got = got.map_err(|e| refusal(&e))?;
    // Values of the new file that were not valid are at their defaults: the player is told
    // (once the window is there).
    if let Some(loaded) = &got
        && !loaded.skipped.is_empty()
    {
        crate::error_dialog::notice(&crate::error_dialog::text_with(
            rc2k_core::strings::own::id::SETTINGS_RESET,
            &loaded.skipped.join(", "),
        ));
    }
    match got {
        Some(loaded) if loaded.source == files::Source::New => {
            apply(o, &loaded.options).map(|()| true)
        }
        // The old file's values are in the stores already.
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

/// The save: the new file always, the old one too when asked for.
pub fn save(o: Owners<'static>) -> Result<bool, Refused> {
    let paths = paths(o.saves)?;
    let options = capture(o)?;
    let mut trouble = None;
    let mut write_old = |_: &Options| {
        save_old(o).map(|_| ()).map_err(|e| {
            trouble = Some(e);
            OptionsError::Io("the old options".into())
        })
    };
    let done = files::save(&Disk, &paths, asked(), &options, &mut write_old);
    if let Some(e) = trouble {
        return Err(e);
    }
    done.map_err(|e| refusal(&e))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The options file is written whole through a name beside it, which does not stay, and is
    /// read back; a missing file is nothing.
    #[test]
    fn the_disk_store_writes_whole_and_reads_back() {
        let dir = std::env::temp_dir().join(format!("rc2k_options_disk_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("rc2k.toml");
        assert_eq!(Disk.read(&file).unwrap(), None);
        Disk.write_whole(&file, b"one").unwrap();
        Disk.write_whole(&file, b"two").unwrap();
        assert_eq!(Disk.read(&file).unwrap().as_deref(), Some(&b"two"[..]));
        assert!(!dir.join("rc2k.toml.tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The stores' options taken and put back are the same options.
    #[test]
    fn capture_and_apply_round_trip() {
        let _lock = crate::test_support::test_lock();
        let o = crate::test_support::own::owners();
        let loaded = o.menus.screens.loop_flags.options_loaded.is_set();
        let before = capture(o).unwrap();
        let saved: Vec<Vec<u8>> = options_state::pieces()
            .iter()
            .map(|p| p.read().unwrap())
            .collect();
        assert_eq!(before.to_pieces(), saved);
        apply(o, &before).unwrap();
        let after = capture(o).unwrap();
        // `apply` marks the options loaded, as the game's loader does: put the flag back.
        o.menus.screens.loop_flags.options_loaded.set(loaded);
        assert_eq!(after, before);
    }

    /// A file whose bytes differ by position, with the head and the end mark of a real one.
    fn varied_old_file() -> Vec<u8> {
        use rc2k_core::saves::kept;
        let mut file = kept::head(kept::OPTIONS, 0x0BAD_CAFE).to_vec();
        file.extend(
            (0..rc2k_core::options::old_file::FILE_LEN - kept::HEAD_LEN - 4)
                .map(|n| (n.wrapping_mul(2_654_435_761) >> 13) as u8),
        );
        file.extend_from_slice(&kept::END_MARK);
        // The two lists of car numbers (pieces 44 and 45) must end in -1 for the stores' car
        // lists to be rebuilt from them: both are empty lists here.
        let lengths = Options::piece_lengths();
        let start = kept::HEAD_LEN + lengths[..44].iter().sum::<usize>();
        let both = lengths[44] + lengths[45];
        file[start..start + both].fill(0xFF);
        file
    }

    /// Runs `body` with the stores holding `options`, then puts the stores' options back.
    fn with_options_in_stores<T>(options: &Options, body: impl FnOnce(Owners<'static>) -> T) -> T {
        let _lock = crate::test_support::test_lock();
        let o = crate::test_support::own::owners();
        let loaded = o.menus.screens.loop_flags.options_loaded.is_set();
        let before = capture(o).unwrap();
        apply(o, options).unwrap();
        let result = body(o);
        apply(o, &before).unwrap();
        o.menus.screens.loop_flags.options_loaded.set(loaded);
        result
    }

    /// Options read from an old file of varied bytes, put into the game's stores and taken out
    /// again, are written to the old file's very bytes: no store changes, drops or reorders a
    /// piece (the stores are the game's truth between load and save).
    #[test]
    fn old_file_through_the_stores_is_the_same_bytes() {
        use rc2k_core::options::old_file;
        let file = varied_old_file();
        let options = old_file::read(&file).unwrap().unwrap();
        let (again, pieces) = with_options_in_stores(&options, |o| {
            let pieces: Vec<Vec<u8>> = options_state::pieces()
                .iter()
                .map(|p| p.read().unwrap())
                .collect();
            (capture(o).unwrap(), pieces)
        });
        assert_eq!(again.to_pieces(), pieces);
        assert_eq!(old_file::write(&again, 0x0BAD_CAFE), file);
    }

    /// The options of a real-looking old file, written to `rc2k.toml`'s text and read back,
    /// are the same options; the text's checksum is frozen (a key renamed or a field moved
    /// shows here, and makes players' files unreadable).
    #[test]
    fn toml_through_the_stores_is_the_same_options_and_text() {
        use rc2k_core::options::{old_file, toml_file};
        let options = old_file::read(&varied_old_file()).unwrap().unwrap();
        let text = with_options_in_stores(&options, |o| {
            toml_file::write(&capture(o).unwrap(), None).unwrap()
        });
        // The file holds the texts up to their NULs: what a buffer keeps after its NUL is not
        // in the options read back.
        let held = rc2k_core::options::document::Document::from(&options)
            .into_options()
            .unwrap();
        assert_eq!(
            toml_file::read(&text, &Options::default())
                .unwrap()
                .map(|read| read.options),
            Some(held)
        );
        let sum = crate::test_support::hash(text.as_bytes());
        assert_eq!((text.len(), sum), (200_347, 9_774_075_395_527_410_901));
    }

    /// Loading the new file through the stores leaves the stores holding what it said, with
    /// keys it lacks kept as they were: a file of one section changes only that section.
    #[test]
    fn a_partial_toml_changes_only_what_it_names() {
        use rc2k_core::options::toml_file;
        let _lock = crate::test_support::test_lock();
        let o = crate::test_support::own::owners();
        let loaded = o.menus.screens.loop_flags.options_loaded.is_set();
        let before = capture(o).unwrap();
        let wanted = toml_file::read("[display]\nmenu_mode = 5\n", &before)
            .unwrap()
            .unwrap()
            .options;
        apply(o, &wanted).unwrap();
        let after = capture(o).unwrap();
        apply(o, &before).unwrap();
        o.menus.screens.loop_flags.options_loaded.set(loaded);
        let mut expected = before;
        expected.display.menu_mode = 5;
        assert_eq!(after, expected);
    }
}
