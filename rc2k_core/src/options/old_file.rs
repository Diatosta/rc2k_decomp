//! The old options file, `var\ral.dat`: the head ("R5CI", 0, 0x50004, the time), the 69
//! pieces, "####" (`saves::kept`). Read and written byte for byte as the game always has.

use super::OptionsError;
use super::model::{Options, PIECES_LEN};
use crate::saves::kept;

/// The whole file's length.
pub const FILE_LEN: usize = kept::HEAD_LEN + PIECES_LEN + 4;

/// The options in the old file's bytes, with `time` (the game's packed local time) in its
/// head.
pub fn write(options: &Options, time: u32) -> Vec<u8> {
    let pieces = options.to_pieces();
    let refs: Vec<&[u8]> = pieces.iter().map(Vec::as_slice).collect();
    kept::options_file(time, &refs)
}

/// The options of the old file `bytes`. `Ok(None)` for bytes that are not an options file
/// of this version (the head is wrong), as the game takes them.
pub fn read(bytes: &[u8]) -> Result<Option<Options>, OptionsError> {
    if !kept::head_ok(bytes, kept::OPTIONS) {
        return Ok(None);
    }
    let lengths = Options::piece_lengths();
    let (pieces, ended) = kept::pieces(bytes, &lengths).map_err(|_| OptionsError::OutOfDate)?;
    if !ended {
        return Err(OptionsError::OutOfDate);
    }
    let mut options = Options::default();
    let right = options.take_pieces(&pieces);
    debug_assert!(right);
    Ok(Some(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file whose every byte differs from its neighbours' by position.
    pub fn sample_file() -> Vec<u8> {
        let mut file = kept::head(kept::OPTIONS, 0x1234_5678).to_vec();
        file.extend((0..PIECES_LEN).map(|n| (n * 7 + n / 256) as u8));
        file.extend_from_slice(&kept::END_MARK);
        file
    }

    #[test]
    fn the_pieces_are_the_games_sizes() {
        let lengths = Options::piece_lengths();
        assert_eq!(lengths.len(), 69);
        assert_eq!(lengths.iter().sum::<usize>(), PIECES_LEN);
        #[rustfmt::skip]
        let games: [usize; 69] = [0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x40, 0x4, 0x40, 0x4, 0x10, 0x4, 0x4, 0x4, 0x20, 0x4, 0x4, 0x4, 0x4, 0x4, 0x28C, 0x28C, 0x28C, 0x28C, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x2C, 0x18, 0x4, 0x4, 0x4, 0x6, 0xA0, 0x50, 0x2A48, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0x47C, 0xC30, 0xC30, 0xC30, 0xC30, 0xC30, 0xC30, 0x20, 0x20, 0x20, 0x20];
        assert_eq!(lengths, games);
        assert_eq!(FILE_LEN, 0x10 + 0xB656 + 4);
    }

    #[test]
    fn a_file_read_and_written_again_is_the_same_bytes() {
        let file = sample_file();
        let options = read(&file).unwrap().unwrap();
        assert_eq!(write(&options, 0x1234_5678), file);
    }

    #[test]
    fn a_value_lands_in_its_place() {
        let mut file = sample_file();
        // The second piece (the menus' screen mode) is the dword after the first.
        file[0x14..0x18].copy_from_slice(&7u32.to_le_bytes());
        // The last piece is the fourth player's name.
        let end = FILE_LEN - 4;
        file[end - 0x20] = b'Z';
        let options = read(&file).unwrap().unwrap();
        assert_eq!(options.display.menu_mode, 7);
        assert_eq!(options.players.names[3].0[0], b'Z');
        assert_eq!(
            options.kept.runs,
            u32::from_le_bytes(file[0x10..0x14].try_into().unwrap())
        );
    }

    #[test]
    fn a_wrong_head_is_not_an_options_file_and_a_missing_end_is_out_of_date() {
        let mut file = sample_file();
        file[FILE_LEN - 1] = b'!';
        assert_eq!(read(&file), Err(OptionsError::OutOfDate));
        assert_eq!(read(&file[..100]), Err(OptionsError::OutOfDate));
        file[0] = b'X';
        assert_eq!(read(&file), Ok(None));
        assert_eq!(read(&[]), Ok(None));
    }

    /// With `RC2K_OLD_OPTIONS` naming a real `ral.dat`, it reads and writes back unchanged.
    #[test]
    fn a_real_file_is_the_same_bytes() {
        let Some(path) = std::env::var_os("RC2K_OLD_OPTIONS") else {
            return;
        };
        let file = std::fs::read(path).unwrap();
        let options = read(&file).unwrap().unwrap();
        let time = u32::from_le_bytes(file[12..16].try_into().unwrap());
        assert_eq!(write(&options, time), file);
    }
}
