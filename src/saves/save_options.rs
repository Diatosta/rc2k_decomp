//! Writes the game's kept options, `var\ral.dat`.
//!
//! The file is put together in the scratch block: "R5CI", 0, 0x50004, the time
//! (`system::packed_local_time`), the 69 pieces of `saves::options_state` in the same order, taken
//! from the captured options, and "####"; then `files::write_file`. One caller,
//! `saves::save_game_data`. Refuses what `write_file` refuses.

use crate::errors::Refused;
use crate::files::write_file::write_file_from_store;
use crate::owners::Owners;
use crate::saves::options_state::capture;
use crate::system::packed_local_time::packed_local_time;

const HEAD_LEN: usize = 0x10;
/// The whole file's length.
pub fn file_len() -> usize {
    HEAD_LEN + rc2k_core::options::model::PIECES_LEN + 4
}

/// True if the file was written.
pub fn save_options(
    o: Owners<'static>,
    path: &dyn crate::memory::cells::Store,
) -> Result<bool, Refused> {
    let file = &o.system.work().files.buffer;
    let owned = capture(o)?.to_pieces();
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
        let _lock = crate::test_support::test_lock();
        let owned = capture(crate::test_support::own::owners())
            .unwrap()
            .to_pieces();
        let refs: Vec<&[u8]> = owned.iter().map(Vec::as_slice).collect();
        let bytes = rc2k_core::saves::kept::options_file(0, &refs);
        let sum = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
        });
        assert_eq!((bytes.len(), sum), (file_len(), 2_461_667_189_847_992_428));
    }
}
