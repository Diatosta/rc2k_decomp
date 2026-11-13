//! The game's options in `var/rc2k.toml` and in the old `var\ral.dat`: which is read at start and
//! which is written (the rule is `rc2k_core::options`). The new file wins when it exists; the old
//! one is read only when the new one is not there yet (its values are brought over and the new
//! file written at once), with `--load-old-options`, or by `load_old`. The options are saved to
//! the new file; to the old one only with `--save-old-options` or by `save_old`.

use crate::errors::{BadInput, FatalError, Refused};
use crate::owners::Owners;
use crate::saves::load_options::{load_game_options, pieces_of};
use crate::saves::rebuild_car_lists::rebuild_car_lists;
use rc2k_core::options::files::{self, Asked, Paths};
use rc2k_core::options::{Options, OptionsError};

/// The options as the game's stores hold them now.
fn capture(o: Owners<'static>) -> Result<Options, Refused> {
    let mut owned = Vec::new();
    for piece in pieces_of(o) {
        owned.push(piece.read()?);
    }
    let refs: Vec<&[u8]> = owned.iter().map(Vec::as_slice).collect();
    let mut options = Options::default();
    if !options.take_pieces(&refs) {
        return Err(BadInput("the options' pieces do not fit the model").into());
    }
    Ok(options)
}

/// Puts the options into the game's stores, as reading the old file does.
fn apply(o: Owners<'static>, options: &Options) -> Result<(), Refused> {
    let pieces = options.to_pieces();
    for (place, bytes) in pieces_of(o).iter().zip(&pieces) {
        place.write(bytes)?;
    }
    rebuild_car_lists(o.menus)?;
    o.menus.screens.loop_flags.options_loaded.set(true);
    Ok(())
}

fn paths(o: Owners<'static>) -> Result<Paths, Refused> {
    Ok(Paths {
        new: crate::files::host_path(b"var\\rc2k.toml"),
        old: crate::files::host_path(&crate::files::cstring(
            &o.saves.files.options_path,
            "the options file's path",
        )?),
    })
}

/// What the command line asked for.
fn asked() -> Asked {
    Asked::from_args(std::env::args())
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
    let paths = paths(o)?;
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
    let got = files::load(&paths, asked(), &base, &mut read_old);
    if let Some(e) = trouble {
        return Err(e);
    }
    match got.map_err(|e| refusal(&e))? {
        Some((options, files::Source::New)) => apply(o, &options).map(|()| true),
        // The old file's values are in the stores already.
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

/// The save: the new file always, the old one too when asked for.
pub fn save(o: Owners<'static>) -> Result<bool, Refused> {
    let paths = paths(o)?;
    let options = capture(o)?;
    let mut trouble = None;
    let mut write_old = |_: &Options| {
        save_old(o).map(|_| ()).map_err(|e| {
            trouble = Some(e);
            OptionsError::Io("the old options".into())
        })
    };
    let done = files::save(&paths, asked(), &options, &mut write_old);
    if let Some(e) = trouble {
        return Err(e);
    }
    done.map_err(|e| refusal(&e))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The stores' options taken and put back are the same options.
    #[test]
    fn capture_and_apply_round_trip() {
        let _lock = crate::test_support::test_lock();
        let o = Owners::the();
        let loaded = o.menus.screens.loop_flags.options_loaded.is_set();
        let before = capture(o).unwrap();
        let saved: Vec<Vec<u8>> = pieces_of(o).iter().map(|p| p.read().unwrap()).collect();
        assert_eq!(before.to_pieces(), saved);
        apply(o, &before).unwrap();
        let after = capture(o).unwrap();
        // `apply` marks the options loaded, as the game's loader does: put the flag back.
        o.menus.screens.loop_flags.options_loaded.set(loaded);
        assert_eq!(after, before);
    }
}
