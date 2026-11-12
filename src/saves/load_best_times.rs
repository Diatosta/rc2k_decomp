//! `saves::load_best_times` (0x4AB155): reads the best times and scores, `var\hst.dat`.
//!
//! First `saves::allocate_best_times`; the file is opened (mode 0); its head of 0x10 bytes
//! goes to the scratch block at 0x93E8B0 and must start "R5HS" and have 0x50004 at +8; then
//! six of the tables are read straight from the file, each to the address and with the size
//! its pair of variables holds (tables 1, 2, 3, 4, 7 and 8 of `allocate_best_times`: 0xBB8
//! bytes, an entry of 0x618 a stage, 0x15E0, 0xDB60, 0xDB60, an entry a stage again); the
//! file is closed and `loaded` = 1.
//! A read that fails or gives fewer bytes than asked for (`saves::read_or_close`, inlined) closes
//! the file and ends it there, with what was read so far left in the tables. A head that
//! is not the right one ends it without the file being closed: the original's slip, kept.
//! One caller, `system::load_game_data`. Refuses what `resolve_path` and `allocate_best_times`
//! refuse.

use crate::errors::{BadInput, Refused};
use crate::files::handle::{close, open_by_mode};
use crate::saves::allocate_best_times::allocate_best_times;
use crate::saves::owner::Saves;
use crate::system::owner::System;

// ---- game data ----

/// `aVarHstDat`: the best times file's name.
pub static PATH: [u8; 12] = *b"var\\hst.dat\0";
/// The path's length without its NUL.
pub const PATH_LEN: usize = 11;

// "R5HS"
const HEAD_LEN: u32 = 0x10;
/// The tables of `allocate_best_times` the file holds, in the file's order.
const READ: [usize; 6] = [1, 2, 3, 4, 7, 8];

/// Reads exactly `count` bytes into `to`, or closes the file and says false.
fn read_or_close(file: u32, to: &dyn crate::cells::Store, count: u32) -> Result<bool, BadInput> {
    let (got, ok) = crate::files::handle::read_into_store(file, to, count)?;
    if !ok || got != count {
        close(file);
        return Ok(false);
    }
    Ok(true)
}

/// `saves::load_best_times` on the game's own file and tables: true if the file was loaded.
pub fn load_game_best_times(saves: &Saves, system: &System) -> Result<bool, Refused> {
    let best_times = &saves.best_times;
    allocate_best_times(
        best_times,
        &system.records().stage_tables,
        system.records().stage_tables.championship(), // g_recordTable
    )?;
    let Some(file) = open_by_mode(&PATH[..PATH_LEN], 0)? else {
        return Ok(false);
    };
    let scratch = &system.work().files.buffer;
    if !read_or_close(file, scratch, HEAD_LEN)? {
        return Ok(false);
    }
    // Converted: `rc2k_core::saves::kept::head_ok`.
    let head = scratch.read_at(0, HEAD_LEN as usize, "the best times file's head")?;
    if !rc2k_core::saves::kept::head_ok(&head, rc2k_core::saves::kept::BEST_TIMES) {
        // The file stays open, as in the original.
        return Ok(false);
    }
    for table in READ {
        let typed = crate::saves::best_times::typed(table);
        // Read over the table's bytes (a short read leaves the rest as it was).
        let mut bytes = typed.bytes()?;
        let (got, ok) = crate::files::handle::read_slice(file, &mut bytes);
        typed.set_bytes(&bytes)?;
        if !ok || got as usize != bytes.len() {
            close(file);
            return Ok(false);
        }
    }
    close(file);
    best_times.loaded.set(1);
    Ok(true)
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        assert_eq!(
            crate::test_support::first_values::bytes("saves.load_best_times.PATH"),
            &PATH
        );
    }
}
