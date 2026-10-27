//! `saves::save_best_times` (0x4AB236): writes the best times and scores, `var\hst.dat`.
//!
//! In:  nothing
//! Out: every register preserved; carry = not written
//! Only when the tables' memory is taken and they are filled (`g_bestTimesAllocated` and
//! `g_bestTimesLoaded` both 1). A head of 0x10 bytes is made in the scratch block at 0x93E8B0
//! ("R5HS", 0, 0x50004, the time); the file is opened (mode 1: made anew); the head and the six
//! tables `saves::load_best_times` reads are written, each from the address and with the size its
//! pair of variables holds; the file is closed. Whether a write worked is not looked at: only a
//! file that cannot be opened gives the carry. One caller, `saves::save_game_data`. Refuses what
//! `resolve_path` refuses.

use crate::errors::Refused;
use crate::files::handle::{close, open_by_mode, write_from};
use crate::pointers::GamePtr;
use crate::saves::best_times_store::BEST_TIMES;
use crate::system::packed_local_time::packed_local_time;

// ---- game data ----

const HEAD_LEN: u32 = 0x10;
/// The tables of `allocate_best_times` the file holds, in the file's order.
const WRITTEN: [usize; 6] = [1, 2, 3, 4, 7, 8];

// ---- Rust version ----

/// `saves::save_best_times` on the game's own file and tables: true if the file was opened
/// (and so, as far as the function knows, written).
pub fn in_game() -> Result<bool, Refused> {
    if [BEST_TIMES.allocated.get(), BEST_TIMES.loaded.get()] != [1, 1] {
        return Ok(false);
    }
    // Converted: `rc2k_core::saves::kept::head`.
    let head =
        rc2k_core::saves::kept::head(rc2k_core::saves::kept::BEST_TIMES, packed_local_time());
    for (n, &byte) in head.iter().enumerate() {
        crate::files::scratch::SCRATCH.buffer.set(n, byte);
    }
    let Some(file) = open_by_mode(
        crate::pointers::host(crate::saves::load_best_times::PATH.addr()) as *const u8,
        1,
    )?
    else {
        return Ok(false);
    };
    let scratch = GamePtr::new(crate::files::scratch::SCRATCH.buffer.addr());
    write_from(file, scratch, HEAD_LEN, "the best times file's head")?;
    for table in WRITTEN {
        if table == crate::saves::allocate_best_times::SCORES_TABLE {
            let bytes = crate::saves::best_times::SCORES.with(|t| t.write())?;
            crate::files::handle::write_slice(file, &bytes);
            continue;
        }
        let place = &BEST_TIMES.tables[table];
        let table = place.block.need("a table of best times not taken")?;
        table.with_bytes(place.size.get() as usize, "a table of best times", |b| {
            crate::files::handle::write_slice(file, b)
        })?;
    }
    close(file);
    Ok(true)
}
