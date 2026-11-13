//! `saves::write_savegame_slot` (0x4AB558): builds a saved game in the save-games buffer and
//! writes it to `savedata\savegame\svgameNN`.
//!
//! Takes slot, 0 .. 99.
//! The file is always 0x8000 bytes: "R5SG", 0, version 0x51122, `packed_local_time`, the slot's
//! 0x20-byte name from the saved game list (`names`), then what `gather_savegame` collects
//! (0x725C bytes); the rest of the buffer is written as it is. Only caller:
//! `menu::edit_savegame_name`. Refuses a slot past 99 and what `gather_savegame` refuses.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::write_file::write_file_from;
use crate::menu::name_lists::NAME_LEN;
use crate::menu::owner::Menus;
use crate::saves::gather_savegame::gather_savegame;
use crate::saves::owner::Saves;
use crate::session::{PlainBytes, PlainDword};
use crate::system::packed_local_time::packed_local_time;

// ---- game data ----

pub const FILE_LEN: usize = 0x8000;
pub const SLOTS: u32 = 100;

/// A saved game's name: 0x20 bytes.
pub const NAME_BYTES: usize = 0x20;

/// The saved games' buffers.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SaveGames {
    /// `dword_51EBB0`: a value the options file (`var\ral.dat`) keeps; nothing else reads it.
    pub kept_a: PlainDword,
    /// `dword_51EBB4`: the saved game as it is read or written (`FILE_LEN` bytes).
    pub file: PlainBytes<FILE_LEN>,
    /// `byte_526BB4`: the saved games' names, 100 of 0x20 bytes (ten to a page).
    pub names: PlainBytes<{ SLOTS as usize * NAME_BYTES }>,
    /// `dword_527834`: a value the options file keeps; nothing else reads it.
    pub kept_b: PlainDword,
}

/// The saved games' buffers; the first values are the image's (all zero).
impl SaveGames {
    pub const fn new() -> Self {
        SaveGames {
            kept_a: PlainDword::new(0),
            file: PlainBytes::new([0; FILE_LEN]),
            names: PlainBytes::new([0; SLOTS as usize * NAME_BYTES]),
            kept_b: PlainDword::new(0),
        }
    }
}

// "R5SG"
pub fn slot_name(saves: &'static Saves, slot: u32) -> crate::place::BytePlace {
    crate::place::BytePlace::In(&saves.save_games.names, slot as usize * NAME_LEN)
}

/// True if the file was written.
pub fn write_savegame_slot(
    menus: &Menus,
    saves: &'static Saves,
    slot: u32,
) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    {
        let file = &saves.save_games.file;
        // First, because it is what can be refused: nothing else is written before it.
        gather_savegame(menus, file, 0x30)?;
        // Converted: `rc2k_core::saves::kept::slot_head`.
        use rc2k_core::saves::kept;
        let head = kept::slot_head(
            kept::SAVEGAME,
            kept::SAVEGAME_VERSION,
            packed_local_time(),
            &slot_name(saves, slot).read_array::<NAME_LEN>("a savegame's name")?,
        );
        file.write_at(0, &head, "the savegame's file")?;
        let template = saves.files.savegame_template.read();
        let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
        Ok(write_file_from(&path[..], &file.read()[..FILE_LEN])?.ok)
    }
}
