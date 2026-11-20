//! Reads the game's kept options, `var\ral.dat`: up to 0x10000 bytes of the file go into the
//! scratch block; it is taken if it starts "R5CI" and has 0x50004 at +8. From +0x10 on the 69
//! pieces (0xB656 bytes) are the options, applied to the game's state (`saves::options_state`).
//! After the last piece the file must have "####", else fatal error 0x80ACFF01 "ral.dat out of
//! date" (the options are applied by then). Then `saves::rebuild_car_lists`, and the loaded flag (0
//! until then) becomes 1. A file shorter than that is not noticed as such: what the scratch block
//! held decides. `saves::save_options` writes the same pieces the other way.

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::read_file::read_file_to_store;
use crate::memory::cells::Store;
use crate::owners::Owners;
use crate::saves::options_state;
use crate::saves::rebuild_car_lists::rebuild_car_lists;
use crate::settings::TEXT_ROOM;
use rc2k_core::options::Options;

/// Where the function works: the game's own places, or a test's.
pub struct Places<'a> {
    /// The buffer the file is read into.
    pub file: &'a crate::memory::session::PlainBytes<65536>,
    pub loaded: &'a dyn crate::memory::session::FlagLike,
    /// Puts the options read into the game's state.
    pub apply: &'a mut dyn FnMut(&Options) -> Result<(), Refused>,
    pub rebuild_car_lists: &'a mut dyn FnMut() -> Result<(), BadInput>,
}

pub fn load_options(path: &dyn Store, places: &mut Places) -> Result<bool, Refused> {
    places.loaded.set(false);
    let file = places.file;
    let read = read_file_to_store(
        &crate::files::cstring(path, "the options file's path")?,
        file,
        TEXT_ROOM + 1,
        0,
    )?;
    use rc2k_core::saves::kept;
    let block = file.read_at(0, TEXT_ROOM as usize + 1, "the options file")?;
    if !read.ok || !kept::head_ok(&block, kept::OPTIONS) {
        return Ok(false);
    }
    let (pieces, ended) = kept::pieces(&block, &Options::piece_lengths())?;
    let mut options = Options::default();
    if !options.take_pieces(&pieces) {
        return Err(BadInput("the options' pieces do not fit the model").into());
    }
    (places.apply)(&options)?;
    if !ended {
        return Err(FatalError {
            code: 0x80AC_FF01,
            message: "ral.dat out of date",
        }
        .into());
    }
    (places.rebuild_car_lists)()?;
    places.loaded.set(true);
    Ok(true)
}

pub(super) fn load_game_options(o: Owners<'static>) -> Result<bool, Refused> {
    let menus = o.menus;
    load_options(
        &o.saves.files.options_path,
        &mut Places {
            file: &o.system.work().files.buffer,
            loaded: &menus.screens.loop_flags.options_loaded,
            apply: &mut |options| options_state::apply(o, options),
            rebuild_car_lists: &mut || rebuild_car_lists(menus),
        },
    )
}
