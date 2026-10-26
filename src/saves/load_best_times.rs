//! `saves::load_best_times` (0x4AB155): reads the best times and scores, `var\hst.dat`.
//!
//! In:  nothing
//! Out: carry = not loaded (loaded: carry and zero clear); every register preserved
//! First `saves::allocate_best_times`; the file is opened (mode 0); its head of 0x10 bytes
//! goes to the scratch block at 0x93E8B0 and must start "R5HS" and have 0x50004 at +8; then
//! six of the tables are read straight from the file, each to the address and with the size
//! its pair of variables holds (tables 1, 2, 3, 4, 7 and 8 of `allocate_best_times`: 0xBB8
//! bytes, an entry of 0x618 a stage, 0x15E0, 0xDB60, 0xDB60, an entry a stage again); the
//! file is closed and `dword_747614` = 1.
//! A read that fails or gives fewer bytes than asked for (`sub_4AB21C`, inlined) closes
//! the file and ends it there, with what was read so far left in the tables. A head that
//! is not the right one ends it without the file being closed: the original's slip, kept.
//! One caller, `sub_411239`. Refuses what `resolve_path` and `allocate_best_times` refuse.

use crate::errors::{BadInput, Refused};
use crate::files::handle::{close, open_by_mode, read_to};
use crate::pointers::GamePtr;
use crate::saves::allocate_best_times::allocate_best_times;
use crate::saves::best_times_store::BEST_TIMES;

// ---- game data ----

/// IDA's address of the best times file's name.
pub const PATH_START: usize = 0x5295E4;

/// `aVarHstDat`: the best times file's name.
pub static PATH: crate::session::Bytes<12> = crate::session::Bytes::new(*b"var\\hst.dat\0");

/// Where the name is (`moved::MOVED`'s home for IDA 0x5295E4).
pub fn path_home() -> usize {
    PATH.addr() as usize
}

// "R5HS"
const HEAD_LEN: u32 = 0x10;
/// The tables of `allocate_best_times` the file holds, in the file's order.
const READ: [usize; 6] = [1, 2, 3, 4, 7, 8];

// ---- Rust version ----

/// Reads exactly `count` bytes into `to`, or closes the file and says false.
fn read_or_close(file: u32, to: GamePtr, count: u32) -> Result<bool, BadInput> {
    let (got, ok) = read_to(file, to, count, "the best times file's data")?;
    if !ok || got != count {
        close(file);
        return Ok(false);
    }
    Ok(true)
}

/// `saves::load_best_times` on the game's own file and tables: true if the file was loaded.
pub fn load_game_best_times() -> Result<bool, Refused> {
    allocate_best_times(
        BEST_TIMES.list(),
        crate::records::stages_store::STAGE_TABLES.championship(), // g_recordTable
    )?;
    let Some(file) = open_by_mode(crate::pointers::host(PATH.addr()) as *const u8, 0)? else {
        return Ok(false);
    };
    let scratch = GamePtr::new(crate::files::scratch::SCRATCH.buffer.addr());
    if !read_or_close(file, scratch, HEAD_LEN)? {
        return Ok(false);
    }
    // Converted: `rc2k_core::saves::kept::head_ok`.
    let head = scratch.read_bytes(HEAD_LEN as usize, "the best times file's head")?;
    if !rc2k_core::saves::kept::head_ok(&head, rc2k_core::saves::kept::BEST_TIMES) {
        // The file stays open, as in the original.
        return Ok(false);
    }
    for table in READ {
        let place = &BEST_TIMES.tables[table];
        if !read_or_close(file, GamePtr::new(place.address.get()), place.size.get())? {
            return Ok(false);
        }
    }
    close(file);
    BEST_TIMES.loaded.set(1);
    Ok(true)
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&PATH, PATH_START, PATH_START + 12);
        assert_eq!(crate::moved::place(PATH_START), PATH.addr() as usize);
    }
}
