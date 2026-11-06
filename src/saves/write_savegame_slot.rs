//! `saves::write_savegame_slot` (0x4AB558): builds a saved game in the buffer at 0x51EBB4 and
//! writes it to `savedata\savegame\svgameNN`.
//!
//! In:  ebx = slot, 0 .. 99
//! Out: carry = failed (zero flag = succeeded); every register preserved
//! The file is always 0x8000 bytes: "R5SG", 0, version 0x51122, `packed_local_time`, the slot's
//! 0x20-byte name from the saved game list (`byte_526BB4`), then what `gather_savegame` collects
//! (0x725C bytes); the rest of the buffer is written as it is. Only caller:
//! `menu::edit_savegame_name`. Refuses a slot past 99 and what `gather_savegame` refuses.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::write_file::write_file_from;
use crate::menu::name_lists::NAME_LEN;
use crate::saves::gather_savegame::gather_savegame;
use crate::saves::owner::saves;
use crate::session::{Bytes, Dword};
use crate::system::packed_local_time::packed_local_time;

// ---- game data ----

pub const FILE_LEN: usize = 0x8000;
pub const SLOTS: u32 = 100;

/// IDA's address of the saved games' variables.
pub const START: usize = 0x51EBB0;
/// Their length.
pub const LEN: usize = 0x527838 - START;
/// A saved game's name: 0x20 bytes.
pub const NAME_BYTES: usize = 0x20;

/// The saved games' buffers.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SaveGames {
    /// `dword_51EBB0`: a value the options file (`var\ral.dat`) keeps; nothing else reads it.
    pub kept_a: Dword,
    /// `dword_51EBB4`: the saved game as it is read or written (`FILE_LEN` bytes).
    pub file: Bytes<FILE_LEN>,
    /// `byte_526BB4`: the saved games' names, 100 of 0x20 bytes (ten to a page).
    pub names: Bytes<{ SLOTS as usize * NAME_BYTES }>,
    /// `dword_527834`: a value the options file keeps; nothing else reads it.
    pub kept_b: Dword,
}

/// The saved games' buffers; the first values are the image's (all zero).
impl SaveGames {
    pub const fn new() -> Self {
        SaveGames {
            kept_a: Dword::new(0),
            file: Bytes::new([0; FILE_LEN]),
            names: Bytes::new([0; SLOTS as usize * NAME_BYTES]),
            kept_b: Dword::new(0),
        }
    }
}

/// IDA's address of the file name's template.
pub const TEMPLATE_START: usize = 0x5296FB;

// "R5SG"
pub fn slot_name(slot: u32) -> crate::place::Place {
    crate::place::Place::In(&saves().save_games.names, slot as usize * NAME_LEN)
}

// ---- Rust version ----

/// True if the file was written.
pub fn write_savegame_slot(slot: u32) -> Result<bool, Refused> {
    let saves = saves();
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    {
        let file = &saves.save_games.file;
        // First, because it is what can be refused: nothing else is written before it.
        gather_savegame(crate::menu::owner::menus(), file, 0x30)?;
        // Converted: `rc2k_core::saves::kept::slot_head`.
        use rc2k_core::saves::kept;
        let head = kept::slot_head(
            kept::SAVEGAME,
            kept::SAVEGAME_VERSION,
            packed_local_time(),
            &slot_name(slot).read_array::<NAME_LEN>("a savegame's name")?,
        );
        file.write_at(0, &head, "the savegame's file")?;
        let template = saves.files.savegame_template.read();
        let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
        Ok(write_file_from(&path[..], &file.read()[..FILE_LEN])?.ok)
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::saves::owner::SAVES.save_games,
            START,
            START + LEN,
        );
        crate::gfx::first_values_are_the_games(
            &crate::saves::owner::SAVES.files.savegame_template,
            TEMPLATE_START,
            TEMPLATE_START + 27,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        let saves = saves();
        assert_eq!(offset_of!(SaveGames, kept_a), 0);
        assert_eq!(offset_of!(SaveGames, file), 0x51EBB4 - START);
        assert_eq!(offset_of!(SaveGames, names), 0x526BB4 - START);
        assert_eq!(offset_of!(SaveGames, kept_b), 0x527834 - START);
        assert_eq!(
            crate::moved::place(0x51EBB4),
            saves.save_games.file.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x526BB4),
            saves.save_games.names.addr() as usize
        );
        assert_eq!(
            crate::moved::place(TEMPLATE_START),
            saves.files.savegame_template.addr() as usize
        );
    }
}
