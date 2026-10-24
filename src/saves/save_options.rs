//! `saves::save_options` (0x4AAE57): writes the game's kept options, `var\ral.dat`.
//!
//! In:  nothing
//! Out: every register preserved; carry = not written
//! The file is put together in the scratch block at 0x93E8B0: "R5CI", 0, 0x50004, the
//! time (`system::packed_local_time`), the 69 pieces of `saves::load_options` in the
//! same order, each copied from its place, and "####"; then `files::write_file`.
//! One caller, `saves::save_game_data`.
//! Shadow mode writes the file twice (Rust, then the original) and compares what was put
//! together, but for the time. Self-test: a redirected copy that writes a file of the
//! test's; the two files are compared but for the time.
//! Refuses what `write_file` refuses.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::files::write_file::write_file;
use crate::framework::game_addr;
use crate::saves::load_options::OPTIONS;
use crate::settings::TEXT_RVA;
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
        let file = game_addr(TEXT_RVA) as *mut u8;
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
