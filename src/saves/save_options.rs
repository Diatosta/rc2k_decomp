//! `saves::save_options` (0x4AAE57): writes the game's kept options, `var\ral.dat`.
//!
//! In:  nothing
//! Out: every register preserved; carry = not written
//! The file is put together in the scratch block at 0x93E8B0: "R5CI", 0, 0x50004, the time
//! (`system::packed_local_time`), the 69 pieces of `saves::load_options` in the same order, each
//! copied from its place, and "####"; then `files::write_file`. One caller,
//! `saves::save_game_data`. Refuses what `write_file` refuses.

use crate::errors::Refused;
use crate::files::write_file::write_file;
use crate::pointers::GamePtr;
use crate::saves::load_options::OPTIONS;
use crate::system::packed_local_time::packed_local_time;

// ---- game data ----

// "var\ral.dat"
const HEAD_LEN: usize = 0x10;
/// The whole file's length.
pub fn file_len() -> usize {
    HEAD_LEN + OPTIONS.iter().map(|&(_, len)| len).sum::<usize>() + 4
}

// ---- Rust version ----

/// True if the file was written.
pub fn save_options(path: *const u8) -> Result<bool, Refused> {
    let file = GamePtr::new(crate::files::scratch::SCRATCH.buffer.addr());
    let mut owned = Vec::new();
    for &(place, len) in OPTIONS.iter() {
        owned.push(
            GamePtr::new((crate::moved::place(place as usize)) as u32)
                .read_bytes(len, "a saved option")?,
        );
    }
    let pieces: Vec<&[u8]> = owned.iter().map(|piece| piece.as_slice()).collect();
    let bytes = rc2k_core::saves::kept::options_file(packed_local_time(), &pieces);
    file.write_bytes(&bytes, "the scratch buffer")?;
    Ok(write_file(
        path,
        crate::pointers::host(file.addr()) as *const u8,
        file_len() as u32,
    )?
    .ok)
}
