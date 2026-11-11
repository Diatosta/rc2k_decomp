//! `saves::save_options` (0x4AAE57): writes the game's kept options, `var\ral.dat`.
//!
//! The file is put together in the scratch block at 0x93E8B0: "R5CI", 0, 0x50004, the time
//! (`system::packed_local_time`), the 69 pieces of `saves::load_options` in the same order, each
//! copied from its place, and "####"; then `files::write_file`. One caller,
//! `saves::save_game_data`. Refuses what `write_file` refuses.

use crate::errors::Refused;
use crate::files::write_file::write_file_from_store;
use crate::owners::Owners;
#[cfg(test)]
use crate::saves::load_options::pieces;
use crate::saves::load_options::{PIECES_LEN, pieces_of};
use crate::system::packed_local_time::packed_local_time;

// ---- game data ----

// "var\ral.dat"
const HEAD_LEN: usize = 0x10;
/// The whole file's length.
pub fn file_len() -> usize {
    HEAD_LEN + PIECES_LEN + 4
}

/// True if the file was written.
pub fn save_options(o: Owners<'_>, path: &dyn crate::pointers::Store) -> Result<bool, Refused> {
    let file = &o.system.work.files.buffer;
    let mut owned = Vec::new();
    for piece in pieces_of(o) {
        owned.push(piece.read()?);
    }
    let pieces: Vec<&[u8]> = owned.iter().map(std::vec::Vec::as_slice).collect();
    let bytes = rc2k_core::saves::kept::options_file(packed_local_time(), &pieces);
    file.write_at(0, &bytes, "the scratch buffer")?;
    Ok(write_file_from_store(
        &crate::files::cstring(path, "the options file's path")?,
        file,
        file_len() as u32,
    )?
    .ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The options file's bytes (time 0) from the stores' first values: a checksum fixed
    /// before the pieces became typed fields.
    #[test]
    fn options_file_bytes_are_unchanged() {
        let _lock = crate::store::test_lock();
        let mut owned = Vec::new();
        for piece in pieces() {
            owned.push(piece.read().unwrap());
        }
        let refs: Vec<&[u8]> = owned.iter().map(Vec::as_slice).collect();
        let bytes = rc2k_core::saves::kept::options_file(0, &refs);
        let sum = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
        });
        assert_eq!((bytes.len(), sum), (file_len(), 2_461_667_189_847_992_428));
    }
}
