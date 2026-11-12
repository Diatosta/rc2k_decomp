//! `saves::read_replay_slot` (0x41ECAD): reads a replay's head,
//! `savedata\replay\replayNN`, and copies its name into the replay list.
//!
//! Takes slot, 0 .. 99.
//! Up to 0x430 bytes to the replay header (`header`). Accepted if the read did not fail, the first dword
//! is "REPL" and the dword at +8 is 0x50400; the 0x20 bytes at +0x10 then go to
//! the replay names (`names[slot * 0x20]`). A short file is not noticed as such: what the buffer held
//! before decides (as in `saves::read_savegame_slot`).
//! One caller, `system::load_game_data`, for all 100 slots at start-up. Refuses a slot past 99.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::read_file::read_file_into;
use crate::menu::name_lists::NAME_LEN;
use crate::race::owner::Race;

// ---- game data ----

const HEAD_LEN: usize = 0x430;
const SLOTS: u32 = 100;

fn slot_name(race: &'static Race, slot: u32) -> crate::place::BytePlace {
    crate::place::BytePlace::In(&race.shared.replays().names, slot as usize * NAME_LEN)
}

/// True if the slot's file was read and accepted.
pub fn read_replay_slot(slot: u32, race: &'static Race) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    let template = race.shared.replay_templates.saved;
    let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
    let mut bytes = race.shared.replays().header.read();
    let read = read_file_into(&path[..], &mut bytes[..HEAD_LEN], 0)?;
    race.shared.replays().header.write(&bytes);
    // Converted: `rc2k_core::saves::kept::slot_name`.
    use rc2k_core::saves::kept;
    let block = &bytes[..0x30];
    match kept::slot_name(block, kept::REPLAY, kept::REPLAY_VERSION) {
        Some(name) if read.ok => {
            slot_name(race, slot).write(&name[..NAME_LEN], "a replay slot's name")?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
