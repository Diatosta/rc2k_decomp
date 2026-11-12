//! `saves::save_best_times` (0x4AB236): writes the best times and scores, `var\hst.dat`.
//!
//! Only when the tables' memory is taken and they are filled (`g_bestTimesAllocated` and
//! `g_bestTimesLoaded` both 1). A head of 0x10 bytes is made in the scratch block at 0x93E8B0
//! ("R5HS", 0, 0x50004, the time); the file is opened (mode 1: made anew); the head and the six
//! tables `saves::load_best_times` reads are written, each from the address and with the size its
//! pair of variables holds; the file is closed. Whether a write worked is not looked at: only a
//! file that cannot be opened gives the carry. One caller, `saves::save_game_data`. Refuses what
//! `resolve_path` refuses.

use crate::errors::Refused;
use crate::files::handle::{close, open_by_mode};
use crate::saves::owner::Saves;
use crate::system::owner::System;
use crate::system::packed_local_time::packed_local_time;

// ---- game data ----

const HEAD_LEN: u32 = 0x10;
/// The tables of `allocate_best_times` the file holds, in the file's order.
const WRITTEN: [usize; 6] = [1, 2, 3, 4, 7, 8];

/// `saves::save_best_times` on the game's own file and tables: true if the file was opened
/// (and so, as far as the function knows, written).
pub fn in_game(saves: &Saves, system: &System) -> Result<bool, Refused> {
    let best_times = &saves.best_times;
    if [best_times.allocated.get(), best_times.loaded.get()] != [1, 1] {
        return Ok(false);
    }
    // Converted: `rc2k_core::saves::kept::head`.
    let head =
        rc2k_core::saves::kept::head(rc2k_core::saves::kept::BEST_TIMES, packed_local_time());
    for (n, &byte) in head.iter().enumerate() {
        system.work().files.buffer.set(n, byte);
    }
    let Some(file) = open_by_mode(
        &crate::saves::load_best_times::PATH[..crate::saves::load_best_times::PATH_LEN],
        1,
    )?
    else {
        return Ok(false);
    };
    crate::files::handle::write_from_store(file, &system.work().files.buffer, HEAD_LEN)?;
    for table in WRITTEN {
        let bytes = crate::saves::best_times::typed(table).bytes()?;
        crate::files::handle::write_slice(file, &bytes);
    }
    close(file);
    Ok(true)
}
