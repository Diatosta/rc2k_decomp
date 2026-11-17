//! Writes the best times and scores, `var\hst.dat`.
//!
//! Only when the tables' memory is taken and they are filled (the allocated and loaded flags both
//! 1). A head of 0x10 bytes is made ("R5HS", 0, 0x50004, the time); the file is opened (mode 1:
//! made anew); the head and the six tables `saves::load_best_times` reads are written, each with
//! the size its pair of variables holds; the file is closed. A file that cannot be opened, or a write that falls short, is a failure
//! (logged; the result is false). Refuses what `resolve_path` refuses.

use crate::errors::Refused;
use crate::files::handle::{close, open_by_mode};
use crate::saves::owner::Saves;
use crate::system::packed_local_time::packed_local_time;

/// The tables of `allocate_best_times` the file holds, in the file's order.
const WRITTEN: [usize; 6] = [1, 2, 3, 4, 7, 8];

/// On the game's own file and tables: true if the file was opened and every write worked.
pub fn in_game(saves: &Saves) -> Result<bool, Refused> {
    let best_times = &saves.best_times;
    if [best_times.allocated.get(), best_times.loaded.get()] != [1, 1] {
        return Ok(false);
    }
    let head =
        rc2k_core::saves::kept::head(rc2k_core::saves::kept::BEST_TIMES, packed_local_time());
    let Some(file) = open_by_mode(
        &crate::saves::load_best_times::PATH[..crate::saves::load_best_times::PATH_LEN],
        1,
    )?
    else {
        return Ok(false);
    };
    let mut whole = crate::files::handle::write_slice(file, &head).1;
    for table in WRITTEN {
        let bytes = saves.best_tables.typed(table).bytes()?;
        whole &= crate::files::handle::write_slice(file, &bytes).1;
    }
    close(file);
    if !whole {
        crate::logfile::program_log::log("saves: the best times file was not written whole");
    }
    Ok(whole)
}
