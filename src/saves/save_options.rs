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
pub unsafe fn save_options(path: *const u8) -> Result<bool, Refused> {
    unsafe {
        let file = crate::files::scratch::SCRATCH.buffer.addr() as *mut u8;
        // Converted: `rc2k_core::saves::kept::options_file`, put together in the scratch
        // block as the original does.
        let pieces: Vec<&[u8]> = OPTIONS
            .iter()
            .map(|&(place, len)| {
                std::slice::from_raw_parts(crate::moved::place(place as usize) as *const u8, len)
            })
            .collect();
        let bytes = rc2k_core::saves::kept::options_file(packed_local_time(), &pieces);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), file, bytes.len());
        Ok(write_file(path, file, file_len() as u32)?.ok)
    }
}
