//! `saves::read_savegame_slot` (0x4AB4FE): reads a saved game, `savedata\savegame\svgameNN`,
//! into the buffer at 0x51EBB4 and copies its name into the saved game list.
//!
//! Takes slot, 0 .. 99.
//! It reads up to 0x8000 bytes. The file is accepted if the read did not fail, the first
//! dword is "R5SG" and the dword at +8 is 0x51122; the 0x20 bytes at +0x10 then go to
//! `byte_526BB4[slot * 0x20]`. A short file is not noticed as such: what the buffer held
//! before decides (reproduced). It only fills the buffer: putting the state back into the
//! game is done by its callers `screens::draw_savegame_preview` and `saves::load_savegame`, which
//! are not ported. Called for all 100 slots at start-up (`system::load_game_data`). Refuses a slot
//! past 99.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::read_file::read_file_into;
use crate::menu::name_lists::NAME_LEN;
use crate::saves::owner::saves;
use crate::saves::write_savegame_slot::{FILE_LEN, SLOTS, slot_name};

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub fn read_savegame_slot(slot: u32) -> Result<bool, Refused> {
    let saves = saves();
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    let template = saves.files.savegame_template.read();
    let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
    let mut bytes = saves.save_games.file.read();
    let read = read_file_into(&path[..], &mut bytes[..FILE_LEN], 0)?;
    saves.save_games.file.write(&bytes);
    // Converted: `rc2k_core::saves::kept::slot_name`.
    use rc2k_core::saves::kept;
    let block = &bytes[..0x30];
    match kept::slot_name(block, kept::SAVEGAME, kept::SAVEGAME_VERSION) {
        Some(name) if read.ok => {
            slot_name(slot).write(&name[..NAME_LEN], "a savegame slot's name")?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

//
// Slots without a file take the failing path. Files with a wrong head are only covered if
// such files happen to exist.
