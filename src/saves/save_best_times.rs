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
use crate::files::handle::{close, open_by_mode, write};
use crate::saves::allocate_best_times::pair;
use crate::saves::best_times_store::BEST_TIMES;
use crate::system::packed_local_time::packed_local_time;

// ---- game data ----

const HEAD_LEN: u32 = 0x10;
/// The tables of `allocate_best_times` the file holds, in the file's order.
const WRITTEN: [usize; 6] = [1, 2, 3, 4, 7, 8];

// ---- Rust version ----

/// What the function is given: the game's own.
pub struct Places<'a> {
    pub path: *const u8,
    /// `g_bestTimesAllocated` and `g_bestTimesLoaded`.
    pub switches: [u32; 2],
    /// The address and the size of the `n`th table of the file.
    pub table: &'a dyn Fn(usize) -> (u32, u32),
}

/// True if the file was opened (and so, as far as the function knows, written).
pub unsafe fn save_best_times(places: &Places) -> Result<bool, Refused> {
    unsafe {
        if places.switches != [1, 1] {
            return Ok(false);
        }
        // Converted: `rc2k_core::saves::kept::head`.
        let head =
            rc2k_core::saves::kept::head(rc2k_core::saves::kept::BEST_TIMES, packed_local_time());
        for (n, &byte) in head.iter().enumerate() {
            crate::files::scratch::SCRATCH.buffer.set(n, byte);
        }
        let Some(file) = open_by_mode(places.path, 1)? else {
            return Ok(false);
        };
        write(
            file,
            crate::files::scratch::SCRATCH.buffer.addr() as *const u8,
            HEAD_LEN,
        );
        for n in 0..WRITTEN.len() {
            let (from, count) = (places.table)(n);
            write(file, from as *const u8, count);
        }
        close(file);
        Ok(true)
    }
}

fn game_table(n: usize) -> (u32, u32) {
    unsafe {
        let pair = pair(BEST_TIMES.vars(), WRITTEN[n]);
        (pair.read_unaligned(), pair.add(1).read_unaligned())
    }
}

/// `save_best_times` on the game's own file and tables.
pub fn in_game() -> Result<bool, Refused> {
    unsafe {
        save_best_times(&Places {
            path: crate::saves::load_best_times::PATH.addr() as *const u8,
            switches: [BEST_TIMES.allocated.get(), BEST_TIMES.loaded.get()],
            table: &game_table,
        })
    }
}
